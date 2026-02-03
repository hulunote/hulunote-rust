use axum::{extract::State, Extension, Json};
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::*;

use super::{get_database_id, AppState};

const ROOT_NAV_ID: &str = "00000000-0000-0000-0000-000000000000";

/// Get database ID by note ID (note.id is UUID, note.database_id is VARCHAR)
async fn get_database_id_by_note(pool: &sqlx::PgPool, note_id: &str) -> Result<Option<String>> {
    let note_uuid = Uuid::parse_str(note_id)
        .map_err(|_| AppError::BadRequest("Invalid note ID format".to_string()))?;

    let result: Option<(String,)> = sqlx::query_as(
        "SELECT database_id FROM hulunote_notes WHERE id = $1"
    )
    .bind(note_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.0))
}

/// Create or update a nav (outline node)
pub async fn create_or_update_nav(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<CreateOrUpdateNavRequest>,
) -> Result<Json<Value>> {
    // Get database_id (as String since database_id column is VARCHAR)
    let mut database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        req.database_id.as_deref(),
        req.database_name.as_deref().or(req.database.as_deref()),
    )
    .await?
    .map(|u| u.to_string());

    // If no database_id, try to get from note
    if database_id.is_none() {
        database_id = get_database_id_by_note(state.pool.as_ref(), &req.note_id).await?;
    }

    let database_id = database_id
        .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    let now = Utc::now();
    let backend_ts = now.timestamp_millis();

    // Check if nav exists (update) or create new
    if let Some(nav_id) = &req.id {
        let nav_uuid = Uuid::parse_str(nav_id)
            .map_err(|_| AppError::BadRequest("Invalid nav ID".to_string()))?;

        // Check if exists (nav.id is UUID)
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM hulunote_navs WHERE id = $1"
        )
        .bind(nav_uuid)
        .fetch_optional(state.pool.as_ref())
        .await?;

        if exists.is_some() {
            // Update existing nav
            if let Some(content) = &req.content {
                sqlx::query("UPDATE hulunote_navs SET content = $1, updated_at = NOW() WHERE id = $2")
                    .bind(content)
                    .bind(nav_uuid)
                    .execute(state.pool.as_ref())
                    .await?;
            }

            // parid is VARCHAR, use String
            if let Some(parid) = &req.parid {
                sqlx::query("UPDATE hulunote_navs SET parid = $1, updated_at = NOW() WHERE id = $2")
                    .bind(parid)
                    .bind(nav_uuid)
                    .execute(state.pool.as_ref())
                    .await?;
            }

            if let Some(order) = req.order {
                sqlx::query("UPDATE hulunote_navs SET same_deep_order = $1, updated_at = NOW() WHERE id = $2")
                    .bind(order)
                    .bind(nav_uuid)
                    .execute(state.pool.as_ref())
                    .await?;
            }

            if let Some(is_delete) = req.is_delete {
                sqlx::query("UPDATE hulunote_navs SET is_delete = $1, updated_at = NOW() WHERE id = $2")
                    .bind(is_delete)
                    .bind(nav_uuid)
                    .execute(state.pool.as_ref())
                    .await?;
            }

            if let Some(is_display) = req.is_display {
                sqlx::query("UPDATE hulunote_navs SET is_display = $1, updated_at = NOW() WHERE id = $2")
                    .bind(is_display)
                    .bind(nav_uuid)
                    .execute(state.pool.as_ref())
                    .await?;
            }

            if let Some(properties) = &req.properties {
                sqlx::query("UPDATE hulunote_navs SET properties = $1, updated_at = NOW() WHERE id = $2")
                    .bind(properties)
                    .bind(nav_uuid)
                    .execute(state.pool.as_ref())
                    .await?;
            }

            return Ok(Json(json!({
                "success": true,
                "id": nav_id,
                "backend-ts": backend_ts
            })));
        }
    }

    // Create new nav
    let nav_id = req.id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::new_v4);

    // parid, note_id, database_id are VARCHAR columns - use String
    let parid = req.parid.as_deref().unwrap_or(ROOT_NAV_ID);
    let content = req.content.as_deref().unwrap_or("");
    let order = req.order.unwrap_or(0.0);
    let properties = req.properties.as_deref().unwrap_or("");

    let nav: HulunoteNav = sqlx::query_as(
        r#"
        INSERT INTO hulunote_navs (id, parid, same_deep_order, content, account_id, note_id, database_id, properties)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, parid, same_deep_order, content, account_id, note_id, database_id,
                  is_display, is_public, is_delete, properties, extra_id, created_at, updated_at
        "#,
    )
    .bind(nav_id)           // id is UUID
    .bind(parid)            // parid is VARCHAR
    .bind(order)
    .bind(content)
    .bind(account_id)
    .bind(&req.note_id)     // note_id is VARCHAR
    .bind(&database_id)     // database_id is VARCHAR
    .bind(properties)
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok(Json(json!({
        "success": true,
        "id": nav.id.to_string(),
        "nav": NavInfo::from(nav),
        "backend-ts": backend_ts
    })))
}

/// Get navs for a note
pub async fn get_note_navs(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<GetNavsRequest>,
) -> Result<Json<Value>> {
    // note.id is UUID, so parse it
    let note_uuid = Uuid::parse_str(&req.note_id)
        .map_err(|_| AppError::BadRequest("Invalid note ID format".to_string()))?;

    // Check note access (simplified - just check if note exists)
    let note_exists: Option<(i64,)> = sqlx::query_as(
        "SELECT account_id FROM hulunote_notes WHERE id = $1"
    )
    .bind(note_uuid)
    .fetch_optional(state.pool.as_ref())
    .await?;

    if note_exists.is_none() {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    // navs.note_id is VARCHAR, so use String for the query
    let navs: Vec<HulunoteNav> = sqlx::query_as(
        r#"
        SELECT id, parid, same_deep_order, content, account_id, note_id, database_id,
               is_display, is_public, is_delete, properties, extra_id, created_at, updated_at
        FROM hulunote_navs
        WHERE note_id = $1 AND is_delete = false
        ORDER BY same_deep_order ASC
        "#,
    )
    .bind(&req.note_id)  // note_id column is VARCHAR
    .fetch_all(state.pool.as_ref())
    .await?;

    let nav_list: Vec<NavInfo> = navs.into_iter().map(NavInfo::from).collect();

    Ok(Json(json!({
        "nav-list": nav_list
    })))
}

/// Get all navs in a database by page
pub async fn get_all_navs_by_page(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<GetAllNavsByPageRequest>,
) -> Result<Json<Value>> {
    let database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        req.database_id.as_deref(),
        req.database_name.as_deref().or(req.database.as_deref()),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    // Convert to String for VARCHAR column
    let database_id_str = database_id.to_string();

    let page = req.page.unwrap_or(1).max(1);
    let size = req.size.unwrap_or(1000).min(5000);
    let offset = (page - 1) * size;
    let backend_ts = req.backend_ts.unwrap_or(0);

    // Get total count (database_id is VARCHAR)
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM hulunote_navs
        WHERE database_id = $1
        AND EXTRACT(EPOCH FROM updated_at) * 1000 > $2
        "#
    )
    .bind(&database_id_str)
    .bind(backend_ts as f64)
    .fetch_one(state.pool.as_ref())
    .await?;

    let all_pages = (count.0 as f64 / size as f64).ceil() as i64;

    // Get navs (database_id is VARCHAR)
    let navs: Vec<HulunoteNav> = sqlx::query_as(
        r#"
        SELECT id, parid, same_deep_order, content, account_id, note_id, database_id,
               is_display, is_public, is_delete, properties, extra_id, created_at, updated_at
        FROM hulunote_navs
        WHERE database_id = $1
        AND EXTRACT(EPOCH FROM updated_at) * 1000 > $2
        ORDER BY updated_at ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&database_id_str)
    .bind(backend_ts as f64)
    .bind(size)
    .bind(offset)
    .fetch_all(state.pool.as_ref())
    .await?;

    let nav_list: Vec<NavInfo> = navs.into_iter().map(NavInfo::from).collect();
    let new_backend_ts = Utc::now().timestamp_millis();

    Ok(Json(json!({
        "nav-list": nav_list,
        "all-pages": all_pages,
        "backend-ts": new_backend_ts
    })))
}

/// Get all navs in a database (no pagination)
pub async fn get_all_navs(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<GetAllNavsByPageRequest>,
) -> Result<Json<Value>> {
    let database_id = get_database_id(
        state.pool.as_ref(),
        account_id,
        req.database_id.as_deref(),
        req.database_name.as_deref().or(req.database.as_deref()),
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Database not found".to_string()))?;

    // Convert to String for VARCHAR column
    let database_id_str = database_id.to_string();
    let backend_ts = req.backend_ts.unwrap_or(0);

    let navs: Vec<HulunoteNav> = sqlx::query_as(
        r#"
        SELECT id, parid, same_deep_order, content, account_id, note_id, database_id,
               is_display, is_public, is_delete, properties, extra_id, created_at, updated_at
        FROM hulunote_navs
        WHERE database_id = $1
        AND EXTRACT(EPOCH FROM updated_at) * 1000 > $2
        ORDER BY updated_at ASC
        "#,
    )
    .bind(&database_id_str)
    .bind(backend_ts as f64)
    .fetch_all(state.pool.as_ref())
    .await?;

    let nav_list: Vec<NavInfo> = navs.into_iter().map(NavInfo::from).collect();
    let new_backend_ts = Utc::now().timestamp_millis();

    Ok(Json(json!({
        "nav-list": nav_list,
        "backend-ts": new_backend_ts
    })))
}
