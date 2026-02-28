/// Limiter - Brick wall limiting (compressor with very high ratio and fast attack)
/// Prevents signal from exceeding a threshold, protecting against clipping
pub struct Limiter {
    /// Ceiling/Threshold in dB (signals cannot exceed this)
    ceiling_db: f32,
    /// Release time in milliseconds (how fast limiting disengages)
    release_ms: f32,
    /// Whether the limiter is enabled
    enabled: bool,
    /// Sample rate for time constant calculations
    sample_rate: f32,
    
    // Internal state
    /// Current envelope level (in linear amplitude)
    envelope: f32,
    /// Release coefficient (calculated from release_ms)
    release_coeff: f32,
    /// Smoothed gain reduction (prevents zipper noise)
    smoothed_gain_reduction: f32,
    
    /// Current gain reduction in dB (for metering/visualization)
    pub gain_reduction_db: f32,
    /// Current input level in dB (for visualization)
    pub input_level_db: f32,
}

impl Limiter {
    /// Create a new limiter with default settings
    pub fn new(sample_rate: f32) -> Self {
        let mut limiter = Self {
            ceiling_db: -0.1,    // Just below 0dB to prevent clipping
            release_ms: 100.0,   // Fast release for transparent limiting
            enabled: false,
            sample_rate,
            envelope: 0.0,
            release_coeff: 0.0,
            smoothed_gain_reduction: 0.0,
            gain_reduction_db: 0.0,
            input_level_db: -90.0,
        };
        
        limiter.update_coefficients();
        limiter
    }
    
    /// Update release coefficient based on time constant
    fn update_coefficients(&mut self) {
        let release_tau = self.release_ms / 1000.0;
        self.release_coeff = (-1.0 / (release_tau * self.sample_rate)).exp();
    }
    
    /// Set ceiling in dB
    pub fn set_ceiling(&mut self, ceiling_db: f32) {
        self.ceiling_db = ceiling_db.clamp(-20.0, 0.0);
    }
    
    /// Set release time in milliseconds
    pub fn set_release(&mut self, release_ms: f32) {
        self.release_ms = release_ms.clamp(10.0, 1000.0);
        self.update_coefficients();
    }
    
    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.envelope = 0.0;
            self.smoothed_gain_reduction = 0.0;
            self.gain_reduction_db = 0.0;
            self.input_level_db = -90.0;
        }
    }
    
    /// Update sample rate and recalculate coefficients
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }
    
    /// Process a stereo frame through the limiter
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            self.gain_reduction_db = 0.0;
            return (left, right);
        }
        
        // Calculate the peak level of the stereo signal (in linear amplitude)
        let input_level = left.abs().max(right.abs());
        
        // Very fast attack (0.1ms) for limiter - use instant attack for peaks
        let attack_coeff = (-1.0 / (0.0001 * self.sample_rate)).exp();
        
        // Update envelope follower
        let coeff = if input_level > self.envelope {
            attack_coeff  // Very fast attack
        } else {
            self.release_coeff  // Normal release
        };
        
        self.envelope = input_level + coeff * (self.envelope - input_level);
        
        // Convert envelope to dB
        let envelope_db = if self.envelope > 0.0 {
            20.0 * self.envelope.log10()
        } else {
            -90.0
        };
        
        // Store input level for visualization
        self.input_level_db = envelope_db;
        
        // Calculate gain reduction with soft knee (2dB for very smooth limiting)
        let knee_width = 2.0; // Small knee for transparent limiting
        let mut gain_reduction_db = 0.0;
        
        if envelope_db > self.ceiling_db + knee_width / 2.0 {
            // Above knee: full limiting (infinite ratio)
            gain_reduction_db = envelope_db - self.ceiling_db;
        } else if envelope_db > self.ceiling_db - knee_width / 2.0 {
            // In knee: soft transition using quadratic curve
            let overshoot_db = envelope_db - self.ceiling_db + knee_width / 2.0;
            let knee_factor = overshoot_db / knee_width;
            gain_reduction_db = knee_factor * knee_factor * knee_width / 4.0;
        }
        
        // Smooth the gain reduction to prevent zipper noise
        let smooth_coeff = (-1.0 / (0.005 * self.sample_rate)).exp();
        self.smoothed_gain_reduction = gain_reduction_db + smooth_coeff * (self.smoothed_gain_reduction - gain_reduction_db);
        
        // Store for metering
        self.gain_reduction_db = gain_reduction_db;
        
        // Convert smoothed gain reduction to linear gain
        let gain = 10.0_f32.powf(-self.smoothed_gain_reduction / 20.0);
        
        // Apply gain to both channels
        (left * gain, right * gain)
    }
    
    /// Reset limiter state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.smoothed_gain_reduction = 0.0;
        self.gain_reduction_db = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_limiter_creation() {
        let limiter = Limiter::new(48000.0);
        assert_eq!(limiter.sample_rate, 48000.0);
        assert!(!limiter.enabled);
    }
    
    #[test]
    fn test_limiter_passthrough_when_disabled() {
        let mut limiter = Limiter::new(48000.0);
        limiter.set_enabled(false);
        
        let (out_l, out_r) = limiter.process(0.5, -0.3);
        assert_eq!(out_l, 0.5);
        assert_eq!(out_r, -0.3);
    }
    
    #[test]
    fn test_limiter_reduces_loud_signals() {
        let mut limiter = Limiter::new(48000.0);
        limiter.set_ceiling(-1.0);
        limiter.set_release(50.0);
        limiter.set_enabled(true);
        
        // Process a very loud signal multiple times to let envelope build up
        let mut out_l = 1.5;
        let mut out_r = 1.5;
        for _ in 0..1000 {
            (out_l, out_r) = limiter.process(1.5, 1.5);
        }
        
        // Output should be limited (less than input)
        assert!(out_l < 1.5);
        assert!(out_r < 1.5);
        
        // Gain reduction should be positive
        assert!(limiter.gain_reduction_db > 0.0);
    }
}
