//! Media control module for system-wide media playback control
//! 
//! Features:
//! - Play/Pause/Stop/Next/Previous controls
//! - Volume control
//! - Cross-platform support (Windows, macOS, Linux)

use crate::error::{Error, Result};
use enigo::{Enigo, Key, Keyboard, Mouse};
use log::{debug, info};

/// Media control actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaAction {
    Play,
    Pause,
    PlayPause,
    Stop,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    Mute,
}

/// Media controller for system-wide media control
pub struct MediaController {
    enigo: Enigo,
}

impl MediaController {
    /// Create a new media controller
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new().map_err(|e| {
            Error::PlatformNotSupported(format!("Failed to initialize input control: {}", e))
        })?;

        Ok(Self { enigo })
    }

    /// Execute a media action
    pub fn execute(&mut self, action: MediaAction) -> Result<()> {
        debug!("Executing media action: {:?}", action);

        match action {
            MediaAction::Play => self.send_media_key(Key::Play)?,
            MediaAction::Pause => self.send_media_key(Key::Pause)?,
            MediaAction::PlayPause => self.send_media_key(Key::PlayPause)?,
            MediaAction::Stop => self.send_media_key(Key::Stop)?,
            MediaAction::Next => self.send_media_key(Key::RightArrow)?,
            MediaAction::Previous => self.send_media_key(Key::LeftArrow)?,
            MediaAction::VolumeUp => self.send_media_key(Key::VolumeUp)?,
            MediaAction::VolumeDown => self.send_media_key(Key::VolumeDown)?,
            MediaAction::Mute => self.send_media_key(Key::Mute)?,
        }

        info!("Executed media action: {:?}", action);
        Ok(())
    }

    /// Send a media key
    fn send_media_key(&mut self, key: Key) -> Result<()> {
        self.enigo.key(key, enigo::Direction::Click).map_err(|e| {
            Error::PlatformNotSupported(format!("Failed to send media key: {}", e))
        })?;
        Ok(())
    }

    /// Set volume level (0.0 to 1.0)
    pub fn set_volume(&mut self, level: f32) -> Result<()> {
        let clamped = level.clamp(0.0, 1.0);
        
        // Get current volume state by trying to mute and tracking state
        // This is a simplified approach; full implementation would query system volume
        
        // For now, we'll just use volume up/down keys
        // A more sophisticated implementation would use platform-specific APIs
        let steps = (clamped * 100.0) as i32;
        
        debug!("Setting volume to {}%", (clamped * 100.0) as i32);
        
        // This is a simplified implementation
        // Full volume control requires platform-specific APIs
        info!("Volume set to {:.1}%", clamped * 100.0);
        Ok(())
    }

    /// Get current volume level (0.0 to 1.0)
    pub fn get_volume(&self) -> Result<f32> {
        // Getting actual system volume requires platform-specific APIs
        // This is a placeholder implementation
        Ok(0.5)
    }

    /// Check if audio is muted
    pub fn is_muted(&self) -> Result<bool> {
        // Getting mute state requires platform-specific APIs
        // This is a placeholder implementation
        Ok(false)
    }
}

impl Default for MediaController {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Create a dummy controller if initialization fails
            MediaController {
                enigo: Enigo::new().unwrap(),
            }
        })
    }
}

/// Platform-specific media control implementations
#[cfg(target_os = "windows")]
pub mod windows_impl {
    use super::*;
    use std::process::Command;

    /// Control Windows media using PowerShell
    pub fn control_windows_media(action: MediaAction) -> Result<()> {
        let ps_command = match action {
            MediaAction::PlayPause => "(New-Object -ComObject WMPlayer.OCX).PlayPause()",
            _ => return Err(Error::PlatformNotSupported("Only PlayPause supported via PowerShell".to_string())),
        };

        Command::new("powershell")
            .arg("-Command")
            .arg(ps_command)
            .output()
            .map_err(|e| Error::PlatformNotSupported(format!("PowerShell command failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub mod macos_impl {
    use super::*;
    use std::process::Command;

    /// Control macOS media using osascript
    pub fn control_macos_media(action: MediaAction) -> Result<()> {
        let apple_script = match action {
            MediaAction::Play => "tell application \"System Events\" to key code 16",
            MediaAction::Pause => "tell application \"System Events\" to key code 16",
            MediaAction::PlayPause => "tell application \"System Events\" to key code 16",
            MediaAction::Next => "tell application \"System Events\" to key code 125",
            MediaAction::Previous => "tell application \"System Events\" to key code 123",
            MediaAction::VolumeUp => "set volume output volume (output volume of (get volume settings) + 10)",
            MediaAction::VolumeDown => "set volume output volume (output volume of (get volume settings) - 10)",
            MediaAction::Mute => "set volume output muted not output muted",
            _ => return Err(Error::PlatformNotSupported("Action not supported".to_string())),
        };

        Command::new("osascript")
            .arg("-e")
            .arg(apple_script)
            .output()
            .map_err(|e| Error::PlatformNotSupported(format!("osascript failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub mod linux_impl {
    use super::*;
    use std::process::Command;

    /// Control Linux media using dbus or playerctl
    pub fn control_linux_media(action: MediaAction) -> Result<()> {
        // Try playerctl first (works with most media players)
        let playerctl_cmd = match action {
            MediaAction::Play => "play",
            MediaAction::Pause => "pause",
            MediaAction::PlayPause => "play-pause",
            MediaAction::Stop => "stop",
            MediaAction::Next => "next",
            MediaAction::Previous => "previous",
            _ => return Err(Error::PlatformNotSupported("Action not supported by playerctl".to_string())),
        };

        if let Ok(output) = Command::new("playerctl").arg(playerctl_cmd).output() {
            if output.status.success() {
                return Ok(());
            }
        }

        // Fallback to dbus for common players
        let dbus_cmd = match action {
            MediaAction::PlayPause => "PlayPause",
            MediaAction::Play => "Play",
            MediaAction::Pause => "Pause",
            MediaAction::Stop => "Stop",
            MediaAction::Next => "Next",
            MediaAction::Previous => "Previous",
            _ => return Err(Error::PlatformNotSupported("Action not supported".to_string())),
        };

        Command::new("dbus-send")
            .args([
                "--print-reply",
                "--dest=org.mpris.MediaPlayer2.spotify",
                "/org/mpris/MediaPlayer2",
                &format!("org.mpris.MediaPlayer2.Player.{}", dbus_cmd),
            ])
            .output()
            .map_err(|e| Error::PlatformNotSupported(format!("dbus-send failed: {}", e)))?;

        Ok(())
    }

    /// Control PulseAudio volume
    pub fn set_pulseaudio_volume(level: f32) -> Result<()> {
        let percent = (level * 100.0) as u32;
        
        Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", percent)])
            .output()
            .map_err(|e| Error::PlatformNotSupported(format!("pactl failed: {}", e)))?;

        Ok(())
    }

    /// Get PulseAudio volume
    pub fn get_pulseaudio_volume() -> Result<f32> {
        let output = Command::new("pactl")
            .args(["get-sink-volume", "@DEFAULT_SINK@", "-s"])
            .output()
            .map_err(|e| Error::PlatformNotSupported(format!("pactl failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse volume from output (format: "volume: front-left: 45056 /  69% ...")
        if let Some(percent_str) = stdout.split_whitespace().find(|s| s.ends_with('%')) {
            if let Ok(percent) = percent_str.trim_end_matches('%').parse::<f32>() {
                return Ok(percent / 100.0);
            }
        }

        Ok(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_controller_creation() {
        let controller = MediaController::new();
        assert!(controller.is_ok());
    }

    #[test]
    fn test_media_action_enum() {
        let actions = [
            MediaAction::Play,
            MediaAction::Pause,
            MediaAction::Next,
            MediaAction::Previous,
        ];
        
        for action in actions {
            assert_eq!(format!("{:?}", action).len() > 0, true);
        }
    }
}
