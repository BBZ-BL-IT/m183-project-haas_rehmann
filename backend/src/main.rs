use tracing::info;

mod auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let oidc_config = auth::get_oidc_config()?;

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
