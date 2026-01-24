use api::{config::Config, create_app};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let config = Config::from_env()?;
    config.validate()?;
    
    tracing_subscriber::fmt()
        .with_env_filter(&config.rust_log)
        .init();

    // Build application routes
    let app = create_app(&config);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting API server on {}", addr);
    info!("Environment: {}", config.app_env);
    info!("CORS allowed origins: {:?}", config.cors_allowed_origins);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
