/// Noise Gate - Reduces signal level when below threshold
/// Used to eliminate background noise and unwanted low-level signals
/// 
/// The gate works opposite to a compressor:
/// - When signal is ABOVE threshold: passes through unchanged
/// - When signal is BELOW threshold: reduces level by the range amount
/// 
/// Processing chain position: After gain, before volume fader

pub struct NoiseGate {
    enabled: bool,
    threshold_db: f32,    // Signal below this gets attenuated
    range_db: f32,        // How much to attenuate when below threshold (typically -80dB)
    attack_ms: f32,       // How fast gate opens (signal goes above threshold)
    release_ms: f32,      // How fast gate closes (signal goes below threshold)
    
    // Internal state
    sample_rate: f32,
    envelope: f32,        // Smoothed signal level (linear)
    attack_coeff: f32,    // Exponential smoothing coefficient for attack
    release_coeff: f32,   // Exponential smoothing coefficient for release
    
    // Metering
    pub attenuation_db: f32,  // Current attenuation being applied
    pub input_level_db: f32,  // Input signal level for visualization
}

impl NoiseGate {
    pub fn new(sample_rate: f32) -> Self {
        let mut gate = Self {
            enabled: false,
            threshold_db: -40.0,
            range_db: -80.0,
            attack_ms: 10.0,     // Slower attack = less distortion on transients
            release_ms: 150.0,   // Longer release = smoother closing
            sample_rate,
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            attenuation_db: 0.0,
            input_level_db: -90.0,
        };
        gate.update_coefficients();
        gate
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        // Convert time constants to exponential coefficients
        // tau = time_ms / 1000, coeff = exp(-1 / (tau * sample_rate))
        let attack_tau = self.attack_ms / 1000.0;
        let release_tau = self.release_ms / 1000.0;
        
        self.attack_coeff = (-1.0 / (attack_tau * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (release_tau * self.sample_rate)).exp();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.attenuation_db = 0.0;
            self.input_level_db = -90.0;
        }
    }

    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db.clamp(-80.0, 0.0);
    }

    pub fn set_range(&mut self, range_db: f32) {
        self.range_db = range_db.clamp(-100.0, 0.0);
    }

    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms.clamp(0.1, 100.0);
        self.update_coefficients();
    }

    pub fn set_release(&mut self, release_ms: f32) {
        self.release_ms = release_ms.clamp(1.0, 5000.0);
        self.update_coefficients();
    }

    /// Process a stereo frame through the gate
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            self.attenuation_db = 0.0;
            return (left, right);
        }
        
        // Calculate the peak level of the stereo signal (in linear amplitude)
        let input_level = left.abs().max(right.abs());
        
        // Update envelope follower (smooth the level detection)
        let coeff = if input_level > self.envelope {
            self.attack_coeff  // Opening gate (signal rising)
        } else {
            self.release_coeff // Closing gate (signal falling)
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
        
        // Calculate attenuation
        let mut attenuation_db = 0.0;
        
        if envelope_db < self.threshold_db {
            // Signal is below threshold, apply attenuation
            // Smooth transition from 0dB at threshold to range_db below threshold
            let below_threshold = self.threshold_db - envelope_db;
            
            // Simple linear fade (could be made more sophisticated)
            // If we're far below threshold, apply full range
            // If we're just below, fade in the attenuation
            attenuation_db = self.range_db.max(-below_threshold).max(self.range_db);
        }
        
        // Store for metering
        self.attenuation_db = attenuation_db;
        
        // Convert attenuation from dB to linear gain
        let gain_linear = if attenuation_db <= -90.0 {
            0.0
        } else {
            10.0_f32.powf(attenuation_db / 20.0)
        };
        
        // Apply gain to both channels
        let left_out = left * gain_linear;
        let right_out = right * gain_linear;
        
        (left_out, right_out)
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.attenuation_db = 0.0;
        self.input_level_db = -90.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_creation() {
        let gate = NoiseGate::new(48000.0);
        assert!(!gate.enabled);
        assert_eq!(gate.threshold_db, -40.0);
    }

    #[test]
    fn test_gate_passes_loud_signal() {
        let mut gate = NoiseGate::new(48000.0);
        gate.set_enabled(true);
        gate.set_threshold(-20.0);
        
        // Process loud signal (should pass through)
        let (left, right) = gate.process(0.5, 0.5);
        assert!(left.abs() > 0.4); // Should be mostly unchanged
        assert!(right.abs() > 0.4);
    }

    #[test]
    fn test_gate_attenuates_quiet_signal() {
        let mut gate = NoiseGate::new(48000.0);
        gate.set_enabled(true);
        gate.set_threshold(-20.0);
        gate.set_range(-80.0);
        
        // Process quiet signal multiple times to let envelope settle
        for _ in 0..1000 {
            let _ = gate.process(0.001, 0.001);
        }
        
        // After settling, attenuation should be applied
        assert!(gate.attenuation_db < -10.0);
    }
}
