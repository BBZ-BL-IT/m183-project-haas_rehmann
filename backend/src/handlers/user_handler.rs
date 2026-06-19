use crate::state::AppState;
use axum::extract::State;
use axum_oidc_client::auth_session::AuthSession;

pub async fn get_user_info(State(_state): State<AppState>, session: AuthSession) -> String {
    let scopes = session.scope.as_deref().unwrap_or("no scopes provided");

    format!(
        "Auth Test Successful!\n\
         Scopes: {}\n\
         Access Token: {:.15}...\n\
         ID Token: {:.15}...",
        scopes, session.access_token, session.id_token
    )
}
