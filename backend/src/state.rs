use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
}

impl AppState {
    pub fn new(db_pool: PgPool) -> Self {
        let db_pool = Arc::new(db_pool);

        AppState { db_pool }
    }
}
