//! Performance Budget Manager — adaptive quality control.
//!
//! Tracks frame times and downgrades material quality when the compositor
//! drops below the target frame rate. Integrates with the battery state
//! for power-aware quality reduction.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Quality levels for material downgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualityLevel {
    /// Full quality — all effects enabled.
    High = 0,
    /// Reduced — fewer blur passes, no dispersion.
    Medium = 1,
    /// Minimal — basic blur only.
    Low = 2,
    /// Safe — no liquid effects.
    Safe = 3,
}

impl QualityLevel {
    pub fn name(self) -> &'static str {
        match self {
            QualityLevel::High => "HIGH",
            QualityLevel::Medium => "MEDIUM",
            QualityLevel::Low => "LOW",
            QualityLevel::Safe => "SAFE",
        }
    }
}

/// Manages performance budget and quality adjustments.
pub struct PerformanceBudgetManager {
    /// Target frame duration.
    target_frame_duration: Duration,
    /// Maximum blur passes at current quality.
    max_blur_passes: u8,
    /// Whether to disable dispersion when on battery.
    disable_dispersion_on_battery: bool,
    /// Whether to auto-downgrade on frame drops.
    downgrade_on_frame_drop: bool,
    /// Recent frame times for averaging.
    frame_times: VecDeque<Duration>,
    /// Maximum frame times to track.
    max_samples: usize,
    /// Current quality level.
    current_quality: QualityLevel,
    /// Count of consecutive frame drops.
    consecutive_drops: u32,
    /// When the last quality change occurred.
    last_quality_change: Instant,
    /// Cooldown before another downgrade/upgrade.
    quality_cooldown: Duration,
    /// Whether we're on battery power.
    on_battery: bool,
    /// Frame drop threshold (fraction of target frame time).
    drop_threshold: f64,
}

impl PerformanceBudgetManager {
    pub fn new(config: &niri_config::PerformanceBudget) -> Self {
        let target_frame_duration = Duration::from_secs_f64(1.0 / config.target_fps as f64);
        Self {
            target_frame_duration,
            max_blur_passes: config.max_blur_passes,
            disable_dispersion_on_battery: config.disable_dispersion_on_battery,
            downgrade_on_frame_drop: config.downgrade_material_on_frame_drop,
            frame_times: VecDeque::new(),
            max_samples: 60,
            current_quality: QualityLevel::High,
            consecutive_drops: 0,
            last_quality_change: Instant::now(),
            quality_cooldown: Duration::from_secs(2),
            on_battery: false,
            drop_threshold: 1.5, // 50% over target = frame drop
        }
    }

    /// Record a frame time. Returns the current quality level.
    pub fn record_frame(&mut self, frame_time: Duration) -> QualityLevel {
        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }

        if !self.downgrade_on_frame_drop {
            return self.current_quality;
        }

        let is_drop = frame_time > self.target_frame_duration.mul_f64(self.drop_threshold);

        if is_drop {
            self.consecutive_drops += 1;
        } else {
            // Gradually recover.
            if self.consecutive_drops > 0 {
                self.consecutive_drops = self.consecutive_drops.saturating_sub(1);
            }
        }

        // Downgrade after 5 consecutive drops.
        if self.consecutive_drops >= 5
            && self.current_quality < QualityLevel::Safe
            && self.last_quality_change.elapsed() > self.quality_cooldown
        {
            self.current_quality = match self.current_quality {
                QualityLevel::High => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Safe,
                QualityLevel::Safe => QualityLevel::Safe,
            };
            self.last_quality_change = Instant::now();
            self.consecutive_drops = 0;
        }

        // Upgrade after 60 good frames.
        if self.consecutive_drops == 0
            && self.current_quality > QualityLevel::High
            && self.average_frame_time() < self.target_frame_duration
            && self.last_quality_change.elapsed() > Duration::from_secs(10)
        {
            self.current_quality = match self.current_quality {
                QualityLevel::Safe => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::High,
            };
            self.last_quality_change = Instant::now();
        }

        self.current_quality
    }

    /// Get the current recommended blur passes (capped by quality).
    pub fn effective_blur_passes(&self) -> u8 {
        match self.current_quality {
            QualityLevel::High => self.max_blur_passes,
            QualityLevel::Medium => (self.max_blur_passes).min(2),
            QualityLevel::Low => (self.max_blur_passes).min(1),
            QualityLevel::Safe => 0,
        }
    }

    /// Whether chromatic dispersion should be disabled.
    pub fn disable_dispersion(&self) -> bool {
        (self.on_battery && self.disable_dispersion_on_battery)
            || self.current_quality <= QualityLevel::Medium
    }

    /// Current quality level.
    pub fn quality_level(&self) -> QualityLevel {
        self.current_quality
    }

    /// Average frame time over the tracking window.
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }

    /// Current estimated FPS.
    pub fn current_fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg.is_zero() {
            return 0.0;
        }
        1.0 / avg.as_secs_f64()
    }

    /// Notify the manager that battery state changed.
    pub fn set_battery_state(&mut self, on_battery: bool) {
        self.on_battery = on_battery;
    }

    /// Whether a quality change notification should be shown.
    pub fn quality_just_changed(&self) -> Option<QualityLevel> {
        if self.last_quality_change.elapsed() < Duration::from_millis(500) {
            Some(self.current_quality)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_high_quality() {
        let config = niri_config::PerformanceBudget::default();
        let mgr = PerformanceBudgetManager::new(&config);
        assert_eq!(mgr.quality_level(), QualityLevel::High);
    }

    #[test]
    fn frame_times_tracked() {
        let config = niri_config::PerformanceBudget::default();
        let mut mgr = PerformanceBudgetManager::new(&config);
        mgr.record_frame(Duration::from_millis(8));
        mgr.record_frame(Duration::from_millis(16));
        assert!(mgr.current_fps() > 0.0);
    }

    #[test]
    fn effective_blur_passes_scales_with_quality() {
        let config = niri_config::PerformanceBudget {
            max_blur_passes: 4,
            ..Default::default()
        };
        let mgr = PerformanceBudgetManager::new(&config);
        assert_eq!(mgr.effective_blur_passes(), 4);
    }
}
