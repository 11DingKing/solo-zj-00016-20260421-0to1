use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Invalid URL")]
    InvalidUrl,
    
    #[error("Short link not found")]
    NotFound,
    
    #[error("Short link expired")]
    Expired,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Redis(e) => {
                tracing::error!("Redis error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::InvalidUrl => (StatusCode::BAD_REQUEST, "Invalid URL"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Short link not found"),
            AppError::Expired => (StatusCode::GONE, "Short link has expired"),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
