use clap::Parser;
use std::io::{BufRead, Write};
use std::path::PathBuf;

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

    let (ips, port, mut handle) = fsv::run(fsv::Config {
        path: args.path,
        port: args.port,
    })
    .await?;

    // Print all access URLs.
    println!("fsv {}", serving_path);
    for ip in &ips {
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
    // Press Ctrl-C to exit and shut down the server.
    println!("Enter to broadcast, Ctrl+C to quit");

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    loop {
        {
            let mut out = stdout.lock();
            out.write_all("❯ ".as_bytes()).ok();
            out.flush().ok();
        }

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => break, // EOF (Ctrl-D on Unix) or error
            Ok(_) => {}
        }

        let text = line.trim();
        if text.is_empty() {
            continue;
        }

        match handle.send(text) {
            Ok(n) => println!("  ✓ broadcast to {n} client(s)"),
            Err(e) => eprintln!("  ✗ broadcast error: {e}"),
        }
    }

    println!();
    handle.shutdown().ok();
    println!("  ✓ server shut down");
    Ok(())
}
