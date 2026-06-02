use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, Notify};
use tower_http::cors::{Any, CorsLayer};

use crate::error::FsvError;
use crate::types::{AppState, Config, Server};
use crate::unified::unified_handler;
use crate::util::get_local_ips;
use crate::ws::ws_handler;

/// Starts the fsv HTTP server and returns server info including a control handle.
pub async fn run(config: Config) -> Result<Server, FsvError> {
    let port = find_port::find_port("127.0.0.1", config.port).expect("can't find available port");

    let listener =
        tokio::net::TcpListener::bind(std::net::SocketAddr::from(([0, 0, 0, 0], port)))
            .await?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (ws_tx, _) = broadcast::channel::<String>(100);
    let ws_connections = Arc::new(AtomicUsize::new(0));
    let shutdown_notify = Arc::new(Notify::new());

    let state = AppState {
        root_path: config.path,
        ws_tx: ws_tx.clone(),
        ws_connections: ws_connections.clone(),
        shutdown_notify: shutdown_notify.clone(),
    };

    // Configure CORS to allow all origins, methods, and headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Unified routing: all paths use the same handler
    // HTTP method determines behavior:
    // - GET: HTML (root) or file download
    // - POST: API calls
    // - PROPFIND/OPTIONS/HEAD: WebDAV protocol
    // Exception: /ws for WebSocket upgrade
    let app = axum::Router::new()
        .route("/ws", axum::routing::get(ws_handler))
        .route("/", axum::routing::any(unified_handler))
        .route("/{*path}", axum::routing::any(unified_handler))
        .layer(cors)
        .with_state(state);

    let shutdown_notify_for_server = shutdown_notify.clone();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                tokio::select! {
                    _ = shutdown_rx => {},
                    _ = shutdown_notify.notified() => {},
                }
            })
            .await
            .unwrap();
    });

    Ok(Server {
        ips: get_local_ips(),
        port,
        shutdown_tx: Some(shutdown_tx),
        ws_tx,
        shutdown_notify: shutdown_notify_for_server,
    })
}
