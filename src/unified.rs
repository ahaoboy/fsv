use axum::{
    extract::{Request, State},
    http::Method,
    response::{IntoResponse, Response},
};

use crate::error::FsvError;
use crate::handlers;
use crate::types::AppState;
use crate::webdav;

/// Unified handler that routes based on HTTP method:
/// - GET: Serve HTML (root) or files
/// - POST: API endpoints (determined by path)
/// - PROPFIND/OPTIONS/HEAD: WebDAV protocol
pub async fn unified_handler(
    State(state): State<AppState>,
    method: Method,
    request: Request,
) -> Result<Response, FsvError> {
    let path = request.uri().path().to_string();

    match method {
        // WebDAV methods
        Method::OPTIONS => webdav::webdav_handler(State(state), method, request).await,
        ref m if m.as_str() == "PROPFIND" => {
            webdav::webdav_handler(State(state), method, request).await
        }
        Method::HEAD => webdav::webdav_handler(State(state), method, request).await,

        // GET: HTML or file download
        Method::GET => handle_get(State(state), &path, request).await,

        // POST: API calls
        Method::POST => handle_post(State(state), &path, request).await,

        _ => Ok(axum::response::Response::builder()
            .status(405)
            .body("Method not allowed".into())
            .unwrap()),
    }
}

/// Handle GET requests - serve HTML or files
async fn handle_get(
    State(state): State<AppState>,
    path: &str,
    _request: Request,
) -> Result<Response, FsvError> {
    // Root path returns the HTML UI
    if path == "/" {
        return Ok(handlers::index().await.into_response());
    }

    // All other paths: serve as files
    let rel_path = path.trim_start_matches('/');
    let rel_path = if rel_path.is_empty() {
        None
    } else {
        Some(rel_path)
    };

    webdav::webdav_get(&state, rel_path).await
}

/// Handle POST requests - API endpoints
async fn handle_post(
    State(state): State<AppState>,
    path: &str,
    request: Request,
) -> Result<Response, FsvError> {
    match path {
        "/list" => {
            let query = request.uri().query().unwrap_or("");
            let params: crate::types::FileParams =
                serde_urlencoded::from_str(query).unwrap_or_default();
            handlers::list(State(state), axum::extract::Query(params))
                .await
                .map(|json| json.into_response())
        }

        "/file" => {
            let query = request.uri().query().unwrap_or("");
            let params: crate::types::FileParams =
                serde_urlencoded::from_str(query).unwrap_or_default();
            handlers::file(State(state), axum::extract::Query(params))
                .await
                .map(|resp| resp.into_response())
        }

        "/ws-info" => Ok(handlers::ws_info(State(state))
            .await
            .into_response()),

        "/health" => Ok(handlers::health()
            .await
            .into_response()),

        _ => Ok(axum::response::Response::builder()
            .status(404)
            .body("API endpoint not found".into())
            .unwrap()),
    }
}
