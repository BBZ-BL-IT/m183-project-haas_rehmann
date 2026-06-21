use axum::http::{HeaderValue, Method, header};
use axum_oidc_client::{
    auth::{AuthenticationLayer, LogoutHandler},
    cache::{TwoTierAuthCache, config::TwoTierCacheConfig},
    logout::{handle_default_logout::DefaultLogoutHandler, handle_oidc_logout::OidcLogoutHandler},
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

    // Apply pending migrations on startup 
    migrate!("./migrations").run(&pool).await?;
    info!("Database migrations applied");

    let auth_cache = Arc::new(TwoTierAuthCache::new(None, TwoTierCacheConfig::default())?);

    // RP-initiated logout: if the IdP's end-session endpoint is configured, also
    // tear down the *Kanidm* SSO session on logout (sends the browser there with
    // an id_token_hint). Without this, the local cookie is cleared but Kanidm's
    // session survives, so the next /auth silently logs the previous user back in.
    let logout_handler: Arc<dyn LogoutHandler> = match std::env::var("OIDC_END_SESSION_ENDPOINT") {
        Ok(endpoint) if !endpoint.trim().is_empty() => {
            info!("RP-initiated (OIDC) logout enabled via {}", endpoint.trim());
            Arc::new(OidcLogoutHandler::new(endpoint.trim()))
        }
        _ => {
            warn!(
                "OIDC_END_SESSION_ENDPOINT not set — using local-only logout; the IdP session \
                 persists and the next login may skip the password prompt"
            );
            Arc::new(DefaultLogoutHandler)
        }
    };

    let app_state = AppState::new(pool, crate::config::LoanConfig::from_env());

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
