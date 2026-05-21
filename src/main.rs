use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

/// CLI arguments for fsv application.
#[derive(Parser, Debug)]
#[command(name = "fsv")]
#[command(author, version, about = "fsv: File Share & WebSocket Broadcaster CLI", long_about = None)]
struct Args {
    /// Path of the file or folder to share
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Port to host the web API
    #[arg(short = 'p', long = "port", default_value_t = 8888)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Verify path exists before starting
    if !args.path.exists() {
        eprintln!("Error: The specified path '{}' does not exist.", args.path.display());
        std::process::exit(1);
    }

    let config = fsv::Config {
        path: args.path,
        port: args.port,
    };

    println!("Starting fsv server...");
    let (ips, port, mut handle) = fsv::run(config).await?;

    println!("\nFSV Server running successfully!");
    println!("Web UI and API access links (host:port):");
    for ip in &ips {
        if ip.is_ipv6() {
            println!("  http://[{}]:{}      (Web UI)", ip, port);
            println!("  http://[{}]:{}/api/files", ip, port);
        } else {
            println!("  http://{}:{}        (Web UI)", ip, port);
            println!("  http://{}:{}/api/files", ip, port);
        }
    }

    // Suggest the websocket endpoint URL
    let ws_url = if ips.iter().any(|ip| ip.is_ipv4() && !ip.is_loopback()) {
        let first_external = ips.iter().find(|ip| ip.is_ipv4() && !ip.is_loopback()).unwrap();
        format!("ws://{}:{}/ws", first_external, port)
    } else {
        format!("ws://127.0.0.1:{}/ws", port)
    };
    println!("WebSocket endpoint: {}", ws_url);

    println!("\nControl Shell Commands:");
    println!("  broadcast <message>  - Send a message to all connected WebSocket clients");
    println!("  stop                 - Gracefully shut down the server and exit");
    println!("  help                 - Show available commands");

    // Read commands from stdin
    let mut stdin_reader = tokio::io::BufReader::new(tokio::io::stdin());
    use tokio::io::AsyncBufReadExt;
    let mut line = String::new();

    loop {
        print!("fsv> ");
        io::stdout().flush()?;
        line.clear();

        if stdin_reader.read_line(&mut line).await? == 0 {
            // EOF reached
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "stop" || trimmed == "exit" {
            println!("Shutting down the web server...");
            handle.shutdown()?;
            println!("Web server stopped. Exiting.");
            break;
        } else if trimmed == "help" {
            println!("Commands:");
            println!("  broadcast <message>  - Send a message to all connected WebSocket clients");
            println!("  stop                 - Gracefully shut down the server and exit");
        } else if let Some(stripped) = trimmed.strip_prefix("broadcast ") {
            let msg = stripped.trim();
            if msg.is_empty() {
                println!("Error: Broadcast message content cannot be empty.");
                continue;
            }
            match handle.send_message(msg) {
                Ok(count) => {
                    println!("Broadcasted to {} WebSocket client(s).", count);
                }
                Err(e) => {
                    println!("Broadcast failed: {}", e);
                }
            }
        } else {
            println!("Unknown command: '{}'. Type 'help' or 'stop' to exit.", trimmed);
        }
    }

    Ok(())
}
