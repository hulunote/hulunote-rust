use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::middleware::generate_token;
use crate::models::*;

use super::AppState;

/// Web login handler
pub async fn web_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>> {
    let identifier = req.username.or(req.email);
    
    let identifier = match identifier {
        Some(id) if !id.is_empty() => id,
        _ => return Err(AppError::BadRequest("Username or email required".to_string())),
    };

    // Find account by username or email
    let account: Option<Account> = sqlx::query_as(
        r#"
        SELECT id, username, nickname, password, mail, invitation_code,
               cell_number, oauth_key, need_update_password, is_new_user,
               expires_at, registration_code, created_at, updated_at
        FROM accounts
        WHERE username = $1 OR mail = $1
        "#,
    )
    .bind(&identifier)
    .fetch_optional(state.pool.as_ref())
    .await?;

    let account = match account {
        Some(a) => a,
        None => return Err(AppError::Auth("User not found".to_string())),
    };

    // Check if account has expired
    if let Some(expires_at) = account.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(AppError::Auth("Account has expired".to_string()));
        }
    }

    // Verify password
    let password_hash = account.password.as_ref()
        .ok_or_else(|| AppError::Auth("Password not set".to_string()))?;

    if !verify(&req.password, password_hash)? {
        return Err(AppError::Auth("Invalid password".to_string()));
    }

    // Generate token
    let token = generate_token(account.id)?;

    Ok(Json(json!({
        "token": token,
        "hulunote": AccountInfo::from(account),
        "region": null
    })))
}

/// Web signup handler
pub async fn web_signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<Value>> {
    use chrono::Duration;

    // Validate email
    if !req.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    // Verify registration code
    let reg_code: Option<crate::models::RegistrationCode> = sqlx::query_as(
        r#"
        SELECT id, code, validity_days, is_used, used_by_account_id, used_at, created_at, updated_at
        FROM registration_codes
        WHERE code = $1
        "#,
    )
    .bind(&req.registration_code)
    .fetch_optional(state.pool.as_ref())
    .await?;

    let reg_code = match reg_code {
        Some(code) => code,
        None => return Err(AppError::BadRequest("Invalid registration code".to_string())),
    };

    if reg_code.is_used {
        return Err(AppError::BadRequest("Registration code has already been used".to_string()));
    }

    // Check if user exists
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM accounts WHERE username = $1 OR mail = $2"
    )
    .bind(&req.email)
    .bind(&req.email)
    .fetch_optional(state.pool.as_ref())
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("User already exists".to_string()));
    }

    // Hash password
    let password_hash = hash(&req.password, DEFAULT_COST)?;

    // Generate invitation code
    let invitation_code = Uuid::new_v4().to_string()[..8].to_string();
    let cell_number = Uuid::new_v4().to_string();
    let username = req.username.unwrap_or_else(|| req.email.clone());

    // Calculate expiration date
    let expires_at = chrono::Utc::now() + Duration::days(reg_code.validity_days as i64);

    // Generate random number BEFORE the await (thread_rng is not Send)
    let random_suffix: u32 = rand::thread_rng().gen_range(0..10000);
    let db_name = format!("{}-{}", username, random_suffix);
    let db_id = Uuid::new_v4();

    // Create account
    let account: Account = sqlx::query_as(
        r#"
        INSERT INTO accounts (username, nickname, password, mail, invitation_code, cell_number,
                              is_new_user, expires_at, registration_code)
        VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8)
        RETURNING id, username, nickname, password, mail, invitation_code,
                  cell_number, oauth_key, need_update_password, is_new_user,
                  expires_at, registration_code, created_at, updated_at
        "#,
    )
    .bind(&username)
    .bind(&username)
    .bind(&password_hash)
    .bind(&req.email)
    .bind(&invitation_code)
    .bind(&cell_number)
    .bind(expires_at)
    .bind(&req.registration_code)
    .fetch_one(state.pool.as_ref())
    .await?;

    // Mark registration code as used
    sqlx::query(
        r#"
        UPDATE registration_codes
        SET is_used = true, used_by_account_id = $1, used_at = now()
        WHERE id = $2
        "#,
    )
    .bind(account.id)
    .bind(reg_code.id)
    .execute(state.pool.as_ref())
    .await?;

    // Create default database for user
    sqlx::query(
        r#"
        INSERT INTO hulunote_databases (id, name, account_id, is_default)
        VALUES ($1, $2, $3, true)
        "#,
    )
    .bind(db_id)
    .bind(&db_name)
    .bind(account.id)
    .execute(state.pool.as_ref())
    .await?;

    // Generate token
    let token = generate_token(account.id)?;

    Ok(Json(json!({
        "token": token,
        "hulunote": AccountInfo::from(account),
        "database": db_name,
        "region": null
    })))
}

/// Send email verification code (placeholder)
pub async fn send_ack_msg(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<Value>> {
    let email = req.get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Email required".to_string()))?;

    if !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    // TODO: Implement actual email sending
    // For now, just return success
    tracing::info!("Would send verification email to: {}", email);

    Ok(Json(json!({
        "success": true
    })))
}
