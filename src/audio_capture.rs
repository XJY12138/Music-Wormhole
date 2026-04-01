//! Audio capture module for cross-platform audio input
//! 
//! Supports:
//! - Global system audio capture (loopback)
//! - Application-specific audio capture
//! - Multiple platform backends (ALSA, PulseAudio, CoreAudio, WASAPI)

use crate::config::{AudioConfig, CaptureMode};
use crate::error::{Error, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream};
use log::{debug, error, info};
use ringbuf::{HeapRb, Rb};
use std::sync::Arc;

/// Audio sample type (16-bit signed integer)
pub type AudioSample = i16;

/// Thread-safe audio ring buffer
pub type AudioBuffer = Arc<HeapRb<AudioSample>>;

/// Audio capture state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureState {
    Idle,
    Running,
    Paused,
    Stopped,
}

/// Audio capture handle
pub struct AudioCapture {
    device: Device,
    stream: Option<Stream>,
    buffer: AudioBuffer,
    state: CaptureState,
    config: AudioConfig,
    mode: CaptureMode,
}

impl AudioCapture {
    /// Create a new audio capture instance
    pub fn new(config: &AudioConfig, mode: CaptureMode) -> Result<Self> {
        let host = cpal::default_host();
        
        // Select appropriate device based on capture mode
        let device = match &mode {
            CaptureMode::Global => {
                // Try to find a loopback or output device for capture
                Self::find_capture_device(&host)?
            }
            CaptureMode::Application(app_name) => {
                debug!("Application-specific capture requested: {}", app_name);
                // Note: True application-specific capture requires platform-specific APIs
                // For now, we use the default capture device
                Self::find_capture_device(&host)?
            }
        };

        let buffer_size = config.buffer_size * config.channels as usize;
        let buffer = Arc::new(HeapRb::new(buffer_size * 4)); // 4x buffer for safety

        Ok(Self {
            device,
            stream: None,
            buffer,
            state: CaptureState::Idle,
            config: config.clone(),
            mode,
        })
    }

    /// Find an appropriate capture device
    fn find_capture_device(host: &cpal::Host) -> Result<Device> {
        // Try to get default input device (may work for loopback on some systems)
        if let Some(device) = host.default_input_device() {
            info!("Using default input device for capture");
            return Ok(device);
        }

        // Fallback: try default output device (for loopback scenarios)
        if let Some(device) = host.default_output_device() {
            info!("Using default output device for capture (loopback mode)");
            return Ok(device);
        }

        // Last resort: try any available device
        let devices = host.devices()?;
        for device in devices {
            if device.default_input_config().is_ok() {
                info!("Using available input device for capture");
                return Ok(device);
            }
        }

        Err(Error::AudioCapture("No suitable audio capture device found".to_string()))
    }

    /// Get supported sample formats for the device
    pub fn get_supported_formats(&self) -> Vec<SampleFormat> {
        let mut formats = Vec::new();
        
        if let Ok(config) = self.device.default_input_config() {
            formats.push(config.sample_format());
        }
        
        if let Ok(config) = self.device.default_output_config() {
            let fmt = config.sample_format();
            if !formats.contains(&fmt) {
                formats.push(fmt);
            }
        }
        
        formats
    }

    /// Start audio capture
    pub fn start(&mut self) -> Result<()> {
        if self.state == CaptureState::Running {
            return Ok(());
        }

        let config = self.config.clone();
        let buffer = self.buffer.clone();

        // Build stream configuration
        let stream_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        // Create audio stream with callback
        let err_fn = |err| error!("Audio stream error: {}", err);
        
        let stream = self.device.build_input_stream(
            &stream_config,
            move |data: &[AudioSample], _: &cpal::InputCallbackInfo| {
                // Write captured audio to ring buffer
                let written = buffer.push_slice(data);
                if written < data.len() {
                    debug!("Buffer overflow, dropped {} samples", data.len() - written);
                }
            },
            err_fn,
            None,
        ).map_err(|e| Error::AudioCapture(e.to_string()))?;

        stream.play().map_err(|e| Error::AudioCapture(e.to_string()))?;

        self.stream = Some(stream);
        self.state = CaptureState::Running;

        info!("Audio capture started");
        Ok(())
    }

    /// Pause audio capture
    pub fn pause(&mut self) -> Result<()> {
        if let Some(ref stream) = self.stream {
            stream.pause().map_err(|e| Error::AudioCapture(e.to_string()))?;
            self.state = CaptureState::Paused;
            info!("Audio capture paused");
        }
        Ok(())
    }

    /// Resume audio capture
    pub fn resume(&mut self) -> Result<()> {
        if let Some(ref stream) = self.stream {
            stream.play().map_err(|e| Error::AudioCapture(e.to_string()))?;
            self.state = CaptureState::Running;
            info!("Audio capture resumed");
        }
        Ok(())
    }

    /// Stop audio capture
    pub fn stop(&mut self) -> Result<()> {
        self.stream = None;
        self.state = CaptureState::Stopped;
        info!("Audio capture stopped");
        Ok(())
    }

    /// Read audio samples from the buffer
    pub fn read_samples(&self, count: usize) -> Vec<AudioSample> {
        let mut samples = vec![AudioSample::default(); count];
        let read = self.buffer.pop_slice(&mut samples);
        samples.truncate(read);
        samples
    }

    /// Get current capture state
    pub fn state(&self) -> CaptureState {
        self.state
    }

    /// Get the audio buffer for direct access
    pub fn buffer(&self) -> AudioBuffer {
        self.buffer.clone()
    }

    /// Check if buffer has data available
    pub fn has_data(&self) -> bool {
        self.buffer.occupied() > 0
    }

    /// Get current buffer occupancy
    pub fn buffer_level(&self) -> usize {
        self.buffer.occupied()
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Platform-specific helper functions
#[cfg(target_os = "linux")]
pub mod linux_helpers {
    use super::*;
    
    /// Check if PulseAudio is available
    pub fn is_pulseaudio_available() -> bool {
        std::env::var("PULSE_SERVER").is_ok()
    }
    
    /// Check if ALSA is available
    pub fn is_alsa_available() -> bool {
        std::path::Path::new("/dev/snd").exists()
    }
}

#[cfg(target_os = "windows")]
pub mod windows_helpers {
    use super::*;
    
    /// Check if WASAPI loopback is available
    pub fn is_wasapi_loopback_available() -> bool {
        // WASAPI loopback is available on Windows Vista and later
        true
    }
}

#[cfg(target_os = "macos")]
pub mod macos_helpers {
    use super::*;
    
    /// Check if CoreAudio is available
    pub fn is_coreaudio_available() -> bool {
        true // CoreAudio is always available on macOS
    }
}
