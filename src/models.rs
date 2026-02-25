use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ========== Account Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub mail: Option<String>,
    pub invitation_code: Option<String>,
    pub cell_number: Option<String>,
    pub oauth_key: Option<String>,
    pub need_update_password: Option<bool>,
    pub is_new_user: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub registration_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: Option<String>,
    pub email: String,
    pub password: String,
    pub registration_code: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub hulunote: AccountInfo,
    pub database: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccountInfo {
    #[serde(rename = "accounts/id")]
    pub id: i64,
    #[serde(rename = "accounts/username")]
    pub username: String,
    #[serde(rename = "accounts/nickname", skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(rename = "accounts/mail", skip_serializing_if = "Option::is_none")]
    pub mail: Option<String>,
    #[serde(rename = "accounts/invitation-code", skip_serializing_if = "Option::is_none")]
    pub invitation_code: Option<String>,
    #[serde(rename = "accounts/is-new-user")]
    pub is_new_user: bool,
    #[serde(rename = "accounts/created-at")]
    pub created_at: String,
    #[serde(rename = "accounts/updated-at")]
    pub updated_at: String,
}

impl From<Account> for AccountInfo {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            username: account.username,
            nickname: account.nickname,
            mail: account.mail,
            invitation_code: account.invitation_code,
            is_new_user: account.is_new_user,
            created_at: account.created_at.to_rfc3339(),
            updated_at: account.updated_at.to_rfc3339(),
        }
    }
}

// ========== Database Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HulunoteDatabase {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_delete: bool,
    pub is_public: bool,
    pub is_offline: bool,
    pub is_default: bool,
    pub account_id: i64,
    pub setting: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseInfo {
    #[serde(rename = "hulunote-databases/id")]
    pub id: String,
    #[serde(rename = "hulunote-databases/name")]
    pub name: String,
    #[serde(rename = "hulunote-databases/description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "hulunote-databases/is-delete")]
    pub is_delete: bool,
    #[serde(rename = "hulunote-databases/is-public")]
    pub is_public: bool,
    #[serde(rename = "hulunote-databases/is-default")]
    pub is_default: bool,
    #[serde(rename = "hulunote-databases/account-id")]
    pub account_id: i64,
    #[serde(rename = "hulunote-databases/created-at")]
    pub created_at: String,
    #[serde(rename = "hulunote-databases/updated-at")]
    pub updated_at: String,
}

impl From<HulunoteDatabase> for DatabaseInfo {
    fn from(db: HulunoteDatabase) -> Self {
        Self {
            id: db.id.to_string(),
            name: db.name,
            description: db.description,
            is_delete: db.is_delete,
            is_public: db.is_public,
            is_default: db.is_default,
            account_id: db.account_id,
            created_at: db.created_at.to_rfc3339(),
            updated_at: db.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDatabaseRequest {
    #[serde(rename = "database-name")]
    pub database_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDatabaseRequest {
    #[serde(rename = "database-id")]
    pub database_id: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "is-public")]
    pub is_public: Option<bool>,
    #[serde(rename = "is-default")]
    pub is_default: Option<bool>,
    #[serde(rename = "is-delete")]
    pub is_delete: Option<bool>,
    #[serde(rename = "db-name")]
    pub db_name: Option<String>,
}

// ========== Note Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HulunoteNote {
    pub id: Uuid,
    pub title: String,
    pub database_id: String,
    pub root_nav_id: String,
    pub is_delete: bool,
    pub is_public: bool,
    pub is_shortcut: bool,
    pub account_id: i64,
    pub pv: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NoteInfo {
    #[serde(rename = "hulunote-notes/id")]
    pub id: String,
    #[serde(rename = "hulunote-notes/title")]
    pub title: String,
    #[serde(rename = "hulunote-notes/database-id")]
    pub database_id: String,
    #[serde(rename = "hulunote-notes/root-nav-id")]
    pub root_nav_id: String,
    #[serde(rename = "hulunote-notes/is-delete")]
    pub is_delete: bool,
    #[serde(rename = "hulunote-notes/is-public")]
    pub is_public: bool,
    #[serde(rename = "hulunote-notes/is-shortcut")]
    pub is_shortcut: bool,
    #[serde(rename = "hulunote-notes/account-id")]
    pub account_id: i64,
    #[serde(rename = "hulunote-notes/pv")]
    pub pv: i64,
    #[serde(rename = "hulunote-notes/created-at")]
    pub created_at: String,
    #[serde(rename = "hulunote-notes/updated-at")]
    pub updated_at: String,
}

impl From<HulunoteNote> for NoteInfo {
    fn from(note: HulunoteNote) -> Self {
        Self {
            id: note.id.to_string(),
            title: note.title,
            database_id: note.database_id,
            root_nav_id: note.root_nav_id,
            is_delete: note.is_delete,
            is_public: note.is_public,
            is_shortcut: note.is_shortcut,
            account_id: note.account_id,
            pv: note.pv,
            created_at: note.created_at.to_rfc3339(),
            updated_at: note.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    #[serde(rename = "database-id")]
    pub database_id: Option<String>,
    pub database: Option<String>,
    #[serde(rename = "database-name")]
    pub database_name: Option<String>,
    pub title: String,

    /// Optional client-provided note id (UUID). If omitted, backend generates one.
    #[serde(rename = "note-id", alias = "note_id")]
    pub note_id: Option<String>,

    /// Optional client-provided root nav id (UUID). If omitted, backend generates one.
    #[serde(rename = "root-nav-id", alias = "root_nav_id")]
    pub root_nav_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetNoteListRequest {
    #[serde(rename = "database-id")]
    pub database_id: Option<String>,
    pub database: Option<String>,
    #[serde(rename = "database-name")]
    pub database_name: Option<String>,
    pub page: Option<i64>,
    pub size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    #[serde(rename = "note-id")]
    pub note_id: String,
    pub title: Option<String>,
    #[serde(rename = "is-delete")]
    pub is_delete: Option<bool>,
    #[serde(rename = "is-public")]
    pub is_public: Option<bool>,
    #[serde(rename = "is-shortcut")]
    pub is_shortcut: Option<bool>,
}

// ========== Nav Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HulunoteNav {
    pub id: Uuid,
    pub parid: String,
    pub same_deep_order: f32,
    pub content: String,
    pub account_id: i64,
    pub note_id: String,
    pub database_id: String,
    pub is_display: bool,
    pub is_public: bool,
    pub is_delete: bool,
    pub properties: String,
    pub extra_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NavInfo {
    pub id: String,
    pub parid: String,
    #[serde(rename = "same-deep-order")]
    pub same_deep_order: f32,
    pub content: String,
    #[serde(rename = "account-id")]
    pub account_id: i64,
    #[serde(rename = "last-account-id")]
    pub last_account_id: i64,
    #[serde(rename = "note-id")]
    pub note_id: String,
    #[serde(rename = "hulunote-note")]
    pub hulunote_note: String,
    #[serde(rename = "database-id")]
    pub database_id: String,
    #[serde(rename = "is-display")]
    pub is_display: bool,
    #[serde(rename = "is-public")]
    pub is_public: bool,
    #[serde(rename = "is-delete")]
    pub is_delete: bool,
    pub properties: String,
    #[serde(rename = "created-at")]
    pub created_at: String,
    #[serde(rename = "updated-at")]
    pub updated_at: String,
}

impl From<HulunoteNav> for NavInfo {
    fn from(nav: HulunoteNav) -> Self {
        Self {
            id: nav.id.to_string(),
            parid: nav.parid.clone(),
            same_deep_order: nav.same_deep_order,
            content: nav.content,
            account_id: nav.account_id,
            last_account_id: nav.account_id, // Use account_id as last_account_id
            note_id: nav.note_id.clone(),
            hulunote_note: nav.note_id,
            database_id: nav.database_id,
            is_display: nav.is_display,
            is_public: nav.is_public,
            is_delete: nav.is_delete,
            properties: nav.properties,
            created_at: nav.created_at.to_rfc3339(),
            updated_at: nav.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrUpdateNavRequest {
    #[serde(rename = "database-id")]
    pub database_id: Option<String>,
    pub database: Option<String>,
    #[serde(rename = "database-name")]
    pub database_name: Option<String>,
    #[serde(rename = "note-id")]
    pub note_id: String,
    pub id: Option<String>,
    pub parid: Option<String>,
    pub content: Option<String>,
    #[serde(rename = "is-delete")]
    pub is_delete: Option<bool>,
    #[serde(rename = "is-display")]
    pub is_display: Option<bool>,
    pub properties: Option<String>,
    pub order: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct GetNavsRequest {
    #[serde(rename = "note-id")]
    pub note_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GetAllNavsByPageRequest {
    #[serde(rename = "database-id")]
    pub database_id: Option<String>,
    pub database: Option<String>,
    #[serde(rename = "database-name")]
    pub database_name: Option<String>,
    #[serde(rename = "backend-ts")]
    pub backend_ts: Option<i64>,
    pub page: Option<i64>,
    pub size: Option<i64>,
}

// ========== Registration Code Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegistrationCode {
    pub id: i64,
    pub code: String,
    pub validity_days: i32,
    pub is_used: bool,
    pub used_by_account_id: Option<i64>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ========== Import Models ==========

#[derive(Debug, Deserialize)]
pub struct ImportNoteJson {
    pub note: ImportNoteData,
    pub navs: Vec<ImportNavData>,
}

#[derive(Debug, Deserialize)]
pub struct ImportNoteData {
    #[serde(rename = "hulunote-notes/id")]
    pub id: String,
    #[serde(rename = "hulunote-notes/title")]
    pub title: String,
    #[serde(rename = "hulunote-navs/root-nav-id")]
    pub root_nav_id: String,
    #[serde(rename = "hulunote-notes/is-delete")]
    pub is_delete: Option<bool>,
    #[serde(rename = "hulunote-notes/is-public")]
    pub is_public: Option<bool>,
    #[serde(rename = "hulunote-notes/is-shortcut")]
    pub is_shortcut: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ImportNavData {
    pub id: String,
    pub parid: String,
    pub content: String,
    #[serde(rename = "same-deep-order")]
    pub same_deep_order: f64,
    #[allow(dead_code)]
    #[serde(rename = "hulunote-note")]
    pub hulunote_note: String,
    #[serde(rename = "is-display")]
    pub is_display: Option<bool>,
    #[serde(rename = "is-delete")]
    pub is_delete: Option<bool>,
}

// ========== JWT Claims ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: i64,
    pub role: String,
    pub exp: i64,
}
