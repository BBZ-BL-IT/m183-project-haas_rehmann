//! Runtime configuration read from environment variables.

use std::env;

/// Loan rules, configurable via env (see `.env.example`).
#[derive(Debug, Clone, Copy)]
pub struct LoanConfig {
    /// Maximum number of loans allowed within one rolling window.
    pub max_per_window: i64,
    /// Length of the rolling window, in seconds.
    pub window_seconds: i64,
    /// Maximum amount per individual loan.
    pub max_amount: i64,
}

impl LoanConfig {
    pub fn from_env() -> Self {
        LoanConfig {
            max_per_window: env_i64("LOAN_MAX_PER_WINDOW", 3),
            window_seconds: env_i64("LOAN_WINDOW_SECONDS", 86_400),
            max_amount: env_i64("LOAN_MAX_AMOUNT", 10_000),
        }
    }
}

fn env_i64(name: &str, default: i64) -> i64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}
