use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    TooManyRequests(String),
    Internal(String),
}

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, &str) {
        match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "unauthorized", m),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "forbidden", m),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m),
            AppError::TooManyRequests(m) => (StatusCode::TOO_MANY_REQUESTS, "too_many_requests", m),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", m),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        // Internal errors are logged in full but never leaked to the client.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("internal error: {message}");
            return (
                status,
                Json(json!({ "error": code, "message": "Internal server error" })),
            )
                .into_response();
        }
        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(format!("database error: {err}"))
    }
}
