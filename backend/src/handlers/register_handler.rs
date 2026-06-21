//! Public self-registration – `POST /register` (no session required).

use axum::{Json, extract::State};

use crate::error::AppError;
use crate::models::{RegisterRequest, RegisterResponse};
use crate::state::AppState;
use crate::validate;

/// Create a new Kanidm account and return a credential-reset link the user opens
/// to set their password before logging in.
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let username = req.username.trim();
    validate::validate_kanidm_name(username)?;

    let registrar = state
        .registrar
        .as_ref()
        .ok_or_else(|| AppError::Internal("registration is not configured".to_string()))?;

    let reset_url = registrar.register(username).await?;

    Ok(Json(RegisterResponse {
        username: username.to_string(),
        reset_url,
    }))
}
