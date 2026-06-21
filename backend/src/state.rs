use sqlx::PgPool;
use std::sync::Arc;

use crate::config::LoanConfig;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
    pub loan_config: LoanConfig,
}

impl AppState {
    pub fn new(db_pool: PgPool, loan_config: LoanConfig) -> Self {
        AppState {
            db_pool: Arc::new(db_pool),
            loan_config,
        }
    }
}
