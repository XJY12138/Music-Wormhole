//! Audio playback module for synchronized multi-device audio output

use crate::config::AudioConfig;
use crate::error::{Error, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream};
use log::{debug, error, info};
use ringbuf::{HeapRb, Rb};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Audio sample type (must match capture module)
pub type AudioSample = i16;

/// Thread-safe audio ring buffer for playback
pub type PlaybackBuffer = Arc<HeapRb<AudioSample>>;

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Stopped,
}

/// Synchronization information
#[derive(Debug, Clone)]
pub struct SyncInfo {
    /// Target timestamp for this chunk
    pub target_time: Instant,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Sender's timestamp
    pub sender_timestamp: Duration,
}

/// Audio playback handle
pub struct AudioPlayer {
    device: Device,
    stream: Option<Stream>,
    buffer: PlaybackBuffer,
    state: PlaybackState,
    config: AudioConfig,
    sync_info: Option<SyncInfo>,
    buffer_target_ms: u32,
}

impl AudioPlayer {
    /// Create a new audio player instance
    pub fn new(config: &AudioConfig, target_latency_ms: u32) -> Result<Self> {
        let host = cpal::default_host();
        
        // Get default output device
        let device = host.default_output_device()
            .ok_or_else(|| Error::AudioPlayback("No output device found".to_string()))?;

        let buffer_size = config.buffer_size * config.channels as usize;
        // Allocate larger buffer for network jitter compensation
        let buffer_capacity = buffer_size * (target_latency_ms as usize / 10 + 4);
        let buffer = Arc::new(HeapRb::new(buffer_capacity));

        Ok(Self {
            device,
            stream: None,
            buffer,
            state: PlaybackState::Idle,
            config: config.clone(),
            sync_info: None,
            buffer_target_ms: target_latency_ms,
        })
    }

    /// Write audio samples to the playback buffer
    pub fn write_samples(&self, samples: &[AudioSample]) -> usize {
        let written = self.buffer.push_slice(samples);
        if written < samples.len() {
            debug!("Playback buffer full, dropped {} samples", samples.len() - written);
        }
        written
    }

    /// Start audio playback
    pub fn start(&mut self) -> Result<()> {
        if self.state == PlaybackState::Playing {
            return Ok(());
        }

        let config = self.config.clone();
        let buffer = self.buffer.clone();
        let mut last_underrun = Instant::now();

        // Build stream configuration
        let stream_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        // Calculate minimum buffer level before playing
        let min_buffer_level = config.sample_rate as usize 
            * config.channels as usize 
            * (self.buffer_target_ms as usize) / 1000;

        // Create audio stream with callback
        let err_fn = |err| error!("Audio playback error: {}", err);
        
        let stream = self.device.build_output_stream(
            &stream_config,
            move |out_data: &mut [AudioSample], _: &cpal::OutputCallbackInfo| {
                // Check if we have enough data buffered
                let available = buffer.occupied();
                
                if available < min_buffer_level {
                    // Not enough data, fill with silence to avoid underrun
                    for sample in out_data.iter_mut() {
                        *sample = 0;
                    }
                    return;
                }

                // Read from buffer
                let read = buffer.pop_slice(out_data);
                
                // Fill remaining with silence if buffer was short
                for sample in out_data[read..].iter_mut() {
                    *sample = 0;
                }

                // Track underruns
                if read < out_data.len() {
                    let now = Instant::now();
                    if now.duration_since(last_underrun) > Duration::from_secs(1) {
                        debug!("Playback buffer underrun");
                        last_underrun = now;
                    }
                }
            },
            err_fn,
            None,
        ).map_err(|e| Error::AudioPlayback(e.to_string()))?;

        stream.play().map_err(|e| Error::AudioPlayback(e.to_string()))?;

        self.stream = Some(stream);
        self.state = PlaybackState::Buffering;

        info!("Audio playback started");
        Ok(())
    }

    /// Transition to playing state when enough data is buffered
    pub fn set_playing(&mut self) {
        if self.state == PlaybackState::Buffering {
            let level = self.buffer.occupied();
            let threshold = self.config.sample_rate as usize 
                * self.config.channels as usize 
                * (self.buffer_target_ms as usize) / 1000;
            
            if level >= threshold {
                self.state = PlaybackState::Playing;
                info!("Playback buffer ready, transitioning to playing state");
            }
        }
    }

    /// Pause audio playback
    pub fn pause(&mut self) -> Result<()> {
        if let Some(ref stream) = self.stream {
            stream.pause().map_err(|e| Error::AudioPlayback(e.to_string()))?;
            self.state = PlaybackState::Paused;
            info!("Audio playback paused");
        }
        Ok(())
    }

    /// Resume audio playback
    pub fn resume(&mut self) -> Result<()> {
        if let Some(ref stream) = self.stream {
            stream.play().map_err(|e| Error::AudioPlayback(e.to_string()))?;
            self.state = PlaybackState::Playing;
            info!("Audio playback resumed");
        }
        Ok(())
    }

    /// Stop audio playback
    pub fn stop(&mut self) -> Result<()> {
        self.stream = None;
        self.state = PlaybackState::Stopped;
        // Clear buffer
        let capacity = self.buffer.total();
        let mut temp = vec![AudioSample::default(); capacity];
        let _ = self.buffer.pop_slice(&mut temp);
        info!("Audio playback stopped");
        Ok(())
    }

    /// Set synchronization information
    pub fn set_sync_info(&mut self, sync: SyncInfo) {
        self.sync_info = Some(sync);
    }

    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Get current buffer level in milliseconds
    pub fn buffer_level_ms(&self) -> u32 {
        let samples = self.buffer.occupied();
        let rate = self.config.sample_rate as f64;
        let channels = self.config.channels as f64;
        ((samples as f64) / (rate * channels) * 1000.0) as u32
    }

    /// Get raw buffer for direct access
    pub fn buffer(&self) -> PlaybackBuffer {
        self.buffer.clone()
    }

    /// Check if player is ready (has enough buffered data)
    pub fn is_ready(&self) -> bool {
        self.buffer_level_ms() >= self.buffer_target_ms
    }

    /// Adjust playback timing for synchronization
    pub fn adjust_timing(&mut self, drift_ms: i32) {
        // Positive drift means we're behind, need to speed up
        // Negative drift means we're ahead, need to slow down
        debug!("Timing adjustment: {} ms", drift_ms);
        
        // In a full implementation, this would:
        // 1. Adjust sample rate slightly using rubato
        // 2. Skip or duplicate samples if drift is large
        // 3. Update sync_info with corrected timestamps
        
        if let Some(ref mut sync) = self.sync_info {
            // Adjust target time based on drift
            let adjustment = Duration::from_millis(drift_ms.unsigned_abs() as u64);
            if drift_ms > 0 {
                sync.target_time = sync.target_time - adjustment;
            } else {
                sync.target_time = sync.target_time + adjustment;
            }
        }
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Statistics for monitoring playback performance
#[derive(Debug, Default, Clone)]
pub struct PlaybackStats {
    /// Total samples played
    pub total_samples: u64,
    /// Number of buffer underruns
    pub underruns: u64,
    /// Average buffer level in ms
    pub avg_buffer_level_ms: f64,
    /// Last update time
    pub last_update: Instant,
}

impl PlaybackStats {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            ..Default::default()
        }
    }
}
