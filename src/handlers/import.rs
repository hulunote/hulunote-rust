use axum::extract::{Multipart, State};
use axum::Extension;
use axum::Json;
use serde_json::{json, Value};
use std::io::Read;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::*;

use super::{get_database_id, AppState};

const ROOT_NAV_ID: &str = "00000000-0000-0000-0000-000000000000";

/// Extract JSON files from uploaded data.
/// If the file is a .zip, extracts all .json entries inside it.
/// Otherwise treats the file as a plain JSON file.
fn collect_json_files(filename: &str, data: Vec<u8>) -> Result<Vec<(String, Vec<u8>)>> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".zip") {
        let cursor = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| AppError::BadRequest(format!("Invalid ZIP file {}: {}", filename, e)))?;

        let mut files = Vec::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                AppError::BadRequest(format!("Failed to read ZIP entry: {}", e))
            })?;

            let entry_name = entry.name().to_string();
            // Skip directories and non-json files
            if entry.is_dir() || !entry_name.to_lowercase().ends_with(".json") {
                continue;
            }

            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| {
                AppError::BadRequest(format!("Failed to read ZIP entry {}: {}", entry_name, e))
            })?;
            files.push((entry_name, buf));
        }
        Ok(files)
    } else {
        Ok(vec![(filename.to_string(), data)])
    }
}

/// Import notes from uploaded JSON / ZIP files (multipart form)
///
/// Form fields:
///   - `database-id` or `database-name`: target database
///   - one or more file fields: JSON files or ZIP archives containing JSON files
pub async fn import_notes(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    let mut database_id_str: Option<String> = None;
    let mut database_name_str: Option<String> = None;
    let mut json_files: Vec<(String, Vec<u8>)> = Vec::new();

    // Parse multipart fields
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "database-id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read field: {}", e)))?;
                database_id_str = Some(text);
            }
            "database-name" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read field: {}", e)))?;
                database_name_str = Some(text);
            }
            _ => {
                // Treat as file upload
                let filename = field
                    .file_name()
                    .unwrap_or("unknown.json")
                    .to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?;

                // Expand ZIP files into individual JSON files
                let extracted = collect_json_files(&filename, data.to_vec())?;
                json_files.extend(extracted);
            }
        }
    }

    if json_files.is_empty() {
        return Err(AppError::BadRequest(
            "No JSON files uploaded (or ZIP contains no .json files)".to_string(),
        ));
    }

    // Resolve target database
    let database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        database_id_str.as_deref(),
        database_name_str.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    let database_id_s = database_id.to_string();

    let mut imported: Vec<Value> = Vec::new();
    let mut errors: Vec<Value> = Vec::new();

    for (filename, data) in &json_files {
        match import_single_note(
            state.pool.as_ref(),
            account_id,
            &database_id_s,
            filename,
            data,
        )
        .await
        {
            Ok(info) => imported.push(info),
            Err(e) => errors.push(json!({
                "file": filename,
                "error": e.to_string()
            })),
        }
    }

    Ok(Json(json!({
        "success": true,
        "imported-count": imported.len(),
        "error-count": errors.len(),
        "imported": imported,
        "errors": errors
    })))
}

/// Import a single note JSON file into the database
async fn import_single_note(
    pool: &sqlx::PgPool,
    account_id: i64,
    database_id: &str,
    filename: &str,
    data: &[u8],
) -> Result<Value> {
    let import_data: ImportNoteJson = serde_json::from_slice(data)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON in {}: {}", filename, e)))?;

    let note_data = &import_data.note;

    // Parse original IDs
    let note_id = Uuid::parse_str(&note_data.id)
        .map_err(|_| AppError::BadRequest(format!("Invalid note ID in {}", filename)))?;
    let root_nav_id = Uuid::parse_str(&note_data.root_nav_id)
        .map_err(|_| AppError::BadRequest(format!("Invalid root nav ID in {}", filename)))?;

    // Check if note with same ID already exists
    let exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM hulunote_notes WHERE id = $1")
            .bind(note_id)
            .fetch_optional(pool)
            .await?;
    if exists.is_some() {
        return Err(AppError::BadRequest(format!(
            "Note {} already exists (id={}), skipped",
            note_data.title, note_data.id
        )));
    }

    // Also check title uniqueness within this database
    let title_exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM hulunote_notes WHERE database_id = $1 AND title = $2 AND is_delete = false",
    )
    .bind(database_id)
    .bind(&note_data.title)
    .fetch_optional(pool)
    .await?;
    if title_exists.is_some() {
        return Err(AppError::BadRequest(format!(
            "Note with title '{}' already exists in this database, skipped",
            note_data.title
        )));
    }

    // Begin transaction
    let mut tx = pool.begin().await?;

    // Insert the note
    sqlx::query(
        r#"
        INSERT INTO hulunote_notes (id, title, database_id, root_nav_id, is_delete, is_public, is_shortcut, account_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(note_id)
    .bind(&note_data.title)
    .bind(database_id)
    .bind(root_nav_id.to_string())
    .bind(note_data.is_delete.unwrap_or(false))
    .bind(note_data.is_public.unwrap_or(false))
    .bind(note_data.is_shortcut.unwrap_or(false))
    .bind(account_id)
    .execute(&mut *tx)
    .await?;

    // Insert root nav
    sqlx::query(
        r#"
        INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id)
        VALUES ($1, $2, 0, 'ROOT', $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(root_nav_id)
    .bind(ROOT_NAV_ID)
    .bind(account_id)
    .bind(note_id.to_string())
    .bind(database_id)
    .execute(&mut *tx)
    .await?;

    // Insert all navs
    let mut nav_count: usize = 0;
    for nav in &import_data.navs {
        let nav_id = Uuid::parse_str(&nav.id)
            .map_err(|_| AppError::BadRequest(format!("Invalid nav ID: {}", nav.id)))?;

        let is_display = nav.is_display.unwrap_or(true);
        let is_delete = nav.is_delete.unwrap_or(false);

        sqlx::query(
            r#"
            INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, is_display, is_delete)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(nav_id)
        .bind(&nav.parid)
        .bind(nav.same_deep_order as f32)
        .bind(&nav.content)
        .bind(account_id)
        .bind(note_id.to_string())
        .bind(database_id)
        .bind(is_display)
        .bind(is_delete)
        .execute(&mut *tx)
        .await?;

        nav_count += 1;
    }

    tx.commit().await?;

    Ok(json!({
        "file": filename,
        "note-id": note_id.to_string(),
        "title": note_data.title,
        "nav-count": nav_count
    }))
}
