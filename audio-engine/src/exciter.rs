/// Exciter/Enhancer - Adds brightness and presence to audio
/// 
/// Uses waveshaping to generate harmonic content in the high frequencies,
/// adding perceived clarity and "air" without harsh EQ boosts.

use std::f32::consts::PI;

pub struct Exciter {
    sample_rate: f32,
    enabled: bool,
    
    // Parameters
    amount: f32,      // 0.0 - 1.0, intensity of effect
    frequency: f32,   // Crossover frequency in Hz (where effect starts)
    mix: f32,         // 0.0 - 1.0, wet/dry mix
    
    // High-pass filter for isolating high frequencies
    hp_state_l: [f32; 2], // x[n-1], x[n-2] for left
    hp_state_r: [f32; 2],
    hp_out_l: [f32; 2],   // y[n-1], y[n-2] for left
    hp_out_r: [f32; 2],
}

impl Exciter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            enabled: false,
            amount: 0.5,
            frequency: 3000.0,
            mix: 0.5,
            hp_state_l: [0.0; 2],
            hp_state_r: [0.0; 2],
            hp_out_l: [0.0; 2],
            hp_out_r: [0.0; 2],
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.hp_state_l.fill(0.0);
            self.hp_state_r.fill(0.0);
            self.hp_out_l.fill(0.0);
            self.hp_out_r.fill(0.0);
        }
    }
    
    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.max(0.0).min(1.0);
    }
    
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.max(1000.0).min(10000.0);
    }
    
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.max(0.0).min(1.0);
    }
    
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.hp_state_l.fill(0.0);
        self.hp_state_r.fill(0.0);
        self.hp_out_l.fill(0.0);
        self.hp_out_r.fill(0.0);
    }
    
    pub fn reset(&mut self) {
        self.hp_state_l.fill(0.0);
        self.hp_state_r.fill(0.0);
        self.hp_out_l.fill(0.0);
        self.hp_out_r.fill(0.0);
    }
    
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        // High-pass filter coefficients (Butterworth 2nd order)
        let omega = 2.0 * PI * self.frequency / self.sample_rate;
        let cos_omega = omega.cos();
        let sin_omega = omega.sin();
        let alpha = sin_omega / (2.0 * 0.707); // Q = 0.707 for Butterworth
        
        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;
        
        // Normalize
        let b0 = b0 / a0;
        let b1 = b1 / a0;
        let b2 = b2 / a0;
        let a1 = a1 / a0;
        let a2 = a2 / a0;
        
        // Apply high-pass filter to left channel
        let hp_l = b0 * left + b1 * self.hp_state_l[0] + b2 * self.hp_state_l[1]
                    - a1 * self.hp_out_l[0] - a2 * self.hp_out_l[1];
        
        self.hp_state_l[1] = self.hp_state_l[0];
        self.hp_state_l[0] = left;
        self.hp_out_l[1] = self.hp_out_l[0];
        self.hp_out_l[0] = hp_l;
        
        // Apply high-pass filter to right channel
        let hp_r = b0 * right + b1 * self.hp_state_r[0] + b2 * self.hp_state_r[1]
                    - a1 * self.hp_out_r[0] - a2 * self.hp_out_r[1];
        
        self.hp_state_r[1] = self.hp_state_r[0];
        self.hp_state_r[0] = right;
        self.hp_out_r[1] = self.hp_out_r[0];
        self.hp_out_r[0] = hp_r;
        
        // Apply harmonic enhancement using waveshaping
        // Soft saturation adds even and odd harmonics
        let drive = 1.0 + self.amount * 3.0;
        let enhanced_l = self.waveshape(hp_l * drive);
        let enhanced_r = self.waveshape(hp_r * drive);
        
        // Mix enhanced high frequencies back with original
        let wet_l = left + enhanced_l * self.amount;
        let wet_r = right + enhanced_r * self.amount;
        
        // Apply wet/dry mix
        let out_l = left * (1.0 - self.mix) + wet_l * self.mix;
        let out_r = right * (1.0 - self.mix) + wet_r * self.mix;
        
        (out_l, out_r)
    }
    
    /// Waveshaping function for harmonic generation
    /// Uses soft clipping for smooth, musical harmonics
    fn waveshape(&self, x: f32) -> f32 {
        // Soft saturation curve
        if x.abs() < 0.33 {
            2.0 * x
        } else if x.abs() < 0.67 {
            let sign = x.signum();
            sign * (3.0 - (2.0 - 3.0 * x.abs()).powi(2)) / 3.0
        } else {
            x.signum()
        }
    }
}
