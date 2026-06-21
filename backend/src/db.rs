//! All SQL lives here (sqlx runtime queries – no compile-time DB needed).

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

use crate::error::AppError;

/// Minimal identity row.
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
}

/// Fetch the user for this OIDC subject, creating the row (and its 1:1
/// bank_account + stats rows) on first sight. The stored `username` is kept on
/// return visits so admin renames stick; `email`/`is_admin` are refreshed.
pub async fn get_or_create_user(
    pool: &PgPool,
    subject: &str,
    username: &str,
    email: Option<&str>,
    is_admin: bool,
) -> Result<UserRow, AppError> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (subject, username, email, is_admin)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (subject) DO UPDATE
            SET is_admin   = EXCLUDED.is_admin,
                email      = COALESCE(EXCLUDED.email, users.email),
                updated_at = now()
        RETURNING id, username
        "#,
    )
    .bind(subject)
    .bind(username)
    .bind(email)
    .bind(is_admin)
    .fetch_one(&mut *tx)
    .await?;

    // Ensure the 1:1 child rows exist.
    sqlx::query("INSERT INTO bank_accounts (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING")
        .bind(row.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO stats (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING")
        .bind(row.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(row)
}

/// The joined view used to build `UserInfo`.
#[derive(Debug, sqlx::FromRow)]
pub struct UserProfile {
    pub username: String,
    pub balance: i64,
    pub total_spent: i64,
    pub total_profit: i64,
    pub highest_win_streak: i32,
    pub loans_taken: i64,
    pub loans_value: i64,
}

pub async fn fetch_profile(pool: &PgPool, user_id: i64) -> Result<UserProfile, AppError> {
    let profile = sqlx::query_as::<_, UserProfile>(
        r#"
        SELECT u.username,
               b.amount             AS balance,
               s.total_spent,
               s.total_profit,
               s.highest_win_streak,
               s.loans_taken,
               s.loans_value
        FROM users u
        JOIN bank_accounts b ON b.user_id = u.id
        JOIN stats s         ON s.user_id = u.id
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(profile)
}

/// Loan-limit window state: how many loans were taken inside the rolling window
/// and the oldest one's timestamp (to compute when a slot frees up).
pub struct LoanWindow {
    pub count: i64,
    pub oldest: Option<DateTime<Utc>>,
}

pub async fn loan_window(
    pool: &PgPool,
    user_id: i64,
    window_seconds: i64,
) -> Result<LoanWindow, AppError> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*)::bigint AS cnt,
               MIN(taken_at)    AS oldest
        FROM loans
        WHERE user_id = $1
          AND taken_at >= now() - make_interval(secs => $2)
        "#,
    )
    .bind(user_id)
    .bind(window_seconds as f64)
    .fetch_one(pool)
    .await?;

    Ok(LoanWindow {
        count: row.try_get("cnt")?,
        oldest: row.try_get("oldest")?,
    })
}

/// Record a loan: insert the row, credit the bank account, bump lifetime stats.
/// Returns the new balance.
pub async fn take_loan(pool: &PgPool, user_id: i64, amount: i64) -> Result<i64, AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO loans (user_id, amount) VALUES ($1, $2)")
        .bind(user_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE stats SET loans_taken = loans_taken + 1, loans_value = loans_value + $2, updated_at = now() WHERE user_id = $1",
    )
    .bind(user_id)
    .bind(amount)
    .execute(&mut *tx)
    .await?;

    let balance: i64 = sqlx::query(
        "UPDATE bank_accounts SET amount = amount + $2, updated_at = now() WHERE user_id = $1 RETURNING amount",
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(&mut *tx)
    .await?
    .try_get("amount")?;

    tx.commit().await?;
    Ok(balance)
}

/// Result of settling a spin.
pub struct SpinResult {
    pub balance: i64,
    pub total_spent: i64,
    pub total_profit: i64,
    pub highest_win_streak: i32,
}

/// Apply a spin atomically: verify affordability, settle balance + stats
/// (profit and win-streak), write the audit row. `won` marks a net-positive
/// spin for the win-streak counter.
pub async fn record_spin(
    pool: &PgPool,
    user_id: i64,
    stake: i64,
    amount_earned: i64,
    won: bool,
    reels: &[i32],
) -> Result<SpinResult, AppError> {
    let mut tx = pool.begin().await?;

    let balance: i64 = sqlx::query("SELECT amount FROM bank_accounts WHERE user_id = $1 FOR UPDATE")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?
        .try_get("amount")?;

    if balance < stake {
        return Err(AppError::BadRequest("insufficient balance".to_string()));
    }

    let new_balance: i64 = sqlx::query(
        "UPDATE bank_accounts SET amount = amount - $2 + $3, updated_at = now() WHERE user_id = $1 RETURNING amount",
    )
    .bind(user_id)
    .bind(stake)
    .bind(amount_earned)
    .fetch_one(&mut *tx)
    .await?
    .try_get("amount")?;

    let stats = sqlx::query(
        r#"
        UPDATE stats
        SET total_spent        = total_spent + $2,
            total_profit       = total_profit + ($3 - $2),
            current_win_streak = CASE WHEN $4 THEN current_win_streak + 1 ELSE 0 END,
            highest_win_streak = CASE
                WHEN $4 AND current_win_streak + 1 > highest_win_streak
                THEN current_win_streak + 1 ELSE highest_win_streak END,
            updated_at = now()
        WHERE user_id = $1
        RETURNING total_spent, total_profit, highest_win_streak
        "#,
    )
    .bind(user_id)
    .bind(stake)
    .bind(amount_earned)
    .bind(won)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO spins (user_id, stake, amount_earned, reels) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(stake)
        .bind(amount_earned)
        .bind(reels.to_vec())
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(SpinResult {
        balance: new_balance,
        total_spent: stats.try_get("total_spent")?,
        total_profit: stats.try_get("total_profit")?,
        highest_win_streak: stats.try_get("highest_win_streak")?,
    })
}

// ── Admin ───────────────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct AdminUserRow {
    pub id: i64,
    pub username: String,
    pub balance: i64,
    pub loans_value: i64,
    pub loans_taken: i64,
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<AdminUserRow>, AppError> {
    let rows = sqlx::query_as::<_, AdminUserRow>(
        r#"
        SELECT u.id, u.username, b.amount AS balance, s.loans_value, s.loans_taken
        FROM users u
        JOIN bank_accounts b ON b.user_id = u.id
        JOIN stats s         ON s.user_id = u.id
        ORDER BY u.id
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Apply admin edits. Each `Option` is applied only when `Some`. Returns the
/// resulting row, or `NotFound` if the id doesn't exist.
pub async fn admin_update_user(
    pool: &PgPool,
    id: i64,
    username: Option<&str>,
    balance: Option<i64>,
    loans_value: Option<i64>,
    loans_taken: Option<i64>,
) -> Result<AdminUserRow, AppError> {
    let mut tx = pool.begin().await?;

    // Make sure the user exists (so we can 404 cleanly).
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("user {id} not found")));
    }

    if let Some(username) = username {
        sqlx::query("UPDATE users SET username = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(username)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(balance) = balance {
        sqlx::query("UPDATE bank_accounts SET amount = $2, updated_at = now() WHERE user_id = $1")
            .bind(id)
            .bind(balance)
            .execute(&mut *tx)
            .await?;
    }
    if loans_value.is_some() || loans_taken.is_some() {
        sqlx::query(
            r#"
            UPDATE stats
            SET loans_value = COALESCE($2, loans_value),
                loans_taken = COALESCE($3, loans_taken),
                updated_at  = now()
            WHERE user_id = $1
            "#,
        )
        .bind(id)
        .bind(loans_value)
        .bind(loans_taken)
        .execute(&mut *tx)
        .await?;
    }

    let row = sqlx::query_as::<_, AdminUserRow>(
        r#"
        SELECT u.id, u.username, b.amount AS balance, s.loans_value, s.loans_taken
        FROM users u
        JOIN bank_accounts b ON b.user_id = u.id
        JOIN stats s         ON s.user_id = u.id
        WHERE u.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(row)
}

/// Delete a user (cascades to bank_account, stats, loans, spins).
pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("user {id} not found")));
    }
    Ok(())
}
