use clap::Parser;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// fsv: share a file or folder over HTTP with a live WebSocket broadcast channel.
#[derive(Parser, Debug)]
#[command(name = "fsv", author, version, about, long_about = None)]
struct Args {
    /// File or directory to share
    #[arg(value_name = "PATH", default_value = ".")]
    path: PathBuf,

    /// Port to listen on (0 = OS-assigned)
    #[arg(short = 'p', long, default_value_t = 8888)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.path.exists() {
        eprintln!("error: path '{}' does not exist", args.path.display());
        std::process::exit(1);
    }

    let serving_path = args.path.display().to_string();

    let mut info = fsv::run(fsv::Config {
        path: args.path,
        port: args.port,
    })
    .await?;
    let ips = &info.ips;
    let port = info.port;

    // Print all access URLs.
    println!("fsv {}", serving_path);
    for ip in ips {
        let url = if ip.is_ipv6() {
            format!("http://[{ip}]:{port}")
        } else {
            format!("http://{ip}:{port}")
        };
        println!("  {url}");
    }

    // Display a QR code for the first non-loopback IPv4 address (or loopback).
    let primary = ips
        .iter()
        .find(|ip| ip.is_ipv4() && !ip.is_loopback())
        .or_else(|| ips.first())
        .copied();

    if let Some(ip) = primary {
        let url = format!("http://{}:{}", ip, port);
        qr2term::print_qr(&url).unwrap_or_else(|e| eprintln!("QR error: {e}"));
        // Automatically open the URL in the default browser
        if let Err(e) = open::that(&url) {
            eprintln!("Failed to open browser: {e}");
        }
    }

    // Interactive prompt: each line is broadcast to all WebSocket clients.
    // Press Ctrl-C to exit, or use the web UI shutdown button.
    println!("Enter to broadcast, Ctrl+C to quit");

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();
    let shutdown_notify = info.shutdown_notify.clone();
    let mut api_shutdown = false;

    loop {
        // Print prompt
        stdout.write_all(b"\xe2\x9d\xaf ").await.ok();
        stdout.flush().await.ok();

        tokio::select! {
            result = lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let text = line.trim();
                        if text.is_empty() {
                            continue;
                        }
                        match info.send(text) {
                            Ok(n) => println!("  \u{2713} broadcast to {n} client(s)"),
                            Err(e) => eprintln!("  \u{2717} broadcast error: {e}"),
                        }
                    }
                    _ => break, // EOF or error
                }
            }
            _ = shutdown_notify.notified() => {
                api_shutdown = true;
                break;
            }
        }
    }

    println!();
    if !api_shutdown {
        info.shutdown().ok();
    }
    println!("  \u{2713} server shut down");

    if api_shutdown {
        // API-triggered shutdown: force exit to avoid waiting on background tasks.
        std::process::exit(0);
    }
    Ok(())
}
