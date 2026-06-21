//! Slot machine handler – `POST /spin`.

use axum::{Json, extract::State};
use axum_oidc_client::auth_session::AuthSession;

use crate::db;
use crate::error::AppError;
use crate::game;
use crate::identity::Identity;
use crate::models::{SpinRequest, SpinResponse};
use crate::state::AppState;

/// `POST /spin` – stake some balance, the server rolls the reels and settles.
pub async fn spin(
    State(state): State<AppState>,
    session: AuthSession,
    Json(req): Json<SpinRequest>,
) -> Result<Json<SpinResponse>, AppError> {
    if req.stake_amount <= 0 {
        return Err(AppError::BadRequest(
            "stake_amount must be positive".to_string(),
        ));
    }

    let identity = Identity::from_session(&session)?;
    let pool = state.db_pool.as_ref();

    let user = db::get_or_create_user(
        pool,
        &identity.subject,
        &identity.appname,
        identity.email.as_deref(),
        identity.is_admin,
    )
    .await?;

    // Roll first, then settle. `record_spin` enforces the affordability check
    // inside the transaction, so the stake can never overdraw the balance.
    let outcome = game::play(req.stake_amount);
    let result = db::record_spin(
        pool,
        user.id,
        req.stake_amount,
        outcome.amount_earned,
        &outcome.reels,
    )
    .await?;

    Ok(Json(SpinResponse {
        reels: outcome.reels,
        amount_earned: outcome.amount_earned,
        balance: result.balance,
        total_spent: result.total_spent,
        total_win: result.total_win,
    }))
}
