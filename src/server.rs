use std::net::IpAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot};
use tower_http::cors::{Any, CorsLayer};

use crate::error::FsvError;
use crate::handlers::{file, health, index, list, ws_info};
use crate::types::{AppState, Config, ServerHandle};
use crate::util::get_local_ips;
use crate::ws::ws_handler;

/// Starts the fsv HTTP server and returns the bound addresses, port, and a control handle.
pub async fn run(config: Config) -> Result<(Vec<IpAddr>, u16, ServerHandle), FsvError> {
    let port = find_port::find_port("127.0.0.1", config.port).expect("can't find available port");

    let listener =
        tokio::net::TcpListener::bind(std::net::SocketAddr::from(([0, 0, 0, 0], port)))
            .await?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (ws_tx, _) = broadcast::channel::<String>(100);
    let ws_connections = Arc::new(AtomicUsize::new(0));

    let state = AppState {
        root_path: config.path,
        ws_tx: ws_tx.clone(),
        ws_connections: ws_connections.clone(),
    };

    // Configure CORS to allow all origins, methods, and headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .route("/", axum::routing::get(index))
        .route("/api/list", axum::routing::get(list))
        .route("/api/file", axum::routing::get(file))
        .route("/api/ws-info", axum::routing::get(ws_info))
        .route("/api/health", axum::routing::get(health))
        .route("/ws", axum::routing::get(ws_handler))
        .layer(cors)
        .with_state(state);

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    Ok((
        get_local_ips(),
        port,
        ServerHandle {
            shutdown_tx: Some(shutdown_tx),
            ws_tx,
        },
    ))
}
