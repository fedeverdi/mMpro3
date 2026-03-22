// Master Phase Correlation Meter
// Measures stereo phase relationship for broadcast compliance

use std::collections::VecDeque;

const WINDOW_SIZE_MS: f64 = 400.0; // 400ms window (same as LUFS momentary)

/// Phase correlation measurement result
#[derive(Debug, Clone, Copy)]
pub struct PhaseCorrelationMeasurement {
    pub correlation: f32, // -1.0 to +1.0
    pub mono_compatible: bool, // true if correlation >= 0.0
}

pub struct PhaseCorrelationMeter {
    _sample_rate: f64,
    window_samples: usize,
    
    // Sliding window buffers
    buffer_left: VecDeque<f32>,
    buffer_right: VecDeque<f32>,
    
    // Running sums for efficient calculation
    sum_lr: f64,      // sum(L * R)
    sum_l2: f64,      // sum(L^2)
    sum_r2: f64,      // sum(R^2)
    
    // Current correlation value
    correlation: f32,
}

impl PhaseCorrelationMeter {
    pub fn new(sample_rate: f64) -> Self {
        let window_samples = ((sample_rate * WINDOW_SIZE_MS) / 1000.0) as usize;
        
        Self {
            _sample_rate: sample_rate,
            window_samples,
            buffer_left: VecDeque::with_capacity(window_samples),
            buffer_right: VecDeque::with_capacity(window_samples),
            sum_lr: 0.0,
            sum_l2: 0.0,
            sum_r2: 0.0,
            correlation: 1.0, // Start at perfect correlation
        }
    }
    
    /// Process a single sample pair and update correlation
    pub fn process(&mut self, left: f32, right: f32) {
        let left_f64 = left as f64;
        let right_f64 = right as f64;
        
        // Add new samples to buffers
        self.buffer_left.push_back(left);
        self.buffer_right.push_back(right);
        
        // Update running sums with new sample
        self.sum_lr += left_f64 * right_f64;
        self.sum_l2 += left_f64 * left_f64;
        self.sum_r2 += right_f64 * right_f64;
        
        // Remove old samples if window is full
        if self.buffer_left.len() > self.window_samples {
            if let (Some(old_l), Some(old_r)) = (self.buffer_left.pop_front(), self.buffer_right.pop_front()) {
                let old_l_f64 = old_l as f64;
                let old_r_f64 = old_r as f64;
                
                // Subtract old sample contribution
                self.sum_lr -= old_l_f64 * old_r_f64;
                self.sum_l2 -= old_l_f64 * old_l_f64;
                self.sum_r2 -= old_r_f64 * old_r_f64;
            }
        }
        
        // Calculate correlation if we have enough samples
        if self.buffer_left.len() >= self.window_samples / 2 {
            self.calculate_correlation();
        }
    }
    
    /// Calculate phase correlation from running sums
    /// Formula: correlation = sum(L * R) / sqrt(sum(L^2) * sum(R^2))
    fn calculate_correlation(&mut self) {
        // Avoid division by zero
        if self.sum_l2 <= 1e-10 || self.sum_r2 <= 1e-10 {
            self.correlation = 0.0;
            return;
        }
        
        let denominator = (self.sum_l2 * self.sum_r2).sqrt();
        if denominator <= 1e-10 {
            self.correlation = 0.0;
            return;
        }
        
        let correlation = self.sum_lr / denominator;
        
        // Clamp to valid range and store
        self.correlation = correlation.clamp(-1.0, 1.0) as f32;
    }
    
    /// Check if signal is mono compatible (correlation >= 0.0)
    pub fn is_mono_compatible(&self) -> bool {
        self.correlation >= 0.0
    }
    
    /// Get full measurement
    pub fn get_measurement(&self) -> PhaseCorrelationMeasurement {
        PhaseCorrelationMeasurement {
            correlation: self.correlation,
            mono_compatible: self.is_mono_compatible(),
        }
    }
    
    /// Reset the meter
    pub fn reset(&mut self) {
        self.buffer_left.clear();
        self.buffer_right.clear();
        self.sum_lr = 0.0;
        self.sum_l2 = 0.0;
        self.sum_r2 = 0.0;
        self.correlation = 1.0;
    }
}
