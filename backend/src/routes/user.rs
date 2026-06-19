use crate::{handlers::user_handler::get_user_info, state::AppState};
use axum::{Router, routing::get};

pub fn user_routes() -> Router<AppState> {
    Router::new().route("/me", get(get_user_info))
}
