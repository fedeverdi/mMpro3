// Dynamic Range Meter
// Calculates Peak, RMS, and Dynamic Range in real-time

use std::collections::VecDeque;

const WINDOW_SIZE_MS: f64 = 300.0; // 300ms window for RMS calculation

/// Dynamic range measurements result
#[derive(Debug, Clone, Copy)]
pub struct DynamicRangeMeasurements {
    pub peak_db_l: f32,
    pub peak_db_r: f32,
    pub rms_db_l: f32,
    pub rms_db_r: f32,
    pub dynamic_range_l: f32,
    pub dynamic_range_r: f32,
    pub dynamic_range_stereo: f32, // Average of L/R
}

pub struct DynamicRangeMeter {
    _sample_rate: f64,
    window_samples: usize,
    
    // Peak tracking (since last reset)
    peak_left: f64,
    peak_right: f64,
    
    // RMS calculation (sliding window)
    rms_buffer_left: VecDeque<f64>,
    rms_buffer_right: VecDeque<f64>,
    sum_squares_left: f64,
    sum_squares_right: f64,
    
    // Sample counter for buffer
    samples_accumulated: usize,
}

impl DynamicRangeMeter {
    pub fn new(sample_rate: f64) -> Self {
        let window_samples = ((sample_rate * WINDOW_SIZE_MS) / 1000.0) as usize;
        
        Self {
            _sample_rate: sample_rate,
            window_samples,
            peak_left: 0.0,
            peak_right: 0.0,
            rms_buffer_left: VecDeque::with_capacity(window_samples),
            rms_buffer_right: VecDeque::with_capacity(window_samples),
            sum_squares_left: 0.0,
            sum_squares_right: 0.0,
            samples_accumulated: 0,
        }
    }
    
    /// Process a single sample pair
    pub fn process(&mut self, left: f32, right: f32) {
        let left_f64 = left as f64;
        let right_f64 = right as f64;
        
        // Update peak values
        let abs_left = left_f64.abs();
        let abs_right = right_f64.abs();
        
        if abs_left > self.peak_left {
            self.peak_left = abs_left;
        }
        if abs_right > self.peak_right {
            self.peak_right = abs_right;
        }
        
        // Update RMS sliding window
        let left_squared = left_f64 * left_f64;
        let right_squared = right_f64 * right_f64;
        
        // Add new samples
        self.rms_buffer_left.push_back(left_squared);
        self.rms_buffer_right.push_back(right_squared);
        self.sum_squares_left += left_squared;
        self.sum_squares_right += right_squared;
        
        // Remove old samples if window is full
        if self.rms_buffer_left.len() > self.window_samples {
            if let Some(old_left) = self.rms_buffer_left.pop_front() {
                self.sum_squares_left -= old_left;
            }
        }
        if self.rms_buffer_right.len() > self.window_samples {
            if let Some(old_right) = self.rms_buffer_right.pop_front() {
                self.sum_squares_right -= old_right;
            }
        }
        
        self.samples_accumulated += 1;
    }
    
    /// Convert linear amplitude to dB
    fn to_db(amplitude: f64) -> f32 {
        if amplitude <= 0.0 || !amplitude.is_finite() {
            -90.0 // -90 dB floor
        } else {
            (20.0 * amplitude.log10()).max(-90.0) as f32
        }
    }
    
    /// Get peak level in dB
    pub fn get_peak_db(&self) -> (f32, f32) {
        let peak_db_l = Self::to_db(self.peak_left);
        let peak_db_r = Self::to_db(self.peak_right);
        (peak_db_l, peak_db_r)
    }
    
    /// Get RMS level in dB
    pub fn get_rms_db(&self) -> (f32, f32) {
        let count_left = self.rms_buffer_left.len() as f64;
        let count_right = self.rms_buffer_right.len() as f64;
        
        if count_left == 0.0 || count_right == 0.0 {
            return (-90.0, -90.0);
        }
        
        let rms_left = (self.sum_squares_left / count_left).sqrt();
        let rms_right = (self.sum_squares_right / count_right).sqrt();
        
        let rms_db_l = Self::to_db(rms_left);
        let rms_db_r = Self::to_db(rms_right);
        
        (rms_db_l, rms_db_r)
    }
    
    /// Get dynamic range (Peak - RMS) in dB
    pub fn get_dynamic_range(&self) -> (f32, f32, f32) {
        let (peak_db_l, peak_db_r) = self.get_peak_db();
        let (rms_db_l, rms_db_r) = self.get_rms_db();
        
        let dr_left = peak_db_l - rms_db_l;
        let dr_right = peak_db_r - rms_db_r;
        let dr_stereo = (dr_left + dr_right) / 2.0;
        
        (dr_left, dr_right, dr_stereo)
    }
    
    /// Get all measurements
    pub fn get_measurements(&self) -> DynamicRangeMeasurements {
        let (peak_db_l, peak_db_r) = self.get_peak_db();
        let (rms_db_l, rms_db_r) = self.get_rms_db();
        let (dr_l, dr_r, dr_stereo) = self.get_dynamic_range();
        
        DynamicRangeMeasurements {
            peak_db_l,
            peak_db_r,
            rms_db_l,
            rms_db_r,
            dynamic_range_l: dr_l,
            dynamic_range_r: dr_r,
            dynamic_range_stereo: dr_stereo,
        }
    }
    
    /// Reset all measurements
    pub fn reset(&mut self) {
        self.peak_left = 0.0;
        self.peak_right = 0.0;
        self.rms_buffer_left.clear();
        self.rms_buffer_right.clear();
        self.sum_squares_left = 0.0;
        self.sum_squares_right = 0.0;
        self.samples_accumulated = 0;
    }
}
