use axum_oidc_client::{
    auth::AuthenticationLayer,
    cache::{TwoTierAuthCache, config::TwoTierCacheConfig},
    logout::handle_default_logout::DefaultLogoutHandler,
};
use sqlx::{migrate, postgres::PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::auth;
use crate::routes;
use crate::state::AppState;

pub async fn run() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let oidc_config = auth::get_oidc_config()
        .map_err(|e| anyhow::anyhow!("OIDC Initialization failed: {}", e))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    migrate!("./migrations");

    let auth_cache = Arc::new(TwoTierAuthCache::new(None, TwoTierCacheConfig::default())?);
    let logout_handler = Arc::new(DefaultLogoutHandler);

    let app_state = AppState::new(pool);

    // 5. Build Router & Attach Layers
    // Note: routes::create_router returns your base Axum Router
    let app = routes::create_router(app_state).layer(AuthenticationLayer::new(
        Arc::new(oidc_config),
        auth_cache,
        logout_handler,
    ));

    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid number between 0 and 65535");

    let server_addr = format!("0.0.0.0:{}", port);
    info!("Starting server...");

    let listener = tokio::net::TcpListener::bind(&server_addr).await?;
    info!("Server listening on: {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
