use axum::http::{HeaderValue, Method, header};
use axum_oidc_client::{
    auth::AuthenticationLayer,
    cache::{TwoTierAuthCache, config::TwoTierCacheConfig},
    logout::handle_default_logout::DefaultLogoutHandler,
};
use sqlx::{migrate, postgres::PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

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

    // Apply pending migrations on startup (embedded into the binary at compile
    // time, so the Docker image doesn't need the SQL files at runtime).
    migrate!("./migrations").run(&pool).await?;
    info!("Database migrations applied");

    let auth_cache = Arc::new(TwoTierAuthCache::new(None, TwoTierCacheConfig::default())?);
    let logout_handler = Arc::new(DefaultLogoutHandler);

    let app_state = AppState::new(pool);

    // Base router → OIDC auth layer → (optional) CORS → request tracing.
    let mut app = routes::create_router(app_state).layer(AuthenticationLayer::new(
        Arc::new(oidc_config),
        auth_cache,
        logout_handler,
    ));

    if let Some(cors) = build_cors() {
        app = app.layer(cors);
    }

    let app = app.layer(TraceLayer::new_for_http());

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

/// Build a CORS layer from `CORS_ALLOWED_ORIGINS` (comma separated). Returns
/// `None` when unset, which is the right default for a same-origin deployment
/// (frontend served behind the same host, or reverse-proxied to the backend).
///
/// When the frontend runs on a *different* origin (e.g. the Vite dev server on
/// `http://localhost:5173`), set the variable so the browser is allowed to send
/// the session cookie with credentials.
fn build_cors() -> Option<CorsLayer> {
    let raw = std::env::var("CORS_ALLOWED_ORIGINS").ok()?;
    let origins: Vec<HeaderValue> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|o| match o.parse::<HeaderValue>() {
            Ok(v) => Some(v),
            Err(_) => {
                warn!("ignoring invalid CORS origin: {o}");
                None
            }
        })
        .collect();

    if origins.is_empty() {
        return None;
    }

    info!("CORS enabled for origins: {raw}");
    Some(
        CorsLayer::new()
            .allow_origin(origins)
            .allow_credentials(true)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([header::CONTENT_TYPE]),
    )
}
