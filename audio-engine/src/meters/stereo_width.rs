// Stereo Width Meter
// Measures the stereo width/spread of the audio signal using Mid/Side analysis

use std::collections::VecDeque;

const WINDOW_SIZE_MS: f64 = 400.0; // 400ms window for RMS calculation

/// Stereo width measurement result
#[derive(Debug, Clone, Copy)]
pub struct StereoWidthMeasurement {
    pub width_percent: f32,      // 0-200% (100% = normal stereo)
    pub mid_rms: f32,            // Mid signal RMS (dB)
    pub side_rms: f32,           // Side signal RMS (dB)
    pub balance: f32,            // L/R balance (-1 = left, 0 = center, +1 = right)
}

pub struct StereoWidthMeter {
    window_samples: usize,
    
    // Mid/Side RMS buffers
    mid_buffer: VecDeque<f64>,
    side_buffer: VecDeque<f64>,
    
    // Running sums for efficient RMS calculation
    sum_mid_squared: f64,
    sum_side_squared: f64,
    
    // Balance calculation
    sum_left: f64,
    sum_right: f64,
    
    // Current measurements
    width_percent: f32,
    mid_rms: f32,
    side_rms: f32,
    balance: f32,
}

impl StereoWidthMeter {
    pub fn new(sample_rate: f64) -> Self {
        let window_samples = ((sample_rate * WINDOW_SIZE_MS) / 1000.0) as usize;
        
        Self {
            window_samples,
            mid_buffer: VecDeque::with_capacity(window_samples),
            side_buffer: VecDeque::with_capacity(window_samples),
            sum_mid_squared: 0.0,
            sum_side_squared: 0.0,
            sum_left: 0.0,
            sum_right: 0.0,
            width_percent: 100.0,
            mid_rms: -90.0,
            side_rms: -90.0,
            balance: 0.0,
        }
    }
    
    /// Process a single sample pair
    pub fn process(&mut self, left: f32, right: f32) {
        let left_f64 = left as f64;
        let right_f64 = right as f64;
        
        // Calculate Mid and Side components
        // Mid = (L + R) / 2 (center/mono content)
        // Side = (L - R) / 2 (stereo width content)
        let mid = (left_f64 + right_f64) / 2.0;
        let side = (left_f64 - right_f64) / 2.0;
        
        // Update buffers
        self.mid_buffer.push_back(mid * mid);
        self.side_buffer.push_back(side * side);
        
        // Update running sums
        self.sum_mid_squared += mid * mid;
        self.sum_side_squared += side * side;
        self.sum_left += left_f64.abs() as f64;
        self.sum_right += right_f64.abs() as f64;
        
        // Remove old samples if window is full
        if self.mid_buffer.len() > self.window_samples {
            if let Some(old_mid) = self.mid_buffer.pop_front() {
                self.sum_mid_squared -= old_mid;
            }
        }
        if self.side_buffer.len() > self.window_samples {
            if let Some(old_side) = self.side_buffer.pop_front() {
                self.sum_side_squared -= old_side;
            }
        }
        
        // Calculate measurements periodically (when buffer is sufficiently full)
        if self.mid_buffer.len() >= self.window_samples / 2 {
            self.calculate_measurements();
        }
    }
    
    /// Calculate stereo width and related measurements
    fn calculate_measurements(&mut self) {
        let count = self.mid_buffer.len() as f64;
        
        if count < 10.0 {
            return;
        }
        
        // Calculate RMS values
        let mid_rms_linear = (self.sum_mid_squared / count).sqrt();
        let side_rms_linear = (self.sum_side_squared / count).sqrt();
        
        // Convert to dB
        self.mid_rms = Self::to_db(mid_rms_linear);
        self.side_rms = Self::to_db(side_rms_linear);
        
        // Calculate stereo width percentage
        // Width = (Side / Mid) * 100
        // 0% = mono (no side content)
        // 100% = normal stereo
        // 200% = very wide stereo
        if mid_rms_linear > 1e-10 {
            let width_ratio = side_rms_linear / mid_rms_linear;
            // Scale to percentage (empirical scaling for typical mixes)
            self.width_percent = (width_ratio * 141.4).min(200.0) as f32; // sqrt(2) * 100 for normalization
        } else {
            self.width_percent = 0.0;
        }
        
        // Calculate L/R balance
        let total = self.sum_left + self.sum_right;
        if total > 1e-10 {
            // -1.0 (all left) to +1.0 (all right), 0.0 = balanced
            self.balance = ((self.sum_right - self.sum_left) / total) as f32;
        } else {
            self.balance = 0.0;
        }
        
        // Reset balance accumulators for next window
        self.sum_left = 0.0;
        self.sum_right = 0.0;
    }
    
    /// Convert linear amplitude to dB
    fn to_db(amplitude: f64) -> f32 {
        if amplitude <= 1e-10 {
            -90.0
        } else {
            (20.0 * amplitude.log10()).max(-90.0) as f32
        }
    }
    
    /// Get full measurement
    pub fn get_measurement(&self) -> StereoWidthMeasurement {
        StereoWidthMeasurement {
            width_percent: self.width_percent,
            mid_rms: self.mid_rms,
            side_rms: self.side_rms,
            balance: self.balance,
        }
    }
    
    /// Reset the meter
    pub fn reset(&mut self) {
        self.mid_buffer.clear();
        self.side_buffer.clear();
        self.sum_mid_squared = 0.0;
        self.sum_side_squared = 0.0;
        self.sum_left = 0.0;
        self.sum_right = 0.0;
        self.width_percent = 100.0;
        self.mid_rms = -90.0;
        self.side_rms = -90.0;
        self.balance = 0.0;
    }
    
    /// Set sample rate and resize buffers
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.window_samples = ((sample_rate * WINDOW_SIZE_MS) / 1000.0) as usize;
        self.reset();
    }
}
