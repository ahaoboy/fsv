use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};
use tokio_util::io::ReaderStream;
use tracing;

use crate::error::FsvError;
use crate::types::{AppState, FileInfo, FileParams};
use crate::util::{get_file_info, resolve_safe_path};

/// Lists directory contents or returns metadata for a single file.
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<FileParams>,
) -> Result<Json<Vec<FileInfo>>, FsvError> {
    let canonical_root = state.root_path.canonicalize().map_err(|e| {
        FsvError::PathError(format!("Failed to canonicalize root path: {}", e))
    })?;

    // When the root itself is a file, return its info directly.
    if canonical_root.is_file() {
        tracing::debug!(root = %canonical_root.display(), "listing single-file root");
        return Ok(Json(vec![get_file_info(&canonical_root, &canonical_root)?]));
    }

    let target = resolve_safe_path(&state.root_path, params.path.as_deref())?;

    tracing::debug!(target = %target.display(), "listing directory");

    if target.is_file() {
        return Ok(Json(vec![get_file_info(&canonical_root, &target)?]));
    }

    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(&target).await?;
    while let Some(entry) = dir.next_entry().await? {
        if let Ok(info) = get_file_info(&canonical_root, &entry.path()) {
            entries.push(info);
        }
    }

    // Directories first, then alphabetical.
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.cmp(&b.name))
    });

    tracing::debug!(count = entries.len(), "directory listing complete");

    Ok(Json(entries))
}

/// Streams a file to the client as an octet-stream download.
pub async fn file(
    State(state): State<AppState>,
    Query(params): Query<FileParams>,
) -> Result<impl IntoResponse, FsvError> {
    let canonical_root = state.root_path.canonicalize().map_err(|e| {
        FsvError::PathError(format!("Failed to canonicalize root path: {}", e))
    })?;

    let target = if canonical_root.is_file() {
        canonical_root
    } else {
        resolve_safe_path(&state.root_path, params.path.as_deref())?
    };

    if !target.is_file() {
        return Err(FsvError::NotAFile);
    }

    let file = tokio::fs::File::open(&target).await?;
    let file_len = file.metadata().await?.len();
    let file_name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".into());

    tracing::debug!(
        path = %target.display(),
        size = file_len,
        "streaming file download"
    );

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/octet-stream"),
    );
    if let Ok(v) = axum::http::HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        file_name
    )) {
        headers.insert(header::CONTENT_DISPOSITION, v);
    }
    if let Ok(v) = axum::http::HeaderValue::from_str(&file_len.to_string()) {
        headers.insert(header::CONTENT_LENGTH, v);
    }

    Ok((headers, Body::from_stream(ReaderStream::new(file))))
}

/// Serves the bundled frontend SPA.
pub async fn index() -> axum::response::Html<&'static str> {
    tracing::debug!("serving SPA index.html");
    axum::response::Html(include_str!("../dist/index.html"))
}

/// Returns WebSocket connection statistics.
pub async fn ws_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    let count = state.ws_connections.load(std::sync::atomic::Ordering::Relaxed);
    tracing::debug!(connected = count, "ws-info requested");
    Json(serde_json::json!({
        "connected": count,
        "broadcast_capacity": 100,
    }))
}

/// Health check endpoint - returns server status and uptime.
pub async fn health() -> Json<serde_json::Value> {
    tracing::trace!("health check requested");
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }))
}

/// Shuts down the server gracefully.
pub async fn shutdown(State(state): State<AppState>) -> Json<serde_json::Value> {
    tracing::info!("shutdown API called, scheduling graceful shutdown");
    // Delay shutdown so the HTTP response can be flushed to the client first.
    let notify = state.shutdown_notify.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        notify.notify_waiters();
    });
    Json(serde_json::json!({
        "message": "Shutting down...",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }))
}
