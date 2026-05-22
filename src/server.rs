use std::net::IpAddr;
use tokio::sync::{broadcast, oneshot};

use crate::error::FsvError;
use crate::handlers::{file, index, list};
use crate::types::{AppState, Config, ServerHandle};
use crate::util::get_local_ips;
use crate::ws::ws_handler;

/// Starts the fsv HTTP server and returns the bound addresses, port, and a control handle.
pub async fn run(config: Config) -> Result<(Vec<IpAddr>, u16, ServerHandle), FsvError> {
    let listener =
        tokio::net::TcpListener::bind(std::net::SocketAddr::from(([0, 0, 0, 0], config.port)))
            .await?;
    let port = listener.local_addr()?.port();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (ws_tx, _) = broadcast::channel::<String>(100);

    let state = AppState {
        root_path: config.path,
        ws_tx: ws_tx.clone(),
    };

    let app = axum::Router::new()
        .route("/", axum::routing::get(index))
        .route("/api/list", axum::routing::get(list))
        .route("/api/file", axum::routing::get(file))
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

    Ok((
        get_local_ips(),
        port,
        ServerHandle {
            shutdown_tx: Some(shutdown_tx),
            ws_tx,
        },
    ))
}
