mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;

use axum::Router;
use axum::http::{HeaderName, HeaderValue, Method};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "hulunote_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database pool
    let pool = db::init_pool().await?;

    // Build application state
    let app_state = handlers::AppState::new(pool);

    // CORS configuration - 允许开发环境跨域
    let cors = CorsLayer::new()
        .allow_origin([
            "tauri://localhost".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:8803".parse::<HeaderValue>().unwrap(),
            "http://localhost:8803".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:6689".parse::<HeaderValue>().unwrap(),
            "http://localhost:6689".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-functor-api-token"),
            HeaderName::from_static("authorization"),
        ]);

    // Build the router
    let app = Router::new()
        .merge(routes::create_routes())
        .nest_service("/", ServeDir::new("resources/public"))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Run server
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "6689".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Hulunote server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
