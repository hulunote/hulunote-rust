use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::*;

use super::AppState;

/// Get database ID by various identifiers
pub async fn get_database_id(
    pool: &sqlx::PgPool,
    account_id: i64,
    database_id: Option<&str>,
    database_name: Option<&str>,
) -> Result<Option<Uuid>> {
    // Try database_id first
    if let Some(id) = database_id {
        if let Ok(uuid) = Uuid::parse_str(id) {
            return Ok(Some(uuid));
        }
    }

    // Try database_name
    if let Some(name) = database_name {
        let result: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM hulunote_databases WHERE name = $1 AND account_id = $2 AND is_delete = false"
        )
        .bind(name)
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
        
        return Ok(result.map(|r| r.0));
    }

    Ok(None)
}

/// Create a new database
pub async fn create_database(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<CreateDatabaseRequest>,
) -> Result<Json<Value>> {
    // Check existing database count
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM hulunote_databases WHERE account_id = $1 AND is_delete = false"
    )
    .bind(account_id)
    .fetch_one(state.pool.as_ref())
    .await?;

    if count.0 >= 5 {
        return Err(AppError::BadRequest("Maximum 5 databases allowed".to_string()));
    }

    // Check if database name already exists for this user
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM hulunote_databases WHERE name = $1 AND account_id = $2 AND is_delete = false"
    )
    .bind(&req.database_name)
    .bind(account_id)
    .fetch_optional(state.pool.as_ref())
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(format!("Database '{}' already exists", req.database_name)));
    }

    let db_id = Uuid::new_v4();

    let db: HulunoteDatabase = sqlx::query_as(
        r#"
        INSERT INTO hulunote_databases (id, name, description, account_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, description, is_delete, is_public, is_offline, is_default, 
                  account_id, setting, created_at, updated_at
        "#,
    )
    .bind(db_id)
    .bind(&req.database_name)
    .bind(&req.description)
    .bind(account_id)
    .fetch_one(state.pool.as_ref())
    .await?;

    // Return with "database" key to match frontend expectation
    Ok(Json(json!({
        "database": DatabaseInfo::from(db),
        "success": true
    })))
}

/// Delete database request
#[derive(Debug, Deserialize)]
pub struct DeleteDatabaseRequest {
    #[serde(rename = "database-id")]
    pub database_id: Option<String>,
    #[serde(rename = "database-name")]
    pub database_name: Option<String>,
}

/// Delete a database (soft delete)
pub async fn delete_database(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<DeleteDatabaseRequest>,
) -> Result<Json<Value>> {
    // Get database UUID from id or name
    let db_uuid = if let Some(ref id) = req.database_id {
        Uuid::parse_str(id)
            .map_err(|_| AppError::BadRequest("Invalid database ID".to_string()))?
    } else if let Some(ref name) = req.database_name {
        let result: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM hulunote_databases WHERE name = $1 AND account_id = $2 AND is_delete = false"
        )
        .bind(name)
        .bind(account_id)
        .fetch_optional(state.pool.as_ref())
        .await?;
        
        result
            .map(|r| r.0)
            .ok_or_else(|| AppError::NotFound("Database not found".to_string()))?
    } else {
        return Err(AppError::BadRequest("Database ID or name required".to_string()));
    };

    // Check ownership
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT account_id FROM hulunote_databases WHERE id = $1 AND is_delete = false"
    )
    .bind(db_uuid)
    .fetch_optional(state.pool.as_ref())
    .await?;

    match exists {
        Some((owner_id,)) if owner_id != account_id => {
            return Err(AppError::PermissionDenied("Cannot delete other's database".to_string()));
        }
        None => {
            return Err(AppError::NotFound("Database not found".to_string()));
        }
        _ => {}
    }

    // Soft delete the database
    sqlx::query(
        "UPDATE hulunote_databases SET is_delete = true, updated_at = NOW() WHERE id = $1"
    )
    .bind(db_uuid)
    .execute(state.pool.as_ref())
    .await?;

    // Optionally: soft delete all notes in this database
    sqlx::query(
        "UPDATE hulunote_notes SET is_delete = true, updated_at = NOW() WHERE database_id = $1"
    )
    .bind(db_uuid.to_string())
    .execute(state.pool.as_ref())
    .await?;

    // Optionally: soft delete all navs in this database
    sqlx::query(
        "UPDATE hulunote_navs SET is_delete = true, updated_at = NOW() WHERE database_id = $1"
    )
    .bind(db_uuid.to_string())
    .execute(state.pool.as_ref())
    .await?;

    Ok(Json(json!({
        "success": true,
        "message": "Database deleted successfully"
    })))
}

/// Get database list for current user
pub async fn get_database_list(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(_req): Json<Value>,
) -> Result<Json<Value>> {
    let databases: Vec<HulunoteDatabase> = sqlx::query_as(
        r#"
        SELECT id, name, description, is_delete, is_public, is_offline, is_default, 
               account_id, setting, created_at, updated_at
        FROM hulunote_databases 
        WHERE account_id = $1 AND is_delete = false
        ORDER BY created_at DESC
        "#,
    )
    .bind(account_id)
    .fetch_all(state.pool.as_ref())
    .await?;

    let database_list: Vec<DatabaseInfo> = databases.into_iter().map(DatabaseInfo::from).collect();

    // Get user settings (placeholder)
    let settings = json!({});

    Ok(Json(json!({
        "database-list": database_list,
        "settings": settings
    })))
}

/// Update database
pub async fn update_database(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<UpdateDatabaseRequest>,
) -> Result<Json<Value>> {
    let database_id = req.database_id.or(req.id)
        .ok_or_else(|| AppError::BadRequest("Database ID required".to_string()))?;
    
    let db_uuid = Uuid::parse_str(&database_id)
        .map_err(|_| AppError::BadRequest("Invalid database ID".to_string()))?;

    // Check ownership
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT account_id FROM hulunote_databases WHERE id = $1"
    )
    .bind(db_uuid)
    .fetch_optional(state.pool.as_ref())
    .await?;

    match exists {
        Some((owner_id,)) if owner_id != account_id => {
            return Err(AppError::PermissionDenied("Cannot update other's database".to_string()));
        }
        None => {
            return Err(AppError::NotFound("Database not found".to_string()));
        }
        _ => {}
    }

    // Build update query dynamically
    let mut updates = vec![];
    let mut bind_idx = 2;

    if req.is_public.is_some() {
        updates.push(format!("is_public = ${}", bind_idx));
        bind_idx += 1;
    }
    if req.is_default.is_some() {
        updates.push(format!("is_default = ${}", bind_idx));
        bind_idx += 1;
    }
    if req.is_delete.is_some() {
        updates.push(format!("is_delete = ${}", bind_idx));
        bind_idx += 1;
    }
    if req.db_name.is_some() {
        updates.push(format!("name = ${}", bind_idx));
    }

    if updates.is_empty() {
        return Ok(Json(json!({"success": true})));
    }

    updates.push("updated_at = NOW()".to_string());
    let query = format!(
        "UPDATE hulunote_databases SET {} WHERE id = $1",
        updates.join(", ")
    );

    let mut query_builder = sqlx::query(&query).bind(db_uuid);
    
    if let Some(v) = req.is_public {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = req.is_default {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = req.is_delete {
        query_builder = query_builder.bind(v);
    }
    if let Some(ref v) = req.db_name {
        query_builder = query_builder.bind(v);
    }

    query_builder.execute(state.pool.as_ref()).await?;

    Ok(Json(json!({"success": true})))
}
