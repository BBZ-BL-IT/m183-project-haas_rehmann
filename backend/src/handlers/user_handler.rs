//! Handlers for the authenticated user's own data: profile and loans.

use axum::{Json, extract::Path, extract::State};
use axum_oidc_client::auth_session::AuthSession;

use crate::db;
use crate::error::AppError;
use crate::identity::Identity;
use crate::models::{LoanResponse, UserInfo};
use crate::state::AppState;

/// Maximum loan amount per request and number of loans allowed per day. Kept in
/// sync with the constants in `frontend/src/components/LoanButton.vue`.
const MAX_LOAN_AMOUNT: i64 = 10_000;
const MAX_LOANS_PER_DAY: i64 = 3;

/// `GET /user/info` – the dashboard payload. Also lazily provisions the local
/// user row on first login.
pub async fn get_user_info(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<UserInfo>, AppError> {
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

    let stats = db::loan_stats(pool, user.id).await?;

    Ok(Json(UserInfo {
        appname: user.appname,
        roles: identity.roles,
        balance: user.balance,
        loans_total_amount: stats.loans_total_amount,
        loans_taken: stats.loans_taken,
        // No repayment flow, so everything borrowed is still owed.
        loans_total_owed: stats.loans_total_amount,
        total_spent: user.total_spent,
        total_win: user.total_win,
    }))
}

/// `POST /loan/{amount}` – take out a loan (the amount is in the URL path).
pub async fn take_loan(
    State(state): State<AppState>,
    session: AuthSession,
    Path(amount): Path<i64>,
) -> Result<Json<LoanResponse>, AppError> {
    if amount <= 0 || amount > MAX_LOAN_AMOUNT {
        return Err(AppError::BadRequest(format!(
            "loan amount must be between 1 and {MAX_LOAN_AMOUNT}"
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

    let stats = db::loan_stats(pool, user.id).await?;
    if stats.loans_taken >= MAX_LOANS_PER_DAY {
        return Err(AppError::TooManyRequests(format!(
            "daily loan limit of {MAX_LOANS_PER_DAY} reached"
        )));
    }

    let balance = db::take_loan(pool, user.id, amount).await?;
    let stats = db::loan_stats(pool, user.id).await?;

    Ok(Json(LoanResponse {
        balance,
        loans_total_amount: stats.loans_total_amount,
        loans_taken: stats.loans_taken,
        loans_total_owed: stats.loans_total_amount,
    }))
}
