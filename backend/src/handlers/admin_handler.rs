//! Admin-only handlers. Every handler re-checks the admin role from the live
//! OIDC token (`Identity::require_admin`).

use axum::{Json, extract::Path, extract::State};
use axum_oidc_client::auth_session::AuthSession;

use crate::db;
use crate::error::AppError;
use crate::identity::Identity;
use crate::models::{
    AdminUpdateUserRequest, AdminUpdateUserResponse, AdminUserListResponse, AdminUserRow,
};
use crate::state::AppState;
use crate::validate;

/// `GET /admin/userlist` – list every user (admin only).
pub async fn list_users(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<AdminUserListResponse>, AppError> {
    Identity::from_session(&session)?.require_admin()?;

    let users = db::list_users(state.db_pool.as_ref())
        .await?
        .into_iter()
        .map(|r| AdminUserRow {
            id: r.id,
            username: r.username,
            balance: r.balance,
            loans_value: r.loans_value,
            loans_taken: r.loans_taken,
        })
        .collect();

    Ok(Json(AdminUserListResponse { users }))
}

/// `POST /admin/update/user` – edit username / balance / open loans.
pub async fn update_user(
    State(state): State<AppState>,
    session: AuthSession,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<AdminUpdateUserResponse>, AppError> {
    Identity::from_session(&session)?.require_admin()?;

    let username = match req.username.as_deref().map(str::trim) {
        Some(u) => {
            validate::validate_username(u)?;
            Some(u)
        }
        None => None,
    };
    if let Some(balance) = req.balance {
        if balance < 0 {
            return Err(AppError::BadRequest("balance must not be negative".into()));
        }
    }
    if let Some(v) = req.loans_value {
        if v < 0 {
            return Err(AppError::BadRequest("loans_value must not be negative".into()));
        }
    }
    if let Some(c) = req.loans_taken {
        if c < 0 {
            return Err(AppError::BadRequest("loans_taken must not be negative".into()));
        }
    }

    let row = db::admin_update_user(
        state.db_pool.as_ref(),
        req.id,
        username,
        req.balance,
        req.loans_value,
        req.loans_taken,
    )
    .await?;

    Ok(Json(AdminUpdateUserResponse {
        id: row.id,
        username: row.username,
        balance: row.balance,
        loans_value: row.loans_value,
        loans_taken: row.loans_taken,
    }))
}

/// `POST /admin/delete/user/{id}` – delete a user and all their data.
pub async fn delete_user(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let identity = Identity::from_session(&session)?;
    identity.require_admin()?;

    db::delete_user(state.db_pool.as_ref(), id).await?;
    Ok(Json(serde_json::json!({ "deleted": id })))
}
