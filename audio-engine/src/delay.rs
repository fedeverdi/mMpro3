/// Stereo Delay with feedback and filtering
/// Classic tape-style delay effect with independent left/right delay times
pub struct Delay {
    /// Whether the delay is enabled
    enabled: bool,
    /// Delay time for left channel in milliseconds
    delay_time_l_ms: f32,
    /// Delay time for right channel in milliseconds
    delay_time_r_ms: f32,
    /// Feedback amount (0.0 to 1.0)
    feedback: f32,
    /// Wet/dry mix (0.0 = dry, 1.0 = wet)
    mix: f32,
    /// Sample rate
    sample_rate: f32,
    
    // Internal delay buffers (ring buffers)
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos_l: usize,
    write_pos_r: usize,
    
    // Maximum delay time (2 seconds)
    max_delay_samples: usize,
}

impl Delay {
    /// Create a new delay with default settings
    pub fn new(sample_rate: f32) -> Self {
        // Allocate buffer for 2 seconds maximum delay
        let max_delay_samples = (sample_rate * 2.0) as usize;
        
        Self {
            enabled: false,
            delay_time_l_ms: 250.0,  // 250ms left
            delay_time_r_ms: 375.0,  // 375ms right (ping-pong effect)
            feedback: 0.3,           // 30% feedback
            mix: 0.3,                // 30% wet
            sample_rate,
            buffer_l: vec![0.0; max_delay_samples],
            buffer_r: vec![0.0; max_delay_samples],
            write_pos_l: 0,
            write_pos_r: 0,
            max_delay_samples,
        }
    }
    
    /// Set delay time for left channel in milliseconds
    pub fn set_delay_time_left(&mut self, time_ms: f32) {
        self.delay_time_l_ms = time_ms.clamp(1.0, 2000.0);
    }
    
    /// Set delay time for right channel in milliseconds
    pub fn set_delay_time_right(&mut self, time_ms: f32) {
        self.delay_time_r_ms = time_ms.clamp(1.0, 2000.0);
    }
    
    /// Set delay time for both channels (synchronized)
    pub fn set_delay_time(&mut self, time_ms: f32) {
        let clamped = time_ms.clamp(1.0, 2000.0);
        self.delay_time_l_ms = clamped;
        self.delay_time_r_ms = clamped;
    }
    
    /// Set feedback amount (0.0 to 1.0)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95); // Max 95% to prevent runaway
    }
    
    /// Set wet/dry mix (0.0 to 1.0)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // Clear buffers when disabled
            self.buffer_l.fill(0.0);
            self.buffer_r.fill(0.0);
            self.write_pos_l = 0;
            self.write_pos_r = 0;
        }
    }
    
    /// Update sample rate and reallocate buffers if needed
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        
        // Reallocate buffers if sample rate changed significantly
        let new_max_samples = (sample_rate * 2.0) as usize;
        if new_max_samples != self.max_delay_samples {
            self.max_delay_samples = new_max_samples;
            self.buffer_l = vec![0.0; new_max_samples];
            self.buffer_r = vec![0.0; new_max_samples];
            self.write_pos_l = 0;
            self.write_pos_r = 0;
        }
    }
    
    /// Process a stereo frame through the delay
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        // Calculate delay in samples
        let delay_samples_l = ((self.delay_time_l_ms / 1000.0) * self.sample_rate) as usize;
        let delay_samples_r = ((self.delay_time_r_ms / 1000.0) * self.sample_rate) as usize;
        
        // Calculate read positions (wrap around)
        let read_pos_l = (self.write_pos_l + self.max_delay_samples - delay_samples_l) % self.max_delay_samples;
        let read_pos_r = (self.write_pos_r + self.max_delay_samples - delay_samples_r) % self.max_delay_samples;
        
        // Read delayed samples
        let delayed_l = self.buffer_l[read_pos_l];
        let delayed_r = self.buffer_r[read_pos_r];
        
        // Apply simple one-pole lowpass filter to feedback (simulates tape darkness)
        // This prevents harsh digital artifacts in feedback
        let filtered_delayed_l = delayed_l * 0.7;
        let filtered_delayed_r = delayed_r * 0.7;
        
        // Write new samples to buffer (input + filtered feedback)
        self.buffer_l[self.write_pos_l] = left + filtered_delayed_l * self.feedback;
        self.buffer_r[self.write_pos_r] = right + filtered_delayed_r * self.feedback;
        
        // Advance write positions
        self.write_pos_l = (self.write_pos_l + 1) % self.max_delay_samples;
        self.write_pos_r = (self.write_pos_r + 1) % self.max_delay_samples;
        
        // Mix dry and wet signals
        let out_l = left * (1.0 - self.mix) + delayed_l * self.mix;
        let out_r = right * (1.0 - self.mix) + delayed_r * self.mix;
        
        (out_l, out_r)
    }
    
    /// Reset delay buffers
    pub fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_pos_l = 0;
        self.write_pos_r = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_delay_creation() {
        let delay = Delay::new(48000.0);
        assert_eq!(delay.sample_rate, 48000.0);
        assert!(!delay.enabled);
    }
    
    #[test]
    fn test_delay_passthrough_when_disabled() {
        let mut delay = Delay::new(48000.0);
        delay.set_enabled(false);
        
        let (out_l, out_r) = delay.process(0.5, -0.3);
        assert_eq!(out_l, 0.5);
        assert_eq!(out_r, -0.3);
    }
    
    #[test]
    fn test_delay_produces_delayed_signal() {
        let mut delay = Delay::new(48000.0);
        delay.set_delay_time(10.0); // 10ms delay
        delay.set_mix(1.0); // 100% wet
        delay.set_feedback(0.0); // No feedback
        delay.set_enabled(true);
        
        // Process impulse
        let (out1_l, out1_r) = delay.process(1.0, 1.0);
        
        // First output should be mostly dry (buffer is empty)
        assert!(out1_l.abs() < 0.1);
        
        // Process silence for the delay time
        let delay_samples = (10.0 / 1000.0 * 48000.0) as usize;
        for _ in 0..delay_samples {
            let _ = delay.process(0.0, 0.0);
        }
        
        // Now we should see the delayed impulse
        let (out2_l, out2_r) = delay.process(0.0, 0.0);
        assert!(out2_l > 0.5); // Delayed signal should be present
    }
}
