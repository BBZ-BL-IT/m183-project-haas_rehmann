use axum_oidc_client::auth_builder::OAuthConfigurationBuilder;
use std::env;
use std::fs;

fn env_or_file(name: &str) -> Result<String, String> {
    if let Ok(value) = env::var(name) {
        return Ok(value);
    }
    if let Ok(path) = env::var(format!("{name}_FILE")) {
        return fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("failed to read {name}_FILE ({path}): {e}"));
    }
    Err(format!("{name} (or {name}_FILE) must be set"))
}

pub fn get_oidc_config()
-> Result<axum_oidc_client::auth::OAuthConfiguration, Box<dyn std::error::Error>> {
    let client_id = env::var("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID must be set");
    let client_secret = env_or_file("OIDC_CLIENT_SECRET")?;
    let cookie_key = env_or_file("PRIVATE_COOKIE_KEY")?;
    let redirect_uri = env::var("OIDC_REDIRECT_URI").expect("OIDC_REDIRECT_URI must be set");
    let auth_endpoint = env::var("OIDC_AUTH_ENDPOINT").expect("OIDC_AUTH_ENDPOINT must be set");
    let token_endpoint = env::var("OIDC_TOKEN_ENDPOINT").expect("OIDC_TOKEN_ENDPOINT must be set");
    let post_logout_uri = env::var("OIDC_POST_LOGOUT_REDIRECT_URI")
        .expect("OIDC_POST_LOGOUT_REDIRECT_URI must be set");
    let session_max_age_str = env::var("SESSION_MAX_AGE").expect("SESSION_MAX_AGE must be set");

    let session_max_age = session_max_age_str
        .parse::<i64>()
        .map_err(|_| "SESSION_MAX_AGE must be a valid integer (minutes)")?;

    let mut builder = OAuthConfigurationBuilder::default()
        .with_client_id(&client_id)
        .with_client_secret(&client_secret)
        .with_private_cookie_key(&cookie_key)
        .with_redirect_uri(&redirect_uri)
        .with_authorization_endpoint(&auth_endpoint)
        .with_token_endpoint(&token_endpoint)
        .with_post_logout_redirect_uri(&post_logout_uri)
        .with_scopes(vec!["openid", "profile", "email", "groups"])
        .with_session_max_age(session_max_age);

    // Trust a self-signed identity provider (Kanidm in local dev) when a CA certificate path is provided.
    if let Ok(ca_cert) = env::var("OIDC_CA_CERT") {
        builder = builder.with_custom_ca_cert(&ca_cert);
    }

    Ok(builder.build()?)
}
