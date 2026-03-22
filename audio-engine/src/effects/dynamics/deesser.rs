/// De-Esser - Reduces harsh sibilance in vocal recordings
/// 
/// Targets the 4-8kHz range where 'S', 'T', and 'CH' sounds occur,
/// applying dynamic compression only when sibilance is detected.

use std::f32::consts::PI;

pub struct DeEsser {
    sample_rate: f32,
    enabled: bool,
    
    // Parameters
    pub(crate) threshold: f32,   // dB, level where de-essing starts (-60 to 0)
    pub(crate) frequency: f32,   // Center frequency for sibilance detection (3000-8000 Hz)
    pub(crate) range: f32,       // dB, maximum reduction (0-20)
    
    // Band-pass filter for sibilance detection
    bp_state_l: [f32; 4], // Biquad filter state
    bp_state_r: [f32; 4],
    
    // Envelope follower for detection
    envelope_l: f32,
    envelope_r: f32,
    
    // Gain reduction smoothing
    gain_reduction_l: f32,
    gain_reduction_r: f32,
}

impl DeEsser {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            enabled: false,
            threshold: -20.0,
            frequency: 6000.0,
            range: 10.0,
            bp_state_l: [0.0; 4],
            bp_state_r: [0.0; 4],
            envelope_l: 0.0,
            envelope_r: 0.0,
            gain_reduction_l: 1.0,
            gain_reduction_r: 1.0,
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.bp_state_l.fill(0.0);
            self.bp_state_r.fill(0.0);
            self.envelope_l = 0.0;
            self.envelope_r = 0.0;
            self.gain_reduction_l = 1.0;
            self.gain_reduction_r = 1.0;
        }
    }
    
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.max(-60.0).min(0.0);
    }
    
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.max(3000.0).min(8000.0);
    }
    
    pub fn set_range(&mut self, range: f32) {
        self.range = range.max(0.0).min(20.0);
    }
    
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.bp_state_l.fill(0.0);
        self.bp_state_r.fill(0.0);
        self.envelope_l = 0.0;
        self.envelope_r = 0.0;
    }
    
    pub fn reset(&mut self) {
        self.bp_state_l.fill(0.0);
        self.bp_state_r.fill(0.0);
        self.envelope_l = 0.0;
        self.envelope_r = 0.0;
        self.gain_reduction_l = 1.0;
        self.gain_reduction_r = 1.0;
    }
    
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        // Band-pass filter coefficients centered at sibilance frequency
        let omega = 2.0 * PI * self.frequency / self.sample_rate;
        let bandwidth = 2000.0; // Hz
        let q = self.frequency / bandwidth;
        let alpha = omega.sin() / (2.0 * q);
        
        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega.cos();
        let a2 = 1.0 - alpha;
        
        // Normalize
        let b0 = b0 / a0;
        let b1 = b1 / a0;
        let b2 = b2 / a0;
        let a1 = a1 / a0;
        let a2 = a2 / a0;
        
        // Apply band-pass filter to left channel (detect sibilance)
        let bp_l = b0 * left + b1 * self.bp_state_l[0] + b2 * self.bp_state_l[1]
                    - a1 * self.bp_state_l[2] - a2 * self.bp_state_l[3];
        
        self.bp_state_l[1] = self.bp_state_l[0];
        self.bp_state_l[0] = left;
        self.bp_state_l[3] = self.bp_state_l[2];
        self.bp_state_l[2] = bp_l;
        
        // Apply band-pass filter to right channel
        let bp_r = b0 * right + b1 * self.bp_state_r[0] + b2 * self.bp_state_r[1]
                    - a1 * self.bp_state_r[2] - a2 * self.bp_state_r[3];
        
        self.bp_state_r[1] = self.bp_state_r[0];
        self.bp_state_r[0] = right;
        self.bp_state_r[3] = self.bp_state_r[2];
        self.bp_state_r[2] = bp_r;
        
        // Envelope follower for sibilance detection
        let attack_coef = (-1.0 / (0.001 * self.sample_rate)).exp(); // 1ms attack
        let release_coef = (-1.0 / (0.050 * self.sample_rate)).exp(); // 50ms release
        
        let bp_l_abs = bp_l.abs();
        if bp_l_abs > self.envelope_l {
            self.envelope_l = attack_coef * self.envelope_l + (1.0 - attack_coef) * bp_l_abs;
        } else {
            self.envelope_l = release_coef * self.envelope_l + (1.0 - release_coef) * bp_l_abs;
        }
        
        let bp_r_abs = bp_r.abs();
        if bp_r_abs > self.envelope_r {
            self.envelope_r = attack_coef * self.envelope_r + (1.0 - attack_coef) * bp_r_abs;
        } else {
            self.envelope_r = release_coef * self.envelope_r + (1.0 - release_coef) * bp_r_abs;
        }
        
        // Calculate gain reduction based on envelope
        let threshold_lin = self.db_to_linear(self.threshold);
        let range_lin = self.db_to_linear(-self.range);
        
        let target_gain_l = if self.envelope_l > threshold_lin {
            let over = (self.envelope_l / threshold_lin).max(1.0);
            (1.0 / over).max(range_lin)
        } else {
            1.0
        };
        
        let target_gain_r = if self.envelope_r > threshold_lin {
            let over = (self.envelope_r / threshold_lin).max(1.0);
            (1.0 / over).max(range_lin)
        } else {
            1.0
        };
        
        // Smooth gain reduction
        let smooth_coef = 0.95;
        self.gain_reduction_l = smooth_coef * self.gain_reduction_l + (1.0 - smooth_coef) * target_gain_l;
        self.gain_reduction_r = smooth_coef * self.gain_reduction_r + (1.0 - smooth_coef) * target_gain_r;
        
        // Apply gain reduction
        let out_l = left * self.gain_reduction_l;
        let out_r = right * self.gain_reduction_r;
        
        (out_l, out_r)
    }
    
    fn db_to_linear(&self, db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }
}
