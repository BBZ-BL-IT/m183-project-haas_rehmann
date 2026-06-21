//! Data-transfer objects shared with the Vue frontend.
//! These mirror `frontend/src/types/api.ts`; the field names ARE the contract.

use serde::{Deserialize, Serialize};

/// `GET /user/info` – everything the dashboard renders.
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    /// Always `"user"`, plus `"admin"` when the OIDC token grants it.
    pub roles: Vec<String>,
    pub balance: i64,
    // --- stats ---
    pub total_spent: i64,
    /// Net profit; may be negative.
    pub total_profit: i64,
    pub highest_win_streak: i32,
    /// Lifetime number of loans and their total (open) value.
    pub loans_taken: i64,
    pub loans_value: i64,
    // --- loan limit ---
    pub loans_in_window: i64,
    pub loans_max: i64,
    pub loans_window_seconds: i64,
    /// RFC3339 timestamp when the next loan slot frees up (null if under limit).
    pub loans_reset_at: Option<String>,
}

/// `POST /spin` request – the client only picks the stake.
#[derive(Debug, Deserialize)]
pub struct SpinRequest {
    pub stake_amount: i64,
}

/// `POST /spin` response – the three reels plus authoritative new stats.
#[derive(Debug, Serialize)]
pub struct SpinResponse {
    pub reels: Vec<i32>,
    pub amount_earned: i64,
    pub balance: i64,
    pub total_spent: i64,
    pub total_profit: i64,
    pub highest_win_streak: i32,
}

/// `POST /loan/{amount}` response.
#[derive(Debug, Serialize)]
pub struct LoanResponse {
    pub balance: i64,
    pub loans_value: i64,
    pub loans_taken: i64,
    pub loans_in_window: i64,
    pub loans_max: i64,
    pub loans_reset_at: Option<String>,
}

/// `POST /register` request.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
}

/// `POST /register` response – the new account plus a credential-reset link the
/// user opens to set their password before logging in.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub username: String,
    pub reset_url: String,
}

/// One row of the admin user table.
#[derive(Debug, Serialize)]
pub struct AdminUserRow {
    pub id: i64,
    pub username: String,
    pub balance: i64,
    pub loans_value: i64,
    pub loans_taken: i64,
}

/// `GET /admin/userlist` response.
#[derive(Debug, Serialize)]
pub struct AdminUserListResponse {
    pub users: Vec<AdminUserRow>,
}

/// `POST /admin/update/user` request – every field except `id` is optional;
/// only the provided ones are changed.
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub id: i64,
    pub username: Option<String>,
    pub balance: Option<i64>,
    /// Open (owed) loan amount.
    pub loans_value: Option<i64>,
    /// Lifetime loans-taken counter.
    pub loans_taken: Option<i64>,
}

/// `POST /admin/update/user` response.
#[derive(Debug, Serialize)]
pub struct AdminUpdateUserResponse {
    pub id: i64,
    pub username: String,
    pub balance: i64,
    pub loans_value: i64,
    pub loans_taken: i64,
}
