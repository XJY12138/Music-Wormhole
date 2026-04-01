//! Network module for audio streaming and device discovery
//! 
//! Features:
//! - UDP-based low-latency audio streaming
//! - mDNS service discovery for automatic device detection
//! - Control channel for playback commands
//! - Sequence numbering for packet ordering

use crate::config::{Config, NetworkConfig};
use crate::error::{Error, Result};
use bincode;
use log::{debug, error, info, warn};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

/// Audio packet header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPacketHeader {
    /// Sequence number for ordering and loss detection
    pub sequence: u64,
    /// Timestamp from sender (milliseconds since epoch)
    pub timestamp: u64,
    /// Number of audio samples in payload
    pub sample_count: u16,
    /// Channel count
    pub channels: u16,
    /// Sample rate
    pub sample_rate: u32,
}

/// Control message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Start playback at specified time
    Play { timestamp: u64 },
    /// Pause playback
    Pause,
    /// Resume playback
    Resume,
    /// Stop playback
    Stop,
    /// Adjust volume (0.0 to 1.0)
    SetVolume(f32),
    /// Synchronization ping
    Ping { timestamp: u64 },
    /// Synchronization pong
    Pong { timestamp: u64 },
    /// Device information
    DeviceInfo { name: String, ip: String },
    /// Error message
    Error(String),
}

/// Discovered device information
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub name: String,
    pub address: SocketAddr,
    pub audio_port: u16,
    pub control_port: u16,
    pub discovered_at: Instant,
}

/// Network streamer for sending audio
pub struct AudioStreamer {
    config: NetworkConfig,
    audio_socket: Arc<UdpSocket>,
    control_socket: Arc<UdpSocket>,
    sequence: u64,
    peers: Vec<SocketAddr>,
}

impl AudioStreamer {
    /// Create a new audio streamer (sender mode)
    pub async fn new(config: &NetworkConfig) -> Result<Self> {
        let audio_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.audio_port);
        let control_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.control_port);

        let audio_socket = UdpSocket::bind(audio_addr).await?;
        let control_socket = UdpSocket::bind(control_addr).await?;

        info!("Audio streamer bound to {}:{}", audio_addr, control_addr);

        Ok(Self {
            config: config.clone(),
            audio_socket: Arc::new(audio_socket),
            control_socket: Arc::new(control_socket),
            sequence: 0,
            peers: Vec::new(),
        })
    }

    /// Add a peer to send audio to
    pub fn add_peer(&mut self, addr: SocketAddr) {
        if !self.peers.contains(&addr) {
            self.peers.push(addr);
            info!("Added peer: {}", addr);
        }
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, addr: SocketAddr) {
        self.peers.retain(|&p| p != addr);
        info!("Removed peer: {}", addr);
    }

    /// Send audio data to all peers
    pub async fn send_audio(&mut self, samples: &[i16], channels: u16, sample_rate: u32) -> Result<()> {
        let header = AudioPacketHeader {
            sequence: self.sequence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            sample_count: samples.len() as u16,
            channels,
            sample_rate,
        };

        // Serialize header and samples
        let mut payload = bincode::serialize(&header)?;
        
        // Convert samples to bytes
        let sample_bytes: Vec<u8> = samples.iter()
            .flat_map(|&s| s.to_le_bytes())
            .collect();
        payload.extend(sample_bytes);

        // Send to all peers
        for &peer in &self.peers {
            if let Err(e) = self.audio_socket.send_to(&payload, peer).await {
                warn!("Failed to send to {}: {}", peer, e);
            }
        }

        self.sequence = self.sequence.wrapping_add(1);
        Ok(())
    }

    /// Send control message to a specific peer
    pub async fn send_control(&self, msg: &ControlMessage, to: SocketAddr) -> Result<()> {
        let data = bincode::serialize(msg)?;
        self.control_socket.send_to(&data, to).await?;
        debug!("Sent control message to {}", to);
        Ok(())
    }

    /// Broadcast control message to all peers
    pub async fn broadcast_control(&self, msg: &ControlMessage) -> Result<()> {
        let data = bincode::serialize(msg)?;
        for &peer in &self.peers {
            if let Err(e) = self.control_socket.send_to(&data, peer).await {
                warn!("Failed to send control to {}: {}", peer, e);
            }
        }
        Ok(())
    }

    /// Receive control messages
    pub async fn recv_control(&self) -> Result<(ControlMessage, SocketAddr)> {
        let mut buf = vec![0u8; 4096];
        let (len, addr) = self.control_socket.recv_from(&mut buf).await?;
        let msg = bincode::deserialize(&buf[..len])?;
        Ok((msg, addr))
    }

    /// Get list of connected peers
    pub fn peers(&self) -> &[SocketAddr] {
        &self.peers
    }
}

/// Network receiver for receiving audio streams
pub struct AudioReceiver {
    config: NetworkConfig,
    audio_socket: Arc<UdpSocket>,
    control_socket: Arc<UdpSocket>,
    expected_sequence: Option<u64>,
    lost_packets: u64,
    received_packets: u64,
}

impl AudioReceiver {
    /// Create a new audio receiver
    pub async fn new(config: &NetworkConfig) -> Result<Self> {
        let audio_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.audio_port);
        let control_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.control_port);

        let audio_socket = UdpSocket::bind(audio_addr).await?;
        let control_socket = UdpSocket::bind(control_addr).await?;

        info!("Audio receiver bound to {}:{}", audio_addr, control_addr);

        Ok(Self {
            config: config.clone(),
            audio_socket: Arc::new(audio_socket),
            control_socket: Arc::new(control_socket),
            expected_sequence: None,
            lost_packets: 0,
            received_packets: 0,
        })
    }

    /// Receive audio packet
    pub async fn recv_audio(&mut self) -> Result<(AudioPacketHeader, Vec<i16>)> {
        let mut buf = vec![0u8; 65535];
        let (len, _addr) = self.audio_socket.recv_from(&mut buf).await?;

        // Parse header (fixed size)
        let header_size = bincode::serialized_size(&AudioPacketHeader {
            sequence: 0,
            timestamp: 0,
            sample_count: 0,
            channels: 0,
            sample_rate: 0,
        })? as usize;

        let header: AudioPacketHeader = bincode::deserialize(&buf[..header_size])?;
        
        // Parse samples
        let sample_data = &buf[header_size..len];
        let samples: Vec<i16> = sample_data
            .chunks_exact(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(i16::from_le_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();

        // Track sequence for loss detection
        if let Some(expected) = self.expected_sequence {
            if header.sequence != expected {
                let lost = header.sequence.saturating_sub(expected);
                if lost > 0 {
                    self.lost_packets += lost;
                    debug!("Lost {} packets (expected {}, got {})", lost, expected, header.sequence);
                }
            }
        }
        self.expected_sequence = Some(header.sequence.wrapping_add(1));
        self.received_packets += 1;

        Ok((header, samples))
    }

    /// Receive control message
    pub async fn recv_control(&self) -> Result<(ControlMessage, SocketAddr)> {
        let mut buf = vec![0u8; 4096];
        let (len, addr) = self.control_socket.recv_from(&mut buf).await?;
        let msg = bincode::deserialize(&buf[..len])?;
        Ok((msg, addr))
    }

    /// Send control message
    pub async fn send_control(&self, msg: &ControlMessage, to: SocketAddr) -> Result<()> {
        let data = bincode::serialize(msg)?;
        self.control_socket.send_to(&data, to).await?;
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64, f64) {
        let total = self.received_packets + self.lost_packets;
        let loss_rate = if total > 0 {
            (self.lost_packets as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (self.received_packets, self.lost_packets, loss_rate)
    }
}

/// Service discovery using mDNS
pub struct ServiceDiscovery {
    daemon: ServiceDaemon,
    service_type: String,
}

impl ServiceDiscovery {
    const SERVICE_TYPE: &'static str = "_audiosync._tcp.local.";

    /// Create a new service discovery instance
    pub fn new() -> Result<Self> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self {
            daemon,
            service_type: Self::SERVICE_TYPE.to_string(),
        })
    }

    /// Register this device for discovery
    pub fn register(&self, name: &str, port: u16, properties: Vec<(String, String)>) -> Result<()> {
        let service_info = ServiceInfo::new(
            &self.service_type,
            name,
            name,
            "",
            port,
            properties,
        )?;

        self.daemon.register(service_info)?;
        info!("Registered service: {} on port {}", name, port);
        Ok(())
    }

    /// Browse for available devices
    pub fn browse(&self) -> mpsc::Receiver<DiscoveredDevice> {
        let (tx, rx) = mpsc::channel(100);
        let daemon = self.daemon.clone();
        let service_type = self.service_type.clone();

        tokio::spawn(async move {
            let receiver = daemon.browse(&service_type).expect("Failed to browse");
            
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        for addr in info.get_addresses() {
                            let device = DiscoveredDevice {
                                name: info.get_fullname().to_string(),
                                address: SocketAddr::new(*addr, info.get_port()),
                                audio_port: info.get_port(),
                                control_port: info.get_port() + 1,
                                discovered_at: Instant::now(),
                            };
                            let _ = tx.send(device).await;
                        }
                    }
                    _ => {}
                }
            }
        });

        rx
    }

    /// Unregister service
    pub fn unregister(&self, name: &str) -> Result<()> {
        let full_name = format!("{}.{}", name, self.service_type);
        self.daemon.unregister(&full_name)?;
        Ok(())
    }
}

impl Drop for ServiceDiscovery {
    fn drop(&mut self) {
        let _ = self.daemon.shutdown();
    }
}

/// Helper function to get local IP addresses
pub fn get_local_ips() -> Vec<IpAddr> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .iter()
        .filter(|iface| !iface.is_loopback())
        .map(|iface| iface.ip())
        .filter(|ip| ip.is_ipv4())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let header = AudioPacketHeader {
            sequence: 123,
            timestamp: 456789,
            sample_count: 1024,
            channels: 2,
            sample_rate: 48000,
        };

        let serialized = bincode::serialize(&header).unwrap();
        let deserialized: AudioPacketHeader = bincode::deserialize(&serialized).unwrap();

        assert_eq!(header.sequence, deserialized.sequence);
        assert_eq!(header.timestamp, deserialized.timestamp);
        assert_eq!(header.sample_count, deserialized.sample_count);
    }
}
