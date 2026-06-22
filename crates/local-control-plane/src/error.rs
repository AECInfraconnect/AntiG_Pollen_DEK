use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub code: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "Internal error mapped to envelope");
                let envelope = dek_errors::ErrorEnvelope::internal_error(e.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(envelope)).into_response()
            }
            other => {
                let (status, code) = match &other {
                    ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
                    ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
                    ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
                    ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
                    ApiError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
                    ApiError::Internal(_) => unreachable!(),
                };

                let body = ErrorBody {
                    error: other.to_string(),
                    code: code.to_string(),
                };
                (status, Json(body)).into_response()
            }
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
