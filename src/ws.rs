use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};

use crate::types::AppState;

/// Upgrades an HTTP connection to a WebSocket and hands it off to `handle_socket`.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("WebSocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Manages a single WebSocket client: forwards server broadcasts and handles disconnects.
async fn handle_socket(socket: WebSocket, state: AppState) {
    // Increment connection count
    let prev = state
        .ws_connections
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    tracing::info!(connections = prev + 1, "WebSocket client connected");

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        tracing::trace!(message = %text, "forwarding broadcast to WS client");
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            tracing::debug!("WS send failed, client disconnected");
                            break;
                        }
                    }
                    // Lagged behind — skip missed messages and keep going.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "WS client lagged, skipped messages");
                        continue;
                    }
                    Err(_) => break,
                }
            }
            client_msg = receiver.next() => {
                match client_msg {
                    // Client closed the connection cleanly.
                    Some(Ok(Message::Close(_))) => {
                        tracing::debug!("WS client sent close frame");
                        break;
                    }
                    // Ignore any other client-sent messages (server-push only).
                    Some(Ok(_)) => {}
                    // Error or stream ended.
                    Some(Err(e)) => {
                        tracing::debug!(error = %e, "WS client error");
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    // Decrement connection count
    let prev = state
        .ws_connections
        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    tracing::info!(connections = prev - 1, "WebSocket client disconnected");
}
