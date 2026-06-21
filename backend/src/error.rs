//! Application error type and its mapping to HTTP responses.
//!
//! Every handler returns `Result<_, AppError>`. The error is rendered as a JSON
//! body `{ "error": <code>, "message": <human readable> }`, which matches the
//! `ApiError` shape the Vue frontend expects (`toApiError` reads `message`).

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    /// 400 – the request itself was malformed (e.g. invalid loan amount).
    BadRequest(String),
    /// 401 – no valid session / the OIDC token could not be read.
    Unauthorized(String),
    /// 403 – authenticated, but missing the required role.
    Forbidden(String),
    /// 404 – the referenced resource does not exist.
    NotFound(String),
    /// 429 – a rate/quota limit was hit (e.g. more than 3 loans per day).
    TooManyRequests(String),
    /// 500 – anything unexpected (database failures, etc.).
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
