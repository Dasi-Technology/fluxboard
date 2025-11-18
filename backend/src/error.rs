use actix_web::{HttpResponse, error::ResponseError, http::StatusCode};
use serde::Serialize;
use std::fmt;

/// Custom error types for the application
#[derive(Debug)]
pub enum AppError {
    /// Database operation failed
    DatabaseError(sqlx::Error),
    /// Resource not found
    NotFound(String),
    /// Invalid input or request
    BadRequest(String),
    /// Unauthorized access
    Unauthorized(String),
    /// Forbidden access
    Forbidden(String),
    /// Conflict (e.g., duplicate resource)
    Conflict(String),
    /// Validation error
    ValidationError(String),
    /// Internal server error
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// JSON error response structure
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            AppError::DatabaseError(err) => {
                log::error!("Database error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database operation failed".to_string(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InternalError(msg) => {
                log::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: status.to_string(),
            message,
        })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Convert sqlx::Error to AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

/// Convert actix_web::Error to AppError
impl From<actix_web::Error> for AppError {
    fn from(err: actix_web::Error) -> Self {
        AppError::InternalError(format!("Actix error: {}", err))
    }
}

/// Convert validator::ValidationErrors to AppError
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::ValidationError(format!("Validation failed: {}", err))
    }
}

/// Type alias for Result with AppError
pub type AppResult<T> = Result<T, AppError>;
