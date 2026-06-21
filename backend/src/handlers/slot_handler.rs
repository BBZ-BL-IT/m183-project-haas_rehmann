use axum::{Json, extract::State};
use axum_oidc_client::auth_session::AuthSession;

use crate::db;
use crate::error::AppError;
use crate::game;
use crate::identity::Identity;
use crate::models::{SpinRequest, SpinResponse};
use crate::state::AppState;

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

    let outcome = game::play(req.stake_amount);
    let won = outcome.amount_earned > req.stake_amount;

    let result = db::record_spin(
        pool,
        user.id,
        req.stake_amount,
        outcome.amount_earned,
        won,
        &outcome.reels,
    )
    .await?;

    Ok(Json(SpinResponse {
        reels: outcome.reels,
        amount_earned: outcome.amount_earned,
        balance: result.balance,
        total_spent: result.total_spent,
        total_profit: result.total_profit,
        highest_win_streak: result.highest_win_streak,
    }))
}
