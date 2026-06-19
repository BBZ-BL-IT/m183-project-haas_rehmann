pub mod user;

use crate::{routes::user::user_routes, state::AppState};
use axum::Router;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/user", user_routes())
        .with_state(state)
}
