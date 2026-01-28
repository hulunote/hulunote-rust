use axum::{extract::State, Extension, Json};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::*;

use super::{get_database_id, AppState};

const ROOT_NAV_ID: &str = "00000000-0000-0000-0000-000000000000";

/// Create a new note
pub async fn create_note(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<Json<Value>> {
    let database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        req.database_id.as_deref(),
        req.database_name.as_deref().or(req.database.as_deref()),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    let note_id = Uuid::new_v4();
    let root_nav_id = Uuid::new_v4();

    // Create the note
    let note: HulunoteNote = sqlx::query_as(
        r#"
        INSERT INTO hulunote_notes (id, title, database_id, root_nav_id, account_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, title, database_id, root_nav_id, is_delete, is_public, 
                  is_shortcut, account_id, pv, created_at, updated_at
        "#,
    )
    .bind(note_id)
    .bind(&req.title)
    .bind(database_id.to_string())
    .bind(root_nav_id.to_string())
    .bind(account_id)
    .fetch_one(state.pool.as_ref())
    .await?;

    // Create the root nav for this note
    sqlx::query(
        r#"
        INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id)
        VALUES ($1, $2, 0, 'ROOT', $3, $4, $5)
        "#,
    )
    .bind(root_nav_id)
    .bind(ROOT_NAV_ID)
    .bind(account_id)
    .bind(note_id.to_string())
    .bind(database_id.to_string())
    .execute(state.pool.as_ref())
    .await?;

    Ok(Json(json!(NoteInfo::from(note))))
}

/// Get note list by page
pub async fn get_note_list(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<GetNoteListRequest>,
) -> Result<Json<Value>> {
    let database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        req.database_id.as_deref(),
        req.database_name.as_deref().or(req.database.as_deref()),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    let page = req.page.unwrap_or(1).max(1);
    let size = req.size.unwrap_or(100).min(1000);
    let offset = (page - 1) * size;

    // Get total count
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM hulunote_notes WHERE database_id = $1 AND is_delete = false"
    )
    .bind(database_id.to_string())
    .fetch_one(state.pool.as_ref())
    .await?;

    let all_pages = (count.0 as f64 / size as f64).ceil() as i64;

    // Get notes
    let notes: Vec<HulunoteNote> = sqlx::query_as(
        r#"
        SELECT id, title, database_id, root_nav_id, is_delete, is_public, 
               is_shortcut, account_id, pv, created_at, updated_at
        FROM hulunote_notes 
        WHERE database_id = $1 AND is_delete = false
        ORDER BY updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(database_id.to_string())
    .bind(size)
    .bind(offset)
    .fetch_all(state.pool.as_ref())
    .await?;

    let note_list: Vec<NoteInfo> = notes.into_iter().map(NoteInfo::from).collect();

    Ok(Json(json!({
        "note-list": note_list,
        "all-pages": all_pages
    })))
}

/// Get all notes in a database
pub async fn get_all_note_list(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<GetNoteListRequest>,
) -> Result<Json<Value>> {
    let database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        req.database_id.as_deref(),
        req.database_name.as_deref().or(req.database.as_deref()),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    let notes: Vec<HulunoteNote> = sqlx::query_as(
        r#"
        SELECT id, title, database_id, root_nav_id, is_delete, is_public, 
               is_shortcut, account_id, pv, created_at, updated_at
        FROM hulunote_notes 
        WHERE database_id = $1 AND is_delete = false
        ORDER BY updated_at DESC
        "#,
    )
    .bind(database_id.to_string())
    .fetch_all(state.pool.as_ref())
    .await?;

    let note_list: Vec<NoteInfo> = notes.into_iter().map(NoteInfo::from).collect();

    Ok(Json(json!({
        "note-list": note_list
    })))
}

/// Update a note
pub async fn update_note(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<Json<Value>> {
    let note_uuid = Uuid::parse_str(&req.note_id)
        .map_err(|_| AppError::BadRequest("Invalid note ID".to_string()))?;

    // Check ownership
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT account_id FROM hulunote_notes WHERE id = $1"
    )
    .bind(note_uuid)
    .fetch_optional(state.pool.as_ref())
    .await?;

    match exists {
        Some((owner_id,)) if owner_id != account_id => {
            return Err(AppError::PermissionDenied("Cannot update other's note".to_string()));
        }
        None => {
            return Err(AppError::NotFound("Note not found".to_string()));
        }
        _ => {}
    }

    // Build update
    if let Some(title) = &req.title {
        sqlx::query("UPDATE hulunote_notes SET title = $1, updated_at = NOW() WHERE id = $2")
            .bind(title)
            .bind(note_uuid)
            .execute(state.pool.as_ref())
            .await?;
    }

    if let Some(is_delete) = req.is_delete {
        sqlx::query("UPDATE hulunote_notes SET is_delete = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_delete)
            .bind(note_uuid)
            .execute(state.pool.as_ref())
            .await?;
    }

    if let Some(is_public) = req.is_public {
        sqlx::query("UPDATE hulunote_notes SET is_public = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_public)
            .bind(note_uuid)
            .execute(state.pool.as_ref())
            .await?;
    }

    if let Some(is_shortcut) = req.is_shortcut {
        sqlx::query("UPDATE hulunote_notes SET is_shortcut = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_shortcut)
            .bind(note_uuid)
            .execute(state.pool.as_ref())
            .await?;
    }

    Ok(Json(json!({"success": true})))
}

/// Get shortcut notes
pub async fn get_shortcuts_note_list(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<GetNoteListRequest>,
) -> Result<Json<Value>> {
    let database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        req.database_id.as_deref(),
        req.database_name.as_deref().or(req.database.as_deref()),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    let notes: Vec<HulunoteNote> = sqlx::query_as(
        r#"
        SELECT id, title, database_id, root_nav_id, is_delete, is_public, 
               is_shortcut, account_id, pv, created_at, updated_at
        FROM hulunote_notes 
        WHERE database_id = $1 AND is_delete = false AND is_shortcut = true
        ORDER BY updated_at DESC
        "#,
    )
    .bind(database_id.to_string())
    .fetch_all(state.pool.as_ref())
    .await?;

    let note_list: Vec<NoteInfo> = notes.into_iter().map(NoteInfo::from).collect();

    Ok(Json(json!({
        "note-list": note_list
    })))
}
