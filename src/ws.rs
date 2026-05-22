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
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Manages a single WebSocket client: forwards server broadcasts and handles disconnects.
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    // Lagged behind — skip missed messages and keep going.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            client_msg = receiver.next() => {
                match client_msg {
                    // Client closed the connection cleanly.
                    Some(Ok(Message::Close(_))) => break,
                    // Ignore any other client-sent messages (server-push only).
                    Some(Ok(_)) => {}
                    // Error or stream ended.
                    Some(Err(_)) | None => break,
                }
            }
        }
    }
}
