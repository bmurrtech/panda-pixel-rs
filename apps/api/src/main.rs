mod config;
mod routes;

use axum::{
    routing::post,
    Router,
};
use config::Config;
use routes::{compress_batch, compress_image};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let config = Config::from_env()?;
    config.validate()?;
    
    tracing_subscriber::fmt()
        .with_env_filter(&config.rust_log)
        .init();

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application routes
    let app = Router::new()
        .route("/api/compress", post(compress_image))
        .route("/api/compress/batch", post(compress_batch))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting API server on {}", addr);
    info!("Environment: {}", config.app_env);
    info!("CORS allowed origins: {:?}", config.cors_allowed_origins);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
