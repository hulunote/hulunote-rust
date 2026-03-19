use axum::extract::{Extension, Multipart, State};
use axum::Json;
use serde_json::{json, Value};

use crate::error::{AppError, Result};
use crate::middleware::generate_token_with_hours;
use crate::models::*;

use super::AppState;

/// Get current user profile
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
) -> Result<Json<Value>> {
    let account: Account = sqlx::query_as(
        r#"
        SELECT id, username, nickname, password, mail, avatar, introduction,
               invitation_code, cell_number, oauth_key, need_update_password,
               is_new_user, expires_at, registration_code, created_at, updated_at
        FROM accounts
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok(Json(json!({
        "profile": AccountInfo::from(account)
    })))
}

/// Update user profile (nickname, introduction)
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<Value>> {
    let account: Account = sqlx::query_as(
        r#"
        UPDATE accounts
        SET nickname = COALESCE($2, nickname),
            introduction = COALESCE($3, introduction),
            updated_at = now()
        WHERE id = $1
        RETURNING id, username, nickname, password, mail, avatar, introduction,
                  invitation_code, cell_number, oauth_key, need_update_password,
                  is_new_user, expires_at, registration_code, created_at, updated_at
        "#,
    )
    .bind(account_id)
    .bind(&req.nickname)
    .bind(&req.introduction)
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok(Json(json!({
        "profile": AccountInfo::from(account)
    })))
}

/// Upload user avatar
pub async fn upload_avatar(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
    mut multipart: Multipart,
) -> Result<Json<Value>> {
    let mut avatar_path: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "avatar" {
            let filename = field
                .file_name()
                .unwrap_or("avatar.png")
                .to_string();

            // Determine extension
            let ext = filename
                .rsplit('.')
                .next()
                .unwrap_or("png")
                .to_lowercase();

            let allowed_exts = ["png", "jpg", "jpeg", "gif", "webp"];
            if !allowed_exts.contains(&ext.as_str()) {
                return Err(AppError::BadRequest(
                    "Only png, jpg, jpeg, gif, webp files are allowed".to_string(),
                ));
            }

            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?;

            if data.len() > 5 * 1024 * 1024 {
                return Err(AppError::BadRequest(
                    "Avatar file size must be less than 5MB".to_string(),
                ));
            }

            // Save to uploads directory
            let upload_dir = std::path::Path::new("resources/public/uploads/avatars");
            tokio::fs::create_dir_all(upload_dir)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to create upload dir: {}", e)))?;

            let save_filename = format!("{}.{}", account_id, ext);
            let save_path = upload_dir.join(&save_filename);

            tokio::fs::write(&save_path, &data)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;

            avatar_path = Some(format!("/uploads/avatars/{}", save_filename));
        }
    }

    let avatar_url = avatar_path
        .ok_or_else(|| AppError::BadRequest("No avatar file provided".to_string()))?;

    // Update database
    let account: Account = sqlx::query_as(
        r#"
        UPDATE accounts
        SET avatar = $2, updated_at = now()
        WHERE id = $1
        RETURNING id, username, nickname, password, mail, avatar, introduction,
                  invitation_code, cell_number, oauth_key, need_update_password,
                  is_new_user, expires_at, registration_code, created_at, updated_at
        "#,
    )
    .bind(account_id)
    .bind(&avatar_url)
    .fetch_one(state.pool.as_ref())
    .await?;

    Ok(Json(json!({
        "profile": AccountInfo::from(account),
        "avatar_url": avatar_url
    })))
}

/// Generate a new JWT token with 3-month validity (2160 hours)
pub async fn generate_user_token(
    State(state): State<AppState>,
    Extension(account_id): Extension<i64>,
) -> Result<Json<Value>> {
    // Verify account exists and is not expired
    let account: Account = sqlx::query_as(
        r#"
        SELECT id, username, nickname, password, mail, avatar, introduction,
               invitation_code, cell_number, oauth_key, need_update_password,
               is_new_user, expires_at, registration_code, created_at, updated_at
        FROM accounts
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(state.pool.as_ref())
    .await?;

    if let Some(expires_at) = account.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(AppError::Auth("Account has expired".to_string()));
        }
    }

    // Generate token with 3-month validity (90 days = 2160 hours)
    let token = generate_token_with_hours(account.id, 2160)?;

    // Calculate expiry date for display
    let expiry_date = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(2160))
        .expect("valid timestamp");

    Ok(Json(json!({
        "token": token,
        "expires_at": expiry_date.to_rfc3339(),
        "validity_days": 90,
        "hulunote": AccountInfo::from(account)
    })))
}
