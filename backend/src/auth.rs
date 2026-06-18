use axum_oidc_client::auth_builder::OAuthConfigurationBuilder;
use std::env;

pub fn get_oidc_config()
-> Result<axum_oidc_client::auth::OAuthConfiguration, Box<dyn std::error::Error>> {
    let client_id = env::var("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID must be set");
    let client_secret = env::var("OIDC_CLIENT_SECRET").expect("OIDC_CLIENT_SECRET must be set");
    let cookie_key = env::var("PRIVATE_COOKIE_KEY").expect("PRIVATE_COOKIE_KEY must be set");
    let redirect_uri = env::var("OIDC_REDIRECT_URI").expect("OIDC_REDIRECT_URI must be set");
    let auth_endpoint = env::var("OIDC_AUTH_ENDPOINT").expect("OIDC_AUTH_ENDPOINT must be set");
    let token_endpoint = env::var("OIDC_TOKEN_ENDPOINT").expect("OIDC_TOKEN_ENDPOINT must be set");

    let config = OAuthConfigurationBuilder::default()
        .with_client_id(&client_id)
        .with_client_secret(&client_secret)
        .with_private_cookie_key(&cookie_key)
        .with_redirect_uri(&redirect_uri)
        .with_authorization_endpoint(&auth_endpoint)
        .with_token_endpoint(&token_endpoint)
        .with_scopes(vec!["openid", "profile", "email"])
        .build()?;

    Ok(config)
}
