//! Audio Sync Share - Sender Application
//! 
//! Captures system audio and streams it to receiver devices over the network.

use audio_sync_share::config::{Config, CaptureMode};
use audio_sync_share::audio_capture::{AudioCapture, AudioSample};
use audio_sync_share::network::{AudioStreamer, ServiceDiscovery, ControlMessage, get_local_ips};
use audio_sync_share::error::Result;
use clap::Parser;
use log::{info, warn, error};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "sender")]
#[command(about = "Audio Sync Share - Sender (streams audio to receivers)", long_about = None)]
struct Args {
    /// Device name for identification
    #[arg(short, long, default_value = "sender")]
    name: String,

    /// Audio port for streaming
    #[arg(short, long, default_value_t = 50000)]
    audio_port: u16,

    /// Control port for commands
    #[arg(short, long, default_value_t = 50001)]
    control_port: u16,

    /// Target latency in milliseconds
    #[arg(long, default_value_t = 50)]
    latency_ms: u32,

    /// Sample rate in Hz
    #[arg(long, default_value_t = 48000)]
    sample_rate: u32,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Capture from specific application (experimental)
    #[arg(long)]
    app: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    info!("🎵 Audio Sync Share - Sender");
    info!("==============================");
    info!("Device name: {}", args.name);
    info!("Audio port: {}", args.audio_port);
    info!("Control port: {}", args.control_port);
    info!("Target latency: {}ms", args.latency_ms);
    info!("Sample rate: {}Hz", args.sample_rate);

    // Create configuration
    let mut config = Config::new(&args.name);
    config.network.audio_port = args.audio_port;
    config.network.control_port = args.control_port;
    config.sync.target_latency_ms = args.latency_ms;
    config.audio.sample_rate = args.sample_rate;
    config.verbose = args.verbose;
    config.capture_mode = if let Some(app) = &args.app {
        CaptureMode::Application(app.clone())
    } else {
        CaptureMode::Global
    };

    // Show local IP addresses
    let ips = get_local_ips();
    info!("Local IP addresses:");
    for ip in &ips {
        info!("  - {}", ip);
    }

    // Initialize service discovery
    let discovery = Arc::new(ServiceDiscovery::new()?);
    
    // Register service for discovery
    let properties = vec![
        ("device_name".to_string(), args.name.clone()),
        ("audio_port".to_string(), args.audio_port.to_string()),
        ("control_port".to_string(), args.control_port.to_string()),
    ];
    discovery.register(&args.name, args.audio_port, properties)?;
    info!("Service registered for mDNS discovery");

    // Initialize audio capture
    info!("Initializing audio capture...");
    let mut capture = AudioCapture::new(&config.audio, config.capture_mode.clone())?;
    info!("Audio capture initialized");

    // Initialize network streamer
    info!("Initializing network streamer...");
    let mut streamer = AudioStreamer::new(&config.network).await?;
    info!("Network streamer initialized");

    // Channel for peer management
    let peers = Arc::new(Mutex::new(Vec::new()));
    let peers_clone = peers.clone();

    // Task to handle control messages
    let control_socket = streamer.control_socket.clone();
    let peers_for_control = peers.clone();
    let name_for_control = args.name.clone();
    
    tokio::spawn(async move {
        loop {
            match streamer.recv_control().await {
                Ok((msg, addr)) => {
                    match msg {
                        ControlMessage::DeviceInfo { name, .. } => {
                            info!("Discovered device: {} at {}", name, addr);
                            let mut p = peers_for_control.lock().await;
                            if !p.contains(&addr) {
                                p.push(addr);
                            }
                        }
                        ControlMessage::Ping { timestamp } => {
                            // Respond with pong
                            let pong = ControlMessage::Pong { 
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64
                            };
                            let _ = streamer.send_control(&pong, addr).await;
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    warn!("Control message error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    });

    // Start audio capture
    info!("Starting audio capture...");
    capture.start()?;
    info!("✅ Audio capture started");
    info!("✅ Ready to stream! Waiting for receivers...");
    info!("");
    info!("Connect receivers using:");
    for ip in &ips {
        info!("  receiver --host {}", ip);
    }
    info!("");
    info!("Or let them discover this device automatically via mDNS");

    // Main streaming loop
    let mut frame_count: u64 = 0;
    let mut last_stats = std::time::Instant::now();
    let mut total_samples: usize = 0;

    loop {
        // Read audio samples from capture buffer
        let samples = capture.read_samples(config.audio.buffer_size * config.audio.channels as usize);
        
        if !samples.is_empty() {
            // Send to all connected peers
            if let Err(e) = streamer.send_audio(
                &samples,
                config.audio.channels,
                config.audio.sample_rate,
            ).await {
                warn!("Failed to send audio: {}", e);
            }

            frame_count += 1;
            total_samples += samples.len();

            // Print stats every 5 seconds
            if last_stats.elapsed() >= Duration::from_secs(5) {
                let peers_list = peers.lock().await;
                let kbps = (total_samples * 2 * 8) as f64 / last_stats.elapsed().as_secs_f64() / 1000.0;
                info!("📊 Stats: {} frames, {} peers, {:.1} kbps", 
                      frame_count, peers_list.len(), kbps);
                total_samples = 0;
                frame_count = 0;
                last_stats = std::time::Instant::now();
            }
        }

        // Small delay to prevent CPU spinning
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
}
