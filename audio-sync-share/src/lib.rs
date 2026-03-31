//! Audio Sync Share - Cross-platform low-latency audio streaming system
//! 
//! This crate provides real-time audio capture, network streaming,
//! and synchronized playback across multiple devices.

pub mod config;
pub mod audio_capture;
pub mod audio_player;
pub mod network;
pub mod media_control;
pub mod sync;
pub mod error;

pub use config::Config;
pub use error::{Error, Result};
