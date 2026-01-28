use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::config::Config;
use crate::error::{AppError, Result};
use crate::models::Claims;

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response> {
    let token = request
        .headers()
        .get("X-FUNCTOR-API-TOKEN")
        .or_else(|| request.headers().get(header::AUTHORIZATION))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string());

    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => return Err(AppError::Auth("Missing or empty token".to_string())),
    };

    let config = Config::from_env();
    let claims = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?
    .claims;

    // Add account_id to request extensions
    request.extensions_mut().insert(claims.id);

    Ok(next.run(request).await)
}

pub fn generate_token(account_id: i64) -> Result<String> {
    let config = Config::from_env();
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(config.jwt_expiry_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        id: account_id,
        role: "hulunote".to_string(),
        exp: expiration,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(AppError::from)
}
