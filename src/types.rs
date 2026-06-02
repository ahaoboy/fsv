use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, Notify};

use crate::error::FsvError;

/// Server startup configuration.
pub struct Config {
    pub path: PathBuf,
    pub port: u16,
}

/// Information about the running server, returned by `run`.
pub struct Server {
    pub ips: Vec<IpAddr>,
    pub port: u16,
    pub(crate) shutdown_tx: Option<oneshot::Sender<()>>,
    pub(crate) ws_tx: broadcast::Sender<String>,
    pub shutdown_notify: Arc<Notify>,
}

impl Server {
    /// Gracefully shuts down the server.
    pub fn shutdown(&mut self) -> Result<(), FsvError> {
        self.shutdown_tx
            .take()
            .ok_or_else(|| {
                FsvError::Shutdown("Server already shut down or handle uninitialized".into())
            })?
            .send(())
            .map_err(|_| FsvError::Shutdown("Failed to send shutdown signal".into()))
    }

    /// Broadcasts a message to all connected WebSocket clients.
    /// Returns the number of active receivers.
    pub fn send(&self, message: &str) -> Result<usize, FsvError> {
        match self.ws_tx.send(message.to_string()) {
            Ok(count) => Ok(count),
            // No receivers is not an error — just zero clients connected.
            Err(_) => Ok(0),
        }
    }
}

/// Shared application state injected into every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub root_path: PathBuf,
    pub ws_tx: broadcast::Sender<String>,
    pub ws_connections: Arc<AtomicUsize>,
    pub shutdown_notify: Arc<Notify>,
}

/// Metadata for a single file or directory entry.
#[derive(Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

/// Query parameters accepted by the files and download endpoints.
#[derive(Deserialize, Default)]
pub struct FileParams {
    pub path: Option<String>,
}
