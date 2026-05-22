use axum::{
    Json,
    body::Body,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    http::header,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use std::net::{AddrParseError, IpAddr};
use std::path::{Path, PathBuf};
use tokio::sync::{broadcast, oneshot};
use tokio_util::io::ReaderStream;

/// Custom error type representing fsv operation failures.
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

/// Converts `FsvError` into an Axum response with appropriate status code and JSON body.
impl IntoResponse for FsvError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            FsvError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            FsvError::AccessDenied => (StatusCode::FORBIDDEN, self.to_string()),
            FsvError::NotAFile => (StatusCode::BAD_REQUEST, self.to_string()),
            FsvError::Io(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    (StatusCode::NOT_FOUND, "Path not found".to_string())
                } else if e.kind() == io::ErrorKind::PermissionDenied {
                    (StatusCode::FORBIDDEN, "Permission denied".to_string())
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("IO error: {}", e),
                    )
                }
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

/// Server startup configuration.
pub struct Config {
    pub path: PathBuf,
    pub port: u16,
}

/// A handle to control the running server.
pub struct ServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    ws_tx: broadcast::Sender<String>,
}

impl ServerHandle {
    /// Gracefully shuts down the running Axum server.
    pub fn shutdown(&mut self) -> Result<(), FsvError> {
        if let Some(tx) = self.shutdown_tx.take() {
            tx.send(())
                .map_err(|_| FsvError::Shutdown("Failed to send shutdown signal".into()))
        } else {
            Err(FsvError::Shutdown(
                "Server is already shut down or handle is uninitialized".into(),
            ))
        }
    }

    /// Broadcasts a message to all connected WebSocket clients.
    /// Returns the number of clients that successfully received the message.
    pub fn send_message(&self, message: &str) -> Result<usize, FsvError> {
        match self.ws_tx.send(message.to_string()) {
            Ok(count) => Ok(count),
            Err(_) => {
                // If there are no receivers, we just return 0 active clients.
                Ok(0)
            }
        }
    }
}

/// Internal application state shared across handlers.
#[derive(Clone)]
struct AppState {
    root_path: PathBuf,
    ws_tx: broadcast::Sender<String>,
}

/// File or directory entry metadata.
#[derive(Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

/// Parameters for querying or downloading files.
#[derive(Deserialize)]
pub struct FileParams {
    pub path: Option<String>,
}

/// Helper function to list all network interface IPs.
fn get_local_ips() -> Vec<IpAddr> {
    let is_valid_ipv4 = |ip: &IpAddr| match ip {
        IpAddr::V4(v4) => !v4.is_link_local(),
        IpAddr::V6(_) => false,
    };

    let primary = local_ip_address::local_ip().ok().filter(|ip| is_valid_ipv4(ip));

    let mut ips: Vec<IpAddr> = primary.into_iter().collect();

    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in interfaces {
            if is_valid_ipv4(&ip) && !ips.contains(&ip) {
                ips.push(ip);
            }
        }
    }

    if ips.is_empty() {
        ips.push(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    }
    ips
}

/// Resolves a path safely under the root path to prevent directory traversal attacks.
fn resolve_safe_path(root_path: &Path, relative_path: Option<&str>) -> Result<PathBuf, FsvError> {
    let canonical_root = root_path
        .canonicalize()
        .map_err(|e| FsvError::PathError(format!("Failed to canonicalize root path: {}", e)))?;

    if let Some(rel) = relative_path {
        // Normalize slashes and trim prefix slashes to avoid absolute path overrides
        let rel_cleaned = rel.trim_start_matches(['/', '\\']);
        let joined = canonical_root.join(rel_cleaned);

        // Try canonicalizing the joined path. If it fails, the file/folder does not exist.
        let canonical_target = joined.canonicalize().map_err(|_| FsvError::NotFound)?;

        // Ensure the canonicalized path starts with the root path to block traversal escape
        if canonical_target.starts_with(&canonical_root) {
            Ok(canonical_target)
        } else {
            Err(FsvError::AccessDenied)
        }
    } else {
        Ok(canonical_root)
    }
}

/// Generates a FileInfo object for a given path relative to the root path.
fn get_file_info(canonical_root: &Path, target_path: &Path) -> Result<FileInfo, FsvError> {
    let metadata = target_path.metadata()?;
    let name = target_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let relative = target_path
        .strip_prefix(canonical_root)
        .unwrap_or_else(|_| Path::new(""))
        .to_string_lossy()
        .into_owned()
        .replace('\\', "/");

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Ok(FileInfo {
        name,
        path: relative,
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        modified,
    })
}

/// Handler to list files and sub-folders or show details for a single target.
async fn list_files_handler(
    State(state): State<AppState>,
    Query(params): Query<FileParams>,
) -> Result<Json<Vec<FileInfo>>, FsvError> {
    let canonical_root = state
        .root_path
        .canonicalize()
        .map_err(|e| FsvError::PathError(format!("Failed to canonicalize root path: {}", e)))?;

    if canonical_root.is_file() {
        let info = get_file_info(&canonical_root, &canonical_root)?;
        return Ok(Json(vec![info]));
    }

    let target_path = resolve_safe_path(&state.root_path, params.path.as_deref())?;

    if target_path.is_file() {
        let info = get_file_info(&canonical_root, &target_path)?;
        Ok(Json(vec![info]))
    } else {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&target_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if let Ok(info) = get_file_info(&canonical_root, &path) {
                entries.push(info);
            }
        }

        // Sort: directories first, then alphabetically by name
        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.cmp(&b.name)
            }
        });

        Ok(Json(entries))
    }
}

/// Handler to stream file content for download.
async fn download_file_handler(
    State(state): State<AppState>,
    Query(params): Query<FileParams>,
) -> Result<impl IntoResponse, FsvError> {
    let canonical_root = state
        .root_path
        .canonicalize()
        .map_err(|e| FsvError::PathError(format!("Failed to canonicalize root path: {}", e)))?;

    let target_path = if canonical_root.is_file() {
        canonical_root
    } else {
        resolve_safe_path(&state.root_path, params.path.as_deref())?
    };

    if !target_path.is_file() {
        return Err(FsvError::NotAFile);
    }

    let file = tokio::fs::File::open(&target_path).await?;
    let metadata = file.metadata().await?;
    let file_name = target_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".to_string());

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let content_disposition = format!("attachment; filename=\"{}\"", file_name);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/octet-stream"),
    );
    if let Ok(cd_val) = axum::http::HeaderValue::from_str(&content_disposition) {
        headers.insert(header::CONTENT_DISPOSITION, cd_val);
    }
    if let Ok(len_val) = axum::http::HeaderValue::from_str(&metadata.len().to_string()) {
        headers.insert(header::CONTENT_LENGTH, len_val);
    }

    Ok((headers, body))
}

/// WebSocket route handler.
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Logic for managing a single WebSocket client connection.
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    loop {
        tokio::select! {
            broadcast_msg = rx.recv() => {
                match broadcast_msg {
                    Ok(msg) => {
                        if sender.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            client_msg = receiver.next() => {
                match client_msg {
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Ok(_)) => {
                        // Client sent something, ignore it as we only do server -> client broadcasts
                    }
                    Some(Err(_)) | None => {
                        break;
                    }
                }
            }
        }
    }
}

/// Handler to serve the bundled frontend UI as the default page.
async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../dist/index.html"))
}

/// Runs the fsv server with the provided configuration in a background thread.
/// Returns a list of local IP addresses, the bound port, and a ServerHandle on success.
pub async fn run(config: Config) -> Result<(Vec<IpAddr>, u16, ServerHandle), FsvError> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound_port = listener.local_addr()?.port();

    let ips = get_local_ips();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (ws_tx, _) = broadcast::channel::<String>(100);

    let state = AppState {
        root_path: config.path.clone(),
        ws_tx: ws_tx.clone(),
    };

    let app = axum::Router::new()
        .route("/", axum::routing::get(index_handler))
        .route("/api/files", axum::routing::get(list_files_handler))
        .route("/api/download", axum::routing::get(download_file_handler))
        .route("/ws", axum::routing::get(ws_handler))
        .with_state(state);

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    let handle = ServerHandle {
        shutdown_tx: Some(shutdown_tx),
        ws_tx,
    };

    Ok((ips, bound_port, handle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::connect_async;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_fsv_websocket_and_api() {
        // Create temporary directory for tests
        let temp_dir = std::env::temp_dir().join("fsv_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("test.txt");
        std::fs::write(&test_file, b"Hello World").unwrap();

        // Start server on a random port (port 0 selects a free port)
        let config = Config {
            path: temp_dir.clone(),
            port: 0,
        };

        let (_ips, port, mut handle) = run(config).await.unwrap();

        // Connect to WS
        let ws_url = format!("ws://127.0.0.1:{}/ws", port);
        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (_, mut rx) = ws_stream.split();

        // Send a message from server handle
        let broadcast_msg = "Hello WebSocket Client!";
        // Wait a bit to ensure subscription completes in background
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let count = handle.send_message(broadcast_msg).unwrap();
        assert_eq!(count, 1);

        // Receive message from WS client
        let msg = rx.next().await.unwrap().unwrap();
        assert_eq!(msg.to_text().unwrap(), broadcast_msg);

        // Stop server
        handle.shutdown().unwrap();

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
