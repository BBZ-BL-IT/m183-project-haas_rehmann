//! Data-transfer objects shared with the Vue frontend.
//!
//! These mirror `frontend/src/types/api.ts` one-to-one. The field names and JSON
//! shapes here ARE the contract; if you rename a field, update the frontend too.

use serde::{Deserialize, Serialize};

/// `GET /user/info` – everything the UI needs to render the user dashboard.
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub appname: String,
    /// Always contains `"user"`, plus `"admin"` when the OIDC token grants it.
    pub roles: Vec<String>,
    pub balance: i64,
    pub loans_total_amount: i64,
    pub loans_taken: i64,
    pub loans_total_owed: i64,
    pub total_spent: i64,
    pub total_win: i64,
}

/// `POST /spin` request body. The client only chooses the stake – the server
/// decides the outcome (never trust the client with the reels or the payout).
#[derive(Debug, Deserialize)]
pub struct SpinRequest {
    pub stake_amount: i64,
}

/// `POST /spin` response: the three reel symbols plus the authoritative new
/// balance/stats so the frontend can render server truth instead of guessing.
#[derive(Debug, Serialize)]
pub struct SpinResponse {
    /// Exactly three symbols (values 1..=7).
    pub reels: Vec<i32>,
    pub amount_earned: i64,
    pub balance: i64,
    pub total_spent: i64,
    pub total_win: i64,
}

/// `POST /loan/{amount}` response.
#[derive(Debug, Serialize)]
pub struct LoanResponse {
    pub balance: i64,
    pub loans_total_amount: i64,
    pub loans_taken: i64,
    pub loans_total_owed: i64,
}

/// One row of the admin user table.
#[derive(Debug, Serialize)]
pub struct AdminUserRow {
    pub id: i64,
    pub appname: String,
    pub balance: i64,
}

/// `GET /admin/userlist` response.
#[derive(Debug, Serialize)]
pub struct AdminUserListResponse {
    pub users: Vec<AdminUserRow>,
}

/// `POST /admin/update/user` request body.
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub id: i64,
    pub appname: String,
    /// Optional – admins may also adjust a user's balance.
    pub balance: Option<i64>,
}

/// `POST /admin/update/user` response.
#[derive(Debug, Serialize)]
pub struct AdminUpdateUserResponse {
    pub appname: String,
    pub balance: i64,
}
