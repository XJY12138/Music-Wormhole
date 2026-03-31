//! Configuration module for audio sync share system

use serde::{Deserialize, Serialize};

/// Audio configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate in Hz (e.g., 44100, 48000)
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Buffer size in frames
    pub buffer_size: usize,
    /// Bits per sample
    pub bits_per_sample: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 512,
            bits_per_sample: 16,
        }
    }
}

/// Network configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Port for audio streaming
    pub audio_port: u16,
    /// Port for control messages
    pub control_port: u16,
    /// Multicast address for discovery
    pub multicast_addr: String,
    /// Stream chunk size in bytes
    pub chunk_size: usize,
    /// Network interface to use (optional)
    pub interface: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            audio_port: 50000,
            control_port: 50001,
            multicast_addr: "224.0.0.1".to_string(),
            chunk_size: 1024,
            interface: None,
        }
    }
}

/// Synchronization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Target latency in milliseconds
    pub target_latency_ms: u32,
    /// Maximum allowed drift in milliseconds
    pub max_drift_ms: u32,
    /// Enable adaptive buffering
    pub adaptive_buffering: bool,
    /// NTP server for time synchronization (optional)
    pub ntp_server: Option<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            target_latency_ms: 50,
            max_drift_ms: 10,
            adaptive_buffering: true,
            ntp_server: None,
        }
    }
}

/// Capture mode selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureMode {
    /// Capture all system audio
    Global,
    /// Capture audio from a specific application
    Application(String),
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Device name for identification
    pub device_name: String,
    /// Audio configuration
    pub audio: AudioConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Synchronization configuration
    pub sync: SyncConfig,
    /// Capture mode
    pub capture_mode: CaptureMode,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device_name: "unknown".to_string(),
            audio: AudioConfig::default(),
            network: NetworkConfig::default(),
            sync: SyncConfig::default(),
            capture_mode: CaptureMode::Global,
            verbose: false,
        }
    }
}

impl Config {
    /// Create a new configuration with custom device name
    pub fn new(device_name: &str) -> Self {
        Self {
            device_name: device_name.to_string(),
            ..Default::default()
        }
    }

    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.audio.sample_rate, 48000);
        assert_eq!(config.audio.channels, 2);
        assert_eq!(config.network.audio_port, 50000);
    }

    #[test]
    fn test_custom_config() {
        let config = Config::new("test-device");
        assert_eq!(config.device_name, "test-device");
    }
}
