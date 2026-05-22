use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde_json::json;
use std::io;
use std::net::AddrParseError;

/// All errors that can occur during fsv operation.
#[derive(thiserror::Error, Debug)]
pub enum FsvError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Address parse error: {0}")]
    AddrParse(#[from] AddrParseError),

    #[error("Server shutdown error: {0}")]
    Shutdown(String),

    #[error("WebSocket broadcast error: {0}")]
    Broadcast(String),

    #[error("Path error: {0}")]
    PathError(String),

    #[error("Path not found")]
    NotFound,

    #[error("Access denied (directory traversal prevented)")]
    AccessDenied,

    #[error("Target is a directory, not a file")]
    NotAFile,
}

/// Maps `FsvError` variants to HTTP status codes and JSON error bodies.
impl IntoResponse for FsvError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            FsvError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            FsvError::AccessDenied => (StatusCode::FORBIDDEN, self.to_string()),
            FsvError::NotAFile => (StatusCode::BAD_REQUEST, self.to_string()),
            FsvError::Io(ref e) => match e.kind() {
                io::ErrorKind::NotFound => (StatusCode::NOT_FOUND, "Path not found".into()),
                io::ErrorKind::PermissionDenied => {
                    (StatusCode::FORBIDDEN, "Permission denied".into())
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("IO error: {}", e),
                ),
            },
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
