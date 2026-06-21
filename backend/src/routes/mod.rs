//! HTTP route table. Paths mirror `frontend/src/api/endpoints.ts` exactly.
//!
//! The OIDC routes (`/auth`, `/auth/callback`, `/auth/logout`) are NOT declared
//! here – they are intercepted by the `AuthenticationLayer` wrapped around this
//! router in `app.rs`.

use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::{admin_handler, slot_handler, user_handler};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Liveness probe for docker-compose / orchestration health checks.
        .route("/health", get(|| async { "ok" }))
        // User
        .route("/user/info", get(user_handler::get_user_info))
        .route("/loan/{amount}", post(user_handler::take_loan))
        // Slot
        .route("/spin", post(slot_handler::spin))
        // Admin
        .route("/admin/userlist", get(admin_handler::list_users))
        .route("/admin/update/user", post(admin_handler::update_user))
        .with_state(state)
}
