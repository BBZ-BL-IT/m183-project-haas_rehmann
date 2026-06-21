//! Handlers for the authenticated user's own data: profile and loans.

use axum::{Json, extract::Path, extract::State};
use axum_oidc_client::auth_session::AuthSession;
use chrono::Duration;
use sqlx::PgPool;

use crate::config::LoanConfig;
use crate::db;
use crate::error::AppError;
use crate::identity::Identity;
use crate::models::{LoanResponse, UserInfo};
use crate::state::AppState;

/// Loan-limit fields for a response: how many loans are used in the current
/// window and (if at the limit) when the next slot frees up.
async fn loan_limit(
    pool: &PgPool,
    user_id: i64,
    cfg: &LoanConfig,
) -> Result<(i64, Option<String>), AppError> {
    let window = db::loan_window(pool, user_id, cfg.window_seconds).await?;
    let reset_at = if window.count >= cfg.max_per_window {
        window
            .oldest
            .map(|t| (t + Duration::seconds(cfg.window_seconds)).to_rfc3339())
    } else {
        None
    };
    Ok((window.count, reset_at))
}

/// `GET /user/info` – the dashboard payload. Lazily provisions the local rows.
pub async fn get_user_info(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<UserInfo>, AppError> {
    let identity = Identity::from_session(&session)?;
    let pool = state.db_pool.as_ref();
    let cfg = state.loan_config;

    let user = db::get_or_create_user(
        pool,
        &identity.subject,
        &identity.appname,
        identity.email.as_deref(),
        identity.is_admin,
    )
    .await?;

    let profile = db::fetch_profile(pool, user.id).await?;
    let (loans_in_window, loans_reset_at) = loan_limit(pool, user.id, &cfg).await?;

    Ok(Json(UserInfo {
        username: profile.username,
        roles: identity.roles,
        balance: profile.balance,
        total_spent: profile.total_spent,
        total_profit: profile.total_profit,
        highest_win_streak: profile.highest_win_streak,
        loans_taken: profile.loans_taken,
        loans_value: profile.loans_value,
        loans_in_window,
        loans_max: cfg.max_per_window,
        loans_window_seconds: cfg.window_seconds,
        loans_reset_at,
    }))
}

/// `POST /loan/{amount}` – take out a loan (amount is in the URL path).
pub async fn take_loan(
    State(state): State<AppState>,
    session: AuthSession,
    Path(amount): Path<i64>,
) -> Result<Json<LoanResponse>, AppError> {
    let cfg = state.loan_config;
    if amount <= 0 || amount > cfg.max_amount {
        return Err(AppError::BadRequest(format!(
            "loan amount must be between 1 and {}",
            cfg.max_amount
        )));
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

    let (count, _) = loan_limit(pool, user.id, &cfg).await?;
    if count >= cfg.max_per_window {
        return Err(AppError::TooManyRequests(format!(
            "loan limit of {} per {}s reached",
            cfg.max_per_window, cfg.window_seconds
        )));
    }

    let balance = db::take_loan(pool, user.id, amount).await?;
    let profile = db::fetch_profile(pool, user.id).await?;
    let (loans_in_window, loans_reset_at) = loan_limit(pool, user.id, &cfg).await?;

    Ok(Json(LoanResponse {
        balance,
        loans_value: profile.loans_value,
        loans_taken: profile.loans_taken,
        loans_in_window,
        loans_max: cfg.max_per_window,
        loans_reset_at,
    }))
}
