/// Dynamic range compressor
/// Reduces the dynamic range of audio by attenuating signals above a threshold
pub struct Compressor {
    /// Threshold in dB (signals above this are compressed)
    threshold_db: f32,
    /// Compression ratio (e.g., 4.0 means 4:1 compression)
    ratio: f32,
    /// Attack time in milliseconds (how fast compression engages)
    attack_ms: f32,
    /// Release time in milliseconds (how fast compression disengages)
    release_ms: f32,
    /// Whether the compressor is enabled
    enabled: bool,
    /// Sample rate for time constant calculations
    sample_rate: f32,
    
    // Internal state
    /// Current envelope level (in linear amplitude)
    envelope: f32,
    /// Attack coefficient (calculated from attack_ms)
    attack_coeff: f32,
    /// Release coefficient (calculated from release_ms)
    release_coeff: f32,
    
    /// Current gain reduction in dB (for metering/visualization)
    pub gain_reduction_db: f32,
    /// Current input level in dB (for visualization on curve)
    pub input_level_db: f32,
}

impl Compressor {
    /// Create a new compressor with default settings
    pub fn new(sample_rate: f32) -> Self {
        let mut compressor = Self {
            threshold_db: -20.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            enabled: false,
            sample_rate,
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            gain_reduction_db: 0.0,
            input_level_db: -90.0,
        };
        
        compressor.update_coefficients();
        compressor
    }
    
    /// Update attack and release coefficients based on time constants
    fn update_coefficients(&mut self) {
        // Convert milliseconds to time constant (tau)
        // Coefficient = exp(-1 / (tau * sample_rate))
        // tau = time_ms / 1000
        
        let attack_tau = self.attack_ms / 1000.0;
        let release_tau = self.release_ms / 1000.0;
        
        self.attack_coeff = (-1.0 / (attack_tau * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (release_tau * self.sample_rate)).exp();
    }
    
    /// Set threshold in dB
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db;
    }
    
    /// Set compression ratio
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0); // Minimum 1:1 (no compression)
    }
    
    /// Set attack time in milliseconds
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms.max(0.1);
        self.update_coefficients();
    }
    
    /// Set release time in milliseconds
    pub fn set_release(&mut self, release_ms: f32) {
        self.release_ms = release_ms.max(1.0);
        self.update_coefficients();
    }
    
    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // Reset envelope when disabled
            self.envelope = 0.0;
            self.gain_reduction_db = 0.0;
            self.input_level_db = -90.0;
        }
    }
    
    /// Update sample rate and recalculate coefficients
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }
    
    /// Process a stereo frame through the compressor
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            self.gain_reduction_db = 0.0;
            return (left, right);
        }
        
        // Calculate the peak level of the stereo signal (in linear amplitude)
        let input_level = left.abs().max(right.abs());
        
        // Update envelope follower (smooth the level detection)
        let coeff = if input_level > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
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
        
        // Calculate gain reduction
        let mut gain_reduction_db = 0.0;
        
        if envelope_db > self.threshold_db {
            // Signal is above threshold, apply compression
            let overshoot_db = envelope_db - self.threshold_db;
            // Gain reduction = overshoot * (1 - 1/ratio)
            gain_reduction_db = overshoot_db * (1.0 - 1.0 / self.ratio);
        }
        
        // Store for metering
        self.gain_reduction_db = gain_reduction_db;
        
        // Convert gain reduction to linear gain
        let gain = 10.0_f32.powf(-gain_reduction_db / 20.0);
        
        // Apply gain to both channels
        (left * gain, right * gain)
    }
    
    /// Reset compressor state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain_reduction_db = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compressor_creation() {
        let comp = Compressor::new(44100.0);
        assert_eq!(comp.sample_rate, 44100.0);
        assert!(!comp.enabled);
    }
    
    #[test]
    fn test_compressor_passthrough_when_disabled() {
        let mut comp = Compressor::new(44100.0);
        comp.set_enabled(false);
        
        let (out_l, out_r) = comp.process(0.5, -0.3);
        assert_eq!(out_l, 0.5);
        assert_eq!(out_r, -0.3);
    }
    
    #[test]
    fn test_compressor_reduces_loud_signals() {
        let mut comp = Compressor::new(44100.0);
        comp.set_threshold(-20.0);
        comp.set_ratio(4.0);
        comp.set_attack(1.0);
        comp.set_release(100.0);
        comp.set_enabled(true);
        
        // Process a loud signal multiple times to let envelope build up
        let mut out_l = 0.8;
        let mut out_r = 0.8;
        for _ in 0..100 {
            (out_l, out_r) = comp.process(0.8, 0.8);
        }
        
        // Output should be reduced (less than input)
        assert!(out_l < 0.8);
        assert!(out_r < 0.8);
        
        // Gain reduction should be positive (indicating compression)
        assert!(comp.gain_reduction_db > 0.0);
    }
}
