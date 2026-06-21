mod app;
mod auth;
mod config;
mod db;
mod error;
mod game;
mod handlers;
mod identity;
mod kanidm;
mod models;
mod routes;
mod state;
mod validate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    app::run().await?;

    Ok(())
}
