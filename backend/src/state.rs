use sqlx::PgPool;
use std::sync::Arc;

use crate::config::LoanConfig;
use crate::kanidm::KanidmRegistrar;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
    pub loan_config: LoanConfig,
    /// `None` when self-registration isn't configured.
    pub registrar: Option<KanidmRegistrar>,
}

impl AppState {
    pub fn new(
        db_pool: PgPool,
        loan_config: LoanConfig,
        registrar: Option<KanidmRegistrar>,
    ) -> Self {
        AppState {
            db_pool: Arc::new(db_pool),
            loan_config,
            registrar,
        }
    }
}
