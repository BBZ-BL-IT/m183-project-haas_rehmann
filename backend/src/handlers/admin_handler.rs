//! Admin-only handlers. Every handler re-checks the admin role from the live
//! OIDC token (`Identity::require_admin`) – the route being "admin" is not
//! enough on its own.

use axum::{Json, extract::State};
use axum_oidc_client::auth_session::AuthSession;

use crate::db;
use crate::error::AppError;
use crate::identity::Identity;
use crate::models::{
    AdminUpdateUserRequest, AdminUpdateUserResponse, AdminUserListResponse, AdminUserRow,
};
use crate::state::AppState;

/// `GET /admin/userlist` – list every user (admin only).
pub async fn list_users(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<AdminUserListResponse>, AppError> {
    Identity::from_session(&session)?.require_admin()?;

    let rows = db::list_users(state.db_pool.as_ref()).await?;
    let users = rows
        .into_iter()
        .map(|r| AdminUserRow {
            id: r.id,
            appname: r.appname,
            balance: r.balance,
        })
        .collect();

    Ok(Json(AdminUserListResponse { users }))
}

/// `POST /admin/update/user` – rename a user and optionally set their balance.
pub async fn update_user(
    State(state): State<AppState>,
    session: AuthSession,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<AdminUpdateUserResponse>, AppError> {
    Identity::from_session(&session)?.require_admin()?;

    let appname = req.appname.trim();
    if appname.is_empty() {
        return Err(AppError::BadRequest("appname must not be empty".to_string()));
    }
    if let Some(balance) = req.balance {
        if balance < 0 {
            return Err(AppError::BadRequest(
                "balance must not be negative".to_string(),
            ));
        }
    }

    let (appname, balance) =
        db::update_user(state.db_pool.as_ref(), req.id, appname, req.balance).await?;

    Ok(Json(AdminUpdateUserResponse { appname, balance }))
}
