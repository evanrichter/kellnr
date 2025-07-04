use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::original_name::NameError;
use common::version::VersionError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use zip::result::ZipError;

pub type ApiResult<T> = Result<T, ApiError>;

pub struct ApiError {
    status: StatusCode,
    details: ErrorDetails,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorDetails {
    pub errors: Vec<ErrorDetail>,
}

impl From<String> for ErrorDetails {
    fn from(e: String) -> Self {
        let detail = ErrorDetail { detail: e };
        Self {
            errors: vec![detail],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorDetail {
    pub detail: String,
}

impl ApiError {
    pub fn new(msg: &str, error: &str, status: StatusCode) -> Self {
        let e = if error.is_empty() {
            format!("ERROR: {msg}")
        } else {
            format!("ERROR: {msg} -> {error}")
        };
        Self {
            status,
            details: ErrorDetails::from(e),
        }
    }

    fn from_str(e: &str, status: StatusCode) -> Self {
        Self {
            status,
            details: ErrorDetails::from(format!("ERROR: {e}")),
        }
    }

    pub fn from_err(e: &dyn std::error::Error, status: StatusCode) -> Self {
        let e = format!("ERROR: {e}");
        Self {
            status,
            details: ErrorDetails::from(e),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.details)).into_response()
    }
}

impl From<NameError> for ApiError {
    fn from(e: NameError) -> Self {
        ApiError::from_err(&e, StatusCode::BAD_REQUEST)
    }
}

impl From<VersionError> for ApiError {
    fn from(e: VersionError) -> Self {
        ApiError::from_err(&e, StatusCode::BAD_REQUEST)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError::from_err(&e, StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<ZipError> for ApiError {
    fn from(e: ZipError) -> Self {
        match e {
            ZipError::Io(e) => ApiError::from_err(&e, StatusCode::INTERNAL_SERVER_ERROR),
            ZipError::InvalidArchive(s) => ApiError::from_str(&s, StatusCode::BAD_REQUEST),
            ZipError::UnsupportedArchive(s) => ApiError::from_str(s, StatusCode::BAD_REQUEST),
            ZipError::FileNotFound => {
                ApiError::from_str("Zip archive not found", StatusCode::NOT_FOUND)
            }
            _ => ApiError::from_str("Unknown zip error", StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details.errors[0].detail)
    }
}

impl From<db::error::DbError> for ApiError {
    fn from(e: db::error::DbError) -> Self {
        ApiError::from_err(&e, StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<storage::storage_error::StorageError> for ApiError {
    fn from(e: storage::storage_error::StorageError) -> Self {
        ApiError::from_err(&e, StatusCode::INTERNAL_SERVER_ERROR)
    }
}
