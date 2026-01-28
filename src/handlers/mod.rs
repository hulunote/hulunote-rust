mod auth;
mod database;
mod note;
mod nav;

pub use auth::*;
pub use database::*;
pub use note::*;
pub use nav::*;

use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}
