//! Error types for the audio sync share system

use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error enum for all audio sync operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Audio capture failed: {0}")]
    AudioCapture(String),

    #[error("Audio playback failed: {0}")]
    AudioPlayback(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Synchronization error: {0}")]
    Sync(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Channel send error: {0}")]
    ChannelSend(String),

    #[error("Channel recv error: {0}")]
    ChannelRecv(String),
}

impl From<cpal::DevicesError> for Error {
    fn from(e: cpal::DevicesError) -> Self {
        Error::AudioCapture(e.to_string())
    }
}

impl From<cpal::DeviceNameError> for Error {
    fn from(e: cpal::DeviceNameError) -> Self {
        Error::AudioCapture(e.to_string())
    }
}

impl From<cpal::StreamError> for Error {
    fn from(e: cpal::StreamError) -> Self {
        Error::AudioPlayback(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Error::ChannelSend(e.to_string())
    }
}

impl From<tokio::sync::mpsc::error::RecvError> for Error {
    fn from(e: tokio::sync::mpsc::error::RecvError) -> Self {
        Error::ChannelRecv(e.to_string())
    }
}
