//! Synchronization module for multi-device audio alignment
//! 
//! Features:
//! - Network time synchronization (NTP-like protocol)
//! - Adaptive buffering for jitter compensation
//! - Clock drift detection and correction
//! - Sample rate adjustment for precise sync

use crate::config::SyncConfig;
use crate::error::{Error, Result};
use log::{debug, info, warn};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Timing information for synchronization
#[derive(Debug, Clone)]
pub struct TimingInfo {
    /// Local timestamp when packet was sent/received
    pub local_time: Instant,
    /// Remote timestamp from sender
    pub remote_time: u64,
    /// Round-trip time estimate
    pub rtt_ms: f64,
    /// Clock offset estimate
    pub offset_ms: f64,
}

/// Synchronization state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncState {
    Unsynced,
    Syncing,
    Synced,
    Drifting,
}

/// Audio synchronizer for multi-device alignment
pub struct AudioSynchronizer {
    config: SyncConfig,
    state: SyncState,
    
    // Timing measurements
    timing_history: VecDeque<TimingInfo>,
    max_history_size: usize,
    
    // Clock synchronization
    clock_offset_ms: f64,
    clock_drift_ppm: f64,  // Parts per million
    last_sync_time: Option<Instant>,
    
    // Buffer management
    target_buffer_ms: u32,
    current_buffer_ms: u32,
    buffer_adjustment_rate: f32,
    
    // Statistics
    sync_attempts: u64,
    successful_syncs: u64,
}

impl AudioSynchronizer {
    /// Create a new audio synchronizer
    pub fn new(config: &SyncConfig) -> Self {
        Self {
            config: config.clone(),
            state: SyncState::Unsynced,
            timing_history: VecDeque::with_capacity(100),
            max_history_size: 100,
            clock_offset_ms: 0.0,
            clock_drift_ppm: 0.0,
            last_sync_time: None,
            target_buffer_ms: config.target_latency_ms,
            current_buffer_ms: 0,
            buffer_adjustment_rate: 0.1,
            sync_attempts: 0,
            successful_syncs: 0,
        }
    }

    /// Record a timing measurement from ping-pong exchange
    pub fn record_timing(&mut self, local_send: Instant, remote_time: u64, local_recv: Instant) {
        let rtt = local_recv.duration_since(local_send).as_secs_f64() * 1000.0;
        
        // Estimate clock offset using simplified NTP algorithm
        // offset = ((t1 - t0) + (t2 - t3)) / 2
        // For simplicity, we assume t3 ≈ t2 (immediate response)
        let offset = (rtt / 2.0);  // Simplified estimation
        
        let timing = TimingInfo {
            local_time: local_recv,
            remote_time,
            rtt_ms: rtt,
            offset_ms: offset,
        };

        self.timing_history.push_back(timing);
        if self.timing_history.len() > self.max_history_size {
            self.timing_history.pop_front();
        }

        // Update clock offset estimate with moving average
        self.update_clock_estimate();
        
        self.sync_attempts += 1;
    }

    /// Update clock offset and drift estimates
    fn update_clock_estimate(&mut self) {
        if self.timing_history.len() < 5 {
            return;
        }

        // Calculate average offset
        let avg_offset: f64 = self.timing_history.iter()
            .map(|t| t.offset_ms)
            .sum::<f64>() / self.timing_history.len() as f64;

        // Calculate drift from offset changes over time
        if let Some(first) = self.timing_history.front() {
            if let Some(last) = self.timing_history.back() {
                let time_diff = last.local_time.duration_since(first.local_time).as_secs_f64();
                if time_diff > 0.0 {
                    let offset_change = last.offset_ms - first.offset_ms;
                    self.clock_drift_ppm = (offset_change / time_diff) / 1000.0 * 1_000_000.0;
                }
            }
        }

        // Apply low-pass filter to offset
        self.clock_offset_ms = self.clock_offset_ms * 0.9 + avg_offset * 0.1;

        // Update state
        let rtt_avg: f64 = self.timing_history.iter()
            .map(|t| t.rtt_ms)
            .sum::<f64>() / self.timing_history.len() as f64;

        if rtt_avg < self.config.max_drift_ms as f64 && self.timing_history.len() >= 10 {
            self.state = SyncState::Synced;
            self.successful_syncs += 1;
            info!("Synchronized: offset={:.2}ms, drift={:.2}ppm", self.clock_offset_ms, self.clock_drift_ppm);
        } else if self.timing_history.len() >= 5 {
            self.state = SyncState::Syncing;
        }

        self.last_sync_time = Some(Instant::now());
    }

    /// Get estimated clock offset in milliseconds
    pub fn clock_offset_ms(&self) -> f64 {
        self.clock_offset_ms
    }

    /// Get estimated clock drift in parts per million
    pub fn clock_drift_ppm(&self) -> f64 {
        self.clock_drift_ppm
    }

    /// Update current buffer level
    pub fn update_buffer_level(&mut self, level_ms: u32) {
        let prev = self.current_buffer_ms;
        self.current_buffer_ms = level_ms;

        // Detect significant buffer changes
        let diff = (level_ms as i32 - prev as i32).abs();
        if diff > self.config.max_drift_ms as i32 {
            self.state = SyncState::Drifting;
            warn!("Buffer drift detected: {}ms -> {}ms", prev, level_ms);
        }
    }

    /// Calculate required buffer adjustment
    pub fn get_buffer_adjustment(&self) -> f32 {
        let error = self.target_buffer_ms as i32 - self.current_buffer_ms as i32;
        (error as f32) * self.buffer_adjustment_rate
    }

    /// Get recommended sample rate adjustment
    pub fn get_sample_rate_adjustment(&self, base_rate: u32) -> f32 {
        // Combine drift compensation and buffer adjustment
        let drift_factor = 1.0 + (self.clock_drift_ppm / 1_000_000.0);
        let buffer_factor = 1.0 + (self.get_buffer_adjustment() / self.target_buffer_ms as f32);
        
        (base_rate as f32) * drift_factor * buffer_factor
    }

    /// Check if synchronized
    pub fn is_synced(&self) -> bool {
        self.state == SyncState::Synced
    }

    /// Get current sync state
    pub fn state(&self) -> SyncState {
        self.state
    }

    /// Get synchronization statistics
    pub fn stats(&self) -> SyncStats {
        SyncStats {
            state: self.state,
            clock_offset_ms: self.clock_offset_ms,
            clock_drift_ppm: self.clock_drift_ppm,
            sync_attempts: self.sync_attempts,
            successful_syncs: self.successful_syncs,
            success_rate: if self.sync_attempts > 0 {
                self.successful_syncs as f64 / self.sync_attempts as f64
            } else {
                0.0
            },
            avg_rtt_ms: if !self.timing_history.is_empty() {
                self.timing_history.iter().map(|t| t.rtt_ms).sum::<f64>() 
                    / self.timing_history.len() as f64
            } else {
                0.0
            },
            history_size: self.timing_history.len(),
        }
    }

    /// Reset synchronization state
    pub fn reset(&mut self) {
        self.state = SyncState::Unsynced;
        self.timing_history.clear();
        self.clock_offset_ms = 0.0;
        self.clock_drift_ppm = 0.0;
        self.last_sync_time = None;
    }
}

/// Synchronization statistics
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub state: SyncState,
    pub clock_offset_ms: f64,
    pub clock_drift_ppm: f64,
    pub sync_attempts: u64,
    pub successful_syncs: u64,
    pub success_rate: f64,
    pub avg_rtt_ms: f64,
    pub history_size: usize,
}

/// Helper for calculating playback timing
pub struct PlaybackTimer {
    start_time: Option<Instant>,
    samples_played: u64,
    sample_rate: u32,
    channels: u16,
}

impl PlaybackTimer {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            start_time: None,
            samples_played: 0,
            sample_rate,
            channels,
        }
    }

    /// Start or restart the timer
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.samples_played = 0;
    }

    /// Record samples played
    pub fn record_samples(&mut self, count: u64) {
        self.samples_played += count;
    }

    /// Get expected elapsed time based on samples played
    pub fn expected_elapsed(&self) -> Duration {
        let total_samples = self.samples_played / self.channels as u64;
        let secs = total_samples / self.sample_rate as u64;
        let nanos = ((total_samples % self.sample_rate as u64) * 1_000_000_000) / self.sample_rate as u64;
        Duration::new(secs, nanos as u32)
    }

    /// Get actual elapsed time
    pub fn actual_elapsed(&self) -> Duration {
        self.start_time.map(|t| t.elapsed()).unwrap_or(Duration::ZERO)
    }

    /// Get drift between expected and actual time
    pub fn drift(&self) -> Duration {
        let expected = self.expected_elapsed();
        let actual = self.actual_elapsed();
        
        if actual > expected {
            actual - expected
        } else {
            expected - actual
        }
    }

    /// Get drift in milliseconds (positive = behind, negative = ahead)
    pub fn drift_ms(&self) -> i64 {
        let drift = self.drift();
        let ms = drift.as_millis() as i64;
        
        // Determine sign
        if self.actual_elapsed() > self.expected_elapsed() {
            ms  // Behind
        } else {
            -ms  // Ahead
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synchronizer_creation() {
        let config = SyncConfig::default();
        let sync = AudioSynchronizer::new(&config);
        assert_eq!(sync.state(), SyncState::Unsynced);
    }

    #[test]
    fn test_timing_recording() {
        let mut sync = AudioSynchronizer::new(&SyncConfig::default());
        let now = Instant::now();
        
        sync.record_timing(now, 1000, now + Duration::from_millis(10));
        sync.record_timing(now, 1000, now + Duration::from_millis(12));
        sync.record_timing(now, 1000, now + Duration::from_millis(11));
        
        assert_eq!(sync.timing_history.len(), 3);
    }

    #[test]
    fn test_playback_timer() {
        let mut timer = PlaybackTimer::new(48000, 2);
        timer.start();
        
        // Record 48000 samples (should be 0.5 seconds for stereo)
        timer.record_samples(48000);
        
        let expected = timer.expected_elapsed();
        assert_eq!(expected.as_secs(), 0);
        assert!(expected.subsec_millis() >= 499 && expected.subsec_millis() <= 501);
    }
}
