mod auth;
mod database;
mod import;
mod note;
mod nav;
mod user;
pub mod ws;

pub use auth::*;
pub use database::*;
pub use import::*;
pub use note::*;
pub use nav::*;
pub use user::*;

use sqlx::PgPool;
use std::sync::Arc;

use ws::WsBroadcaster;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
    pub ws_broadcaster: WsBroadcaster,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
            ws_broadcaster: WsBroadcaster::new(),
        }
    }
}
