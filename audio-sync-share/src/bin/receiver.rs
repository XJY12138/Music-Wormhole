//! Audio Sync Share - Receiver Application
//! 
//! Receives audio streams from sender devices and plays them back in sync.

use audio_sync_share::config::Config;
use audio_sync_share::audio_player::{AudioPlayer, SyncInfo};
use audio_sync_share::network::{AudioReceiver, ServiceDiscovery, ControlMessage, DiscoveredDevice};
use audio_sync_share::sync::{AudioSynchronizer, PlaybackTimer};
use audio_sync_share::error::Result;
use clap::Parser;
use log::{info, warn, error, debug};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "receiver")]
#[command(about = "Audio Sync Share - Receiver (plays audio from sender)", long_about = None)]
struct Args {
    /// Sender hostname or IP address (optional, can use discovery)
    #[arg(short, long)]
    host: Option<String>,

    /// Audio port for receiving
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

    /// Auto-discover senders (don't specify host)
    #[arg(long)]
    discover: bool,

    /// Device name for this receiver
    #[arg(short, long, default_value = "receiver")]
    name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    info!("🎵 Audio Sync Share - Receiver");
    info!("================================");
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

    // Initialize service discovery
    let discovery = Arc::new(ServiceDiscovery::new()?);
    
    // Find or wait for sender
    let sender_addr = if let Some(host) = &args.host {
        // Parse provided host
        info!("Connecting to specified host: {}", host);
        
        // Try to parse as IP first
        if let Ok(ip) = IpAddr::from_str(host) {
            SocketAddr::new(ip, args.audio_port)
        } else {
            // Resolve hostname (simplified - in production use tokio::net::lookup_host)
            warn!("Hostname resolution not fully implemented, trying as IP");
            return Err(audio_sync_share::error::Error::Network(
                format!("Could not resolve hostname: {}", host)
            ));
        }
    } else if args.discover {
        // Use mDNS discovery
        info!("Discovering senders via mDNS...");
        info!("Waiting for sender announcement (timeout: 30s)...");
        
        let mut rx = discovery.browse();
        let timeout = Duration::from_secs(30);
        let start = Instant::now();
        
        loop {
            if Instant::now() - start > timeout {
                return Err(audio_sync_share::error::Error::Network(
                    "No sender discovered within timeout".to_string()
                ));
            }
            
            match tokio::time::timeout(Duration::from_secs(1), rx.recv()).await {
                Ok(Some(device)) => {
                    info!("✅ Discovered sender: {} at {}", device.name, device.address);
                    break SocketAddr::new(device.address.ip(), device.audio_port);
                }
                Ok(None) | Err(_) => {
                    debug!("Still searching...");
                }
            }
        }
    } else {
        return Err(audio_sync_share::error::Error::Network(
            "No host specified and discovery not enabled. Use --host <ip> or --discover".to_string()
        ));
    };

    // Initialize audio player
    info!("Initializing audio player...");
    let mut player = AudioPlayer::new(&config.audio, config.sync.target_latency_ms)?;
    info!("Audio player initialized");

    // Initialize network receiver
    info!("Initializing network receiver...");
    let mut receiver = AudioReceiver::new(&config.network).await?;
    info!("Network receiver initialized");

    // Initialize synchronizer
    let mut synchronizer = AudioSynchronizer::new(&config.sync);
    let mut playback_timer = PlaybackTimer::new(config.audio.sample_rate, config.audio.channels);

    // Send device info to sender
    let control_msg = ControlMessage::DeviceInfo {
        name: args.name.clone(),
        ip: "0.0.0.0".to_string(), // Simplified
    };
    let control_addr = SocketAddr::new(sender_addr.ip(), args.control_port);
    if let Err(e) = receiver.send_control(&control_msg, control_addr).await {
        warn!("Failed to send device info: {}", e);
    }

    // Start audio playback
    info!("Starting audio playback...");
    player.start()?;
    info!("✅ Audio playback started");
    info!("✅ Waiting for audio stream from {}...", sender_addr);
    info!("");

    // Main receive loop
    let mut packet_count: u64 = 0;
    let mut last_stats = Instant::now();
    let mut total_samples: usize = 0;
    let mut is_playing = false;

    // Start ping-pong for synchronization
    let sync_interval = Duration::from_millis(1000);
    let mut last_sync = Instant::now();

    loop {
        // Perform periodic synchronization
        if Instant::now() - last_sync > sync_interval {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            let ping = ControlMessage::Ping { timestamp: now };
            if let Err(e) = receiver.send_control(&ping, control_addr).await {
                debug!("Sync ping failed: {}", e);
            }
            last_sync = Instant::now();
        }

        // Receive audio packet
        match tokio::time::timeout(Duration::from_millis(100), receiver.recv_audio()).await {
            Ok(Ok((header, samples))) => {
                packet_count += 1;
                total_samples += samples.len();

                // Record timing for synchronization
                let recv_time = Instant::now();
                synchronizer.record_timing(recv_time - Duration::from_millis(10), header.timestamp, recv_time);

                // Update buffer level tracking
                let buffer_ms = player.buffer_level_ms();
                synchronizer.update_buffer_level(buffer_ms);

                // Check if we have enough data to start playing
                if !is_playing && player.is_ready() {
                    player.set_playing();
                    playback_timer.start();
                    is_playing = true;
                    info!("🎶 Playback synchronized and playing!");
                }

                // Write samples to player buffer
                let written = player.write_samples(&samples);
                
                if written < samples.len() {
                    debug!("Buffer overflow: dropped {} samples", samples.len() - written);
                }

                // Adjust timing if needed
                if synchronizer.is_synced() && is_playing {
                    let drift = playback_timer.drift_ms() as i32;
                    if drift.abs() > config.sync.max_drift_ms as i32 {
                        debug!("Adjusting timing for {}ms drift", drift);
                        player.adjust_timing(drift / 2);
                    }
                }
            }
            Ok(Err(e)) => {
                warn!("Receive error: {}", e);
            }
            Err(_) => {
                // Timeout - no data received
                if packet_count > 0 {
                    debug!("No audio data received (timeout)");
                }
            }
        }

        // Print stats every 5 seconds
        if last_stats.elapsed() >= Duration::from_secs(5) {
            let stats = synchronizer.stats();
            let (recv, lost, loss_rate) = receiver.stats();
            let kbps = (total_samples * 2 * 8) as f64 / last_stats.elapsed().as_secs_f64() / 1000.0;
            
            info!("📊 Stats: {} packets, buffer={}ms, sync={:?}, loss={:.2}%, {:.1} kbps",
                  packet_count, 
                  player.buffer_level_ms(),
                  stats.state,
                  loss_rate,
                  kbps);
            
            if stats.success_rate > 0.0 {
                debug!("  Sync: offset={:.2}ms, drift={:.2}ppm, success={:.1}%",
                       stats.clock_offset_ms, stats.clock_drift_ppm, stats.success_rate * 100.0);
            }

            total_samples = 0;
            packet_count = 0;
            last_stats = Instant::now();
        }
    }
}
