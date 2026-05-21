# fsv (File Share Viewer)

`fsv` is a fast, lightweight, and modern command-line application built in Rust and Preact. It allows you to quickly share a local file or directory over HTTP and interact with connected clients via WebSockets in real-time.

## Features

- **Static File Serving & Downloads**: Share local files or entire directories instantly.
- **Built-in Modern Web UI**: Features a beautiful glassmorphic frontend built with Preact and Vite. The entire UI is bundled into a single standalone HTML file inside the binary.
- **WebSocket Broadcasting**: Push real-time messages to all connected Web UI clients directly from the CLI control shell.
- **Secure File Access**: Includes built-in protection against directory traversal attacks.
- **Standalone Binary & Library**: Can be run as a CLI tool or integrated into other Rust applications as a library.

## Building from Source

Ensure you have Rust and Node.js (with `npm`) installed.

1. First, build the frontend Web UI:
   ```bash
   npm run build
   ```
   *This compiles the Preact application in the `ui/` directory and generates a single `dist/index.html` file.*

2. Build the Rust backend server:
   ```bash
   cargo build --release
   ```

## Usage

Start the server by specifying the path you want to share. By default, it will host on port `8888`.

```bash
# Share the current directory
fsv .

# Share a specific directory on a custom port
fsv /path/to/share -p 8080
```

Once started, `fsv` will print the available access links. You can open any of the Web UI links in your browser to view the files.

### Interactive Control Shell

While the server is running, you can use the built-in command prompt (`fsv>`) to control the application:

- `broadcast <message>`: Sends a real-time message to all active WebSocket clients. The message will appear instantly on the Web UI feed.
- `stop` (or `exit`): Gracefully shuts down the server.
- `help`: Displays available commands.

## Using as a Library

You can easily integrate the core server logic into your own Rust projects by disabling the `cli` feature to drop CLI-specific dependencies (like `clap`).

Add the following to your `Cargo.toml`:

```toml
[dependencies]
fsv = { path = "path/to/fsv", default-features = false }
```

Then, you can programmatically start the server:

```rust
use fsv::{Config, run};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let config = Config {
        path: PathBuf::from("."),
        port: 8080,
    };
    
    // Start the server in the background
    let (ips, port, handle) = run(config).await.unwrap();
    println!("Server running on port: {}", port);
    
    // Broadcast a message programmatically
    handle.send_message("Hello from the library code!").unwrap();
}
```

## License

This project is licensed under the MIT License.
