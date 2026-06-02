use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
};
use percent_encoding::percent_decode_str;

use crate::error::FsvError;
use crate::handlers;
use crate::types::AppState;
use crate::util::resolve_safe_path;
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
    // Decode URL-encoded path (e.g., %20 -> space)
    let path = percent_decode_str(request.uri().path())
        .decode_utf8()
        .unwrap_or_default()
        .to_string();

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

/// Handle GET requests - serve HTML, redirect directories, or serve files
async fn handle_get(
    State(state): State<AppState>,
    path: &str,
    request: Request,
) -> Result<Response, FsvError> {
    // Root path always returns the HTML UI
    if path == "/" {
        return Ok(handlers::index().await.into_response());
    }

    let rel_path = path.trim_start_matches('/');
    let rel_path_opt = if rel_path.is_empty() { None } else { Some(rel_path) };

    // If the path maps to a directory, redirect to the SPA with a hash fragment
    if let Ok(target) = resolve_safe_path(&state.root_path, rel_path_opt)
        && target.is_dir() {
            // Use the original percent-encoded path from the URI for the redirect
            let raw_path = request.uri().path().trim_start_matches('/');
            return Ok(Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, format!("/#/{}", raw_path))
                .body(axum::body::Body::empty())
                .unwrap());
        }

    // Check for Range header
    let range_header = request
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok());

    if let Some(range) = range_header {
        webdav::webdav_get_range(&state, rel_path_opt, range).await
    } else {
        webdav::webdav_get(&state, rel_path_opt).await
    }
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

        "/shutdown" => Ok(handlers::shutdown(State(state))
            .await
            .into_response()),

        _ => Ok(axum::response::Response::builder()
            .status(404)
            .body("API endpoint not found".into())
            .unwrap()),
    }
}
