use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::handlers::{self, ws, AppState};
use crate::middleware::auth_middleware;

pub fn create_routes() -> Router<AppState> {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/login/web-login", post(handlers::web_login))
        .route("/login/web-signup", post(handlers::web_signup))
        .route("/login/send-ack-msg", post(handlers::send_ack_msg));

    // WebSocket route (auth via query param token)
    let ws_routes = Router::new()
        .route("/ws", get(ws::ws_handler));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // User profile routes
        .route("/user/profile", get(handlers::get_profile))
        .route("/user/update-profile", post(handlers::update_profile))
        .route("/user/upload-avatar", post(handlers::upload_avatar))
        .route("/user/generate-token", post(handlers::generate_user_token))
        // Database routes
        .route("/hulunote/new-database", post(handlers::create_database))
        .route("/hulunote/create-database", post(handlers::create_database))
        .route("/hulunote/get-database-list", post(handlers::get_database_list))
        .route("/hulunote/update-database", post(handlers::update_database))
        .route("/hulunote/delete-database", post(handlers::delete_database))
        // Note routes
        .route("/hulunote/new-note", post(handlers::create_note))
        .route("/hulunote/get-note-list", post(handlers::get_note_list))
        .route("/hulunote/get-all-note-list", post(handlers::get_all_note_list))
        .route("/hulunote/update-hulunote-note", post(handlers::update_note))
        .route("/hulunote/get-shortcuts-note-list", post(handlers::get_shortcuts_note_list))
        // Nav routes
        .route("/hulunote/create-or-update-nav", post(handlers::create_or_update_nav))
        .route("/hulunote/new-hulunote-navs-uuid-v2", post(handlers::create_or_update_nav))
        .route("/hulunote/get-note-navs", post(handlers::get_note_navs))
        .route("/hulunote/get-nav-list-by-id", post(handlers::get_note_navs))
        .route("/hulunote/get-all-nav-by-page", post(handlers::get_all_navs_by_page))
        .route("/hulunote/get-all-navs", post(handlers::get_all_navs))
        // Import routes
        .route("/hulunote/import-notes", post(handlers::import_notes))
        .route_layer(middleware::from_fn(auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(ws_routes)
        .merge(protected_routes)
}
