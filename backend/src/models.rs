use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub roles: Vec<String>,
    pub balance: i64,
    pub total_spent: i64,
    pub total_profit: i64,
    pub highest_win_streak: i32,
    pub loans_taken: i64,
    pub loans_value: i64,
    pub loans_in_window: i64,
    pub loans_max: i64,
    pub loans_window_seconds: i64,
    pub loans_reset_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpinRequest {
    pub stake_amount: i64,
}

#[derive(Debug, Serialize)]
pub struct SpinResponse {
    pub reels: Vec<i32>,
    pub amount_earned: i64,
    pub balance: i64,
    pub total_spent: i64,
    pub total_profit: i64,
    pub highest_win_streak: i32,
}

#[derive(Debug, Serialize)]
pub struct LoanResponse {
    pub balance: i64,
    pub loans_value: i64,
    pub loans_taken: i64,
    pub loans_in_window: i64,
    pub loans_max: i64,
    pub loans_reset_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminUserRow {
    pub id: i64,
    pub username: String,
    pub balance: i64,
    pub loans_value: i64,
    pub loans_taken: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminUserListResponse {
    pub users: Vec<AdminUserRow>,
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub id: i64,
    pub username: Option<String>,
    pub balance: Option<i64>,
    pub loans_value: Option<i64>,
    pub loans_taken: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AdminUpdateUserResponse {
    pub id: i64,
    pub username: String,
    pub balance: i64,
    pub loans_value: i64,
    pub loans_taken: i64,
}
