/// Chorus - Classic modulated delay effect
/// 
/// Creates a thicker, doubled sound by mixing the original signal
/// with slightly detuned and delayed copies using LFO modulation.

use std::f32::consts::PI;

pub struct Chorus {
    sample_rate: f32,
    enabled: bool,
    
    // Parameters
    rate: f32,        // LFO rate in Hz (0.1 - 5.0)
    depth: f32,       // Modulation depth (0.0 - 1.0)
    mix: f32,         // Wet/dry mix (0.0 - 1.0)
    
    // Delay buffers
    delay_buffer_l: Vec<f32>,
    delay_buffer_r: Vec<f32>,
    write_pos: usize,
    
    // LFO state
    lfo_phase: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        // Max delay time: 20ms
        let max_delay_samples = (sample_rate * 0.02) as usize;
        
        Self {
            sample_rate,
            enabled: false,
            rate: 0.5,
            depth: 0.5,
            mix: 0.5,
            delay_buffer_l: vec![0.0; max_delay_samples],
            delay_buffer_r: vec![0.0; max_delay_samples],
            write_pos: 0,
            lfo_phase: 0.0,
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.delay_buffer_l.fill(0.0);
            self.delay_buffer_r.fill(0.0);
            self.write_pos = 0;
            self.lfo_phase = 0.0;
        }
    }
    
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.max(0.1).min(5.0);
    }
    
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.max(0.0).min(1.0);
    }
    
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.max(0.0).min(1.0);
    }
    
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        let max_delay_samples = (sample_rate * 0.02) as usize;
        self.delay_buffer_l = vec![0.0; max_delay_samples];
        self.delay_buffer_r = vec![0.0; max_delay_samples];
        self.write_pos = 0;
        self.lfo_phase = 0.0;
    }
    
    pub fn reset(&mut self) {
        self.delay_buffer_l.fill(0.0);
        self.delay_buffer_r.fill(0.0);
        self.write_pos = 0;
        self.lfo_phase = 0.0;
    }
    
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        // Write to delay buffers
        self.delay_buffer_l[self.write_pos] = left;
        self.delay_buffer_r[self.write_pos] = right;
        
        // Advance LFO
        let lfo_increment = 2.0 * PI * self.rate / self.sample_rate;
        self.lfo_phase += lfo_increment;
        if self.lfo_phase >= 2.0 * PI {
            self.lfo_phase -= 2.0 * PI;
        }
        
        // LFO generates sine wave (0.0 to 1.0)
        let lfo = (self.lfo_phase.sin() + 1.0) * 0.5;
        
        // Calculate delay time based on LFO
        // Base delay: 5ms, modulation range: 0-15ms
        let base_delay_ms = 0.005;
        let mod_range_ms = 0.015;
        let delay_time_ms = base_delay_ms + mod_range_ms * lfo * self.depth;
        let delay_samples = (delay_time_ms * self.sample_rate) as f32;
        
        // Calculate read position with linear interpolation
        let read_pos_float = (self.write_pos as f32 + self.delay_buffer_l.len() as f32 - delay_samples) 
                             % self.delay_buffer_l.len() as f32;
        
        let read_pos = read_pos_float as usize;
        let frac = read_pos_float - read_pos as f32;
        let next_pos = (read_pos + 1) % self.delay_buffer_l.len();
        
        // Interpolated read from left buffer
        let delayed_l = self.delay_buffer_l[read_pos] * (1.0 - frac) 
                      + self.delay_buffer_l[next_pos] * frac;
        
        // Interpolated read from right buffer (with phase offset for stereo width)
        let lfo_r_phase = (self.lfo_phase + PI) % (2.0 * PI);
        let lfo_r = (lfo_r_phase.sin() + 1.0) * 0.5;
        let delay_time_r_ms = base_delay_ms + mod_range_ms * lfo_r * self.depth;
        let delay_samples_r = (delay_time_r_ms * self.sample_rate) as f32;
        
        let read_pos_r_float = (self.write_pos as f32 + self.delay_buffer_r.len() as f32 - delay_samples_r) 
                               % self.delay_buffer_r.len() as f32;
        
        let read_pos_r = read_pos_r_float as usize;
        let frac_r = read_pos_r_float - read_pos_r as f32;
        let next_pos_r = (read_pos_r + 1) % self.delay_buffer_r.len();
        
        let delayed_r = self.delay_buffer_r[read_pos_r] * (1.0 - frac_r) 
                      + self.delay_buffer_r[next_pos_r] * frac_r;
        
        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.delay_buffer_l.len();
        
        // Mix dry and wet signals
        let out_l = left * (1.0 - self.mix) + delayed_l * self.mix;
        let out_r = right * (1.0 - self.mix) + delayed_r * self.mix;
        
        (out_l, out_r)
    }
}
