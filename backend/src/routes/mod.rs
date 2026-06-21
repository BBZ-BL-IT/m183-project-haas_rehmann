use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::{admin_handler, slot_handler, user_handler};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/user/info", get(user_handler::get_user_info))
        .route("/loan/{amount}", post(user_handler::take_loan))
        .route("/spin", post(slot_handler::spin))
        .route("/admin/userlist", get(admin_handler::list_users))
        .route("/admin/update/user", post(admin_handler::update_user))
        .route("/admin/delete/user/{id}", post(admin_handler::delete_user))
        .with_state(state)
}
