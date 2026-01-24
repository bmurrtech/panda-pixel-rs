pub mod config;
pub mod routes;

use axum::{
    routing::post,
    extract::DefaultBodyLimit,
    Router,
};
use http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use crate::config::Config;
use crate::routes::{compress_batch, compress_image};

pub fn create_app(config: &Config) -> Router {
    // Build CORS layer
    let cors = if config.cors_allowed_origins.contains(&"*".to_string()) || config.app_env == "development" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<HeaderValue> = config.cors_allowed_origins
            .iter()
            .map(|s| s.parse::<HeaderValue>().unwrap())
            .collect();
        
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Build application routes
    Router::new()
        .route("/api/compress", post(compress_image))
        .route("/api/compress/batch", post(compress_batch))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB limit
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
