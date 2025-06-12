mod cli;
mod capture;
mod bandwidth;
mod ui;

use anyhow::{Context, Result};
use cli::Args;
use capture::PacketCapture;
use bandwidth::start_bandwidth_monitor;
use ui::{App, run_ui};
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse_args();
    
    validate_args(&args)?;
    
    println!("Starting tcpgraph...");
    println!("Interface: {}", args.interface);
    println!("Filter: {}", args.filter);
    println!("Update interval: {}s", args.interval);
    
    if let Some(duration) = args.duration {
        println!("Duration: {}s", duration);
    }

    let packet_capture = PacketCapture::new(args.interface.clone(), args.filter.clone());
    
    let packet_rx = packet_capture.start_capture().await
        .context("Failed to start packet capture")?;
    
    let update_interval = Duration::from_secs(args.interval);
    let bandwidth_rx = start_bandwidth_monitor(packet_rx, update_interval).await;
    
    let app = App::new(args.interface, args.filter);
    
    tokio::select! {
        result = tokio::task::spawn_blocking(move || run_ui(app, bandwidth_rx, update_interval)) => {
            result??;
        }
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        }
    }

    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if args.interface.is_empty() {
        anyhow::bail!("Interface name cannot be empty");
    }
    
    if args.filter.is_empty() {
        anyhow::bail!("Filter expression cannot be empty");
    }
    
    if args.interval == 0 {
        anyhow::bail!("Update interval must be greater than 0");
    }
    
    if let Some(duration) = args.duration {
        if duration == 0 {
            anyhow::bail!("Duration must be greater than 0");
        }
    }
    
    Ok(())
}
