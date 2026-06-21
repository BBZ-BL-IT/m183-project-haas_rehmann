//! All SQL lives here. We use sqlx's runtime query API (`query`, `query_as`)
//! rather than the compile-time `query!` macros on purpose: the macros need a
//! live database (or a checked-in `.sqlx/` cache) at *build* time, which would
//! make the Docker image build depend on Postgres. Runtime queries keep the
//! build hermetic while still going through sqlx's typed row mapping.

use sqlx::{PgPool, Row};

use crate::error::AppError;

/// A user row as stored in the database.
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub appname: String,
    pub balance: i64,
    pub total_spent: i64,
    pub total_win: i64,
}

/// Loan figures shown in the UI.
pub struct LoanStats {
    /// Loans taken *today* – drives the "3 per day" limit.
    pub loans_taken: i64,
    /// All-time sum borrowed. With no repayment flow this is also the amount
    /// still owed, so `loans_total_owed` reuses it.
    pub loans_total_amount: i64,
}

/// Fetch the user for this OIDC subject, creating the row on first sight.
///
/// On returning visits we refresh the cached admin flag and email from the
/// token, but deliberately keep the stored `appname` so admin renames stick.
pub async fn get_or_create_user(
    pool: &PgPool,
    subject: &str,
    appname: &str,
    email: Option<&str>,
    is_admin: bool,
) -> Result<UserRow, AppError> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (subject, appname, email, is_admin)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (subject) DO UPDATE
            SET is_admin   = EXCLUDED.is_admin,
                email      = COALESCE(EXCLUDED.email, users.email),
                updated_at = now()
        RETURNING id, appname, balance, total_spent, total_win
        "#,
    )
    .bind(subject)
    .bind(appname)
    .bind(email)
    .bind(is_admin)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Today's loan count and the all-time borrowed total for a user.
pub async fn loan_stats(pool: &PgPool, user_id: i64) -> Result<LoanStats, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE taken_at >= date_trunc('day', now()))  AS loans_taken,
            -- SUM(bigint) is NUMERIC in Postgres; cast back to bigint for i64.
            COALESCE(SUM(amount), 0)::bigint                              AS loans_total_amount
        FROM loans
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(LoanStats {
        loans_taken: row.try_get::<i64, _>("loans_taken")?,
        loans_total_amount: row.try_get::<i64, _>("loans_total_amount")?,
    })
}

/// Record a loan: insert the row and credit the user's balance atomically.
/// Returns the new balance.
pub async fn take_loan(pool: &PgPool, user_id: i64, amount: i64) -> Result<i64, AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO loans (user_id, amount) VALUES ($1, $2)")
        .bind(user_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;

    let balance: i64 =
        sqlx::query("UPDATE users SET balance = balance + $2, updated_at = now() WHERE id = $1 RETURNING balance")
            .bind(user_id)
            .bind(amount)
            .fetch_one(&mut *tx)
            .await?
            .try_get("balance")?;

    tx.commit().await?;
    Ok(balance)
}

/// Outcome of recording a spin.
pub struct SpinResult {
    pub balance: i64,
    pub total_spent: i64,
    pub total_win: i64,
}

/// Apply a spin atomically: verify the user can afford the stake, settle the
/// balance/stats and write the audit row. Returns `BadRequest` if the balance is
/// insufficient (the authoritative check – the client cannot bypass it).
pub async fn record_spin(
    pool: &PgPool,
    user_id: i64,
    stake: i64,
    amount_earned: i64,
    reels: &[i32],
) -> Result<SpinResult, AppError> {
    let mut tx = pool.begin().await?;

    // Lock the row so concurrent spins can't both pass the affordability check.
    let balance: i64 = sqlx::query("SELECT balance FROM users WHERE id = $1 FOR UPDATE")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?
        .try_get("balance")?;

    if balance < stake {
        return Err(AppError::BadRequest("insufficient balance".to_string()));
    }

    let row = sqlx::query(
        r#"
        UPDATE users
        SET balance      = balance - $2 + $3,
            total_spent  = total_spent + $2,
            total_win    = total_win + $3,
            updated_at   = now()
        WHERE id = $1
        RETURNING balance, total_spent, total_win
        "#,
    )
    .bind(user_id)
    .bind(stake)
    .bind(amount_earned)
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
        balance: row.try_get("balance")?,
        total_spent: row.try_get("total_spent")?,
        total_win: row.try_get("total_win")?,
    })
}

/// One row of the admin user list.
#[derive(Debug, sqlx::FromRow)]
pub struct AdminUserRow {
    pub id: i64,
    pub appname: String,
    pub balance: i64,
}

/// All users, ordered by id, for the admin table.
pub async fn list_users(pool: &PgPool) -> Result<Vec<AdminUserRow>, AppError> {
    let rows =
        sqlx::query_as::<_, AdminUserRow>("SELECT id, appname, balance FROM users ORDER BY id")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

/// Update a user's display name and (optionally) balance. Returns the new
/// values, or `NotFound` if the id doesn't exist.
pub async fn update_user(
    pool: &PgPool,
    id: i64,
    appname: &str,
    balance: Option<i64>,
) -> Result<(String, i64), AppError> {
    let row = sqlx::query(
        r#"
        UPDATE users
        SET appname    = $2,
            balance    = COALESCE($3, balance),
            updated_at = now()
        WHERE id = $1
        RETURNING appname, balance
        "#,
    )
    .bind(id)
    .bind(appname)
    .bind(balance)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;

    Ok((row.try_get("appname")?, row.try_get("balance")?))
}
