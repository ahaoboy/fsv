# fsv (File Share Viewer)

<p align="center">
  <img src="public/icon.ico" width="128" alt="fsv logo" />
</p>

`fsv` is a fast, lightweight command-line tool built in Rust with a React + MUI frontend. Share a local file or directory over HTTP, with real-time WebSocket broadcasting and a modern mobile-friendly web UI.

## Features

- **Instant File Sharing**: Serve local files or entire directories over HTTP.
- **Modern Web UI**: Mobile-first React frontend with Material UI, automatic dark/light theme, and hash-based navigation. The entire UI is bundled into a single standalone HTML file.
- **Real-time Broadcasting**: Push messages to all connected web clients via WebSocket from the CLI.
- **Auto-open Browser**: Automatically opens the web UI in your default browser on startup.
- **Hash-route URLs**: Shareable URLs like `http://host:port/#/path/to/folder` for direct folder navigation.
- **File Preview**: Built-in preview for text, images, video, and audio files.
- **QR Code Sharing**: Generate QR codes for file download links.
- **Secure**: Directory traversal protection and safe path resolution.
- **Standalone Binary & Library**: Run as a CLI tool or integrate into other Rust applications.

## Building from Source

Requires Rust and Node.js (with `pnpm`).

```bash
# 1. Build the frontend
pnpm install
pnpm run build

# 2. Build the Rust backend
cargo build --release
```

## Usage

```bash
# Share the current directory (default port 8888)
fsv .

# Share a specific directory on a custom port
fsv /path/to/share -p 8080
```

On startup, `fsv` prints all access URLs, displays a QR code, and automatically opens the web UI in your browser.

### Interactive Broadcast

While the server is running, type any message and press Enter to broadcast it to all connected WebSocket clients:

```
❯ Hello everyone!
  ✓ broadcast to 3 client(s)
```

Press `Ctrl+C` to stop the server.

## Using as a Library

Disable the `cli` feature to drop CLI-specific dependencies:

```toml
[dependencies]
fsv = { path = "path/to/fsv", default-features = false }
```

```rust
use fsv::{Config, run};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let config = Config {
        path: PathBuf::from("."),
        port: 8080,
    };

    let (ips, port, mut handle) = run(config).await.unwrap();
    println!("Server running on port: {}", port);

    // Broadcast a message to all connected clients
    handle.send("Hello from the library!").unwrap();
}
```

## Related Projects

- **[fsv-tauri](https://github.com/ahaoboy/fsv-tauri)** — Desktop GUI version of fsv built with Tauri, offering the same file sharing and WebSocket broadcasting with a native window experience.

## License

MIT
