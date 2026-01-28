use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::handlers::{self, AppState};
use crate::middleware::auth_middleware;

pub fn create_routes() -> Router<AppState> {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/login/web-login", post(handlers::web_login))
        .route("/login/web-signup", post(handlers::web_signup))
        .route("/login/send-ack-msg", post(handlers::send_ack_msg));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Database routes
        .route("/hulunote/new-database", post(handlers::create_database))
        .route("/hulunote/get-database-list", post(handlers::get_database_list))
        .route("/hulunote/update-database", post(handlers::update_database))
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
        .route_layer(middleware::from_fn(auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}
