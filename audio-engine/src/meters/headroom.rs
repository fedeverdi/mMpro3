// Headroom Meter - Measures distance from 0 dBFS (clipping)
// Monitors peak levels and calculates available headroom

/// Headroom measurement result
#[derive(Debug, Clone, Copy)]
pub struct HeadroomMeasurement {
    pub peak_l: f32,           // Peak level left channel (dBFS)
    pub peak_r: f32,           // Peak level right channel (dBFS)
    pub headroom_l: f32,       // Headroom left (dB from 0 dBFS)
    pub headroom_r: f32,       // Headroom right (dB from 0 dBFS)
    pub headroom_stereo: f32,  // Minimum headroom (worst case)
}

/// Headroom meter with peak tracking
pub struct HeadroomMeter {
    // Peak tracking with decay
    current_peak_l: f32,
    current_peak_r: f32,
    
    // Hold peaks for measurement
    hold_peak_l: f32,
    hold_peak_r: f32,
    
    // Hold time counter (samples)
    hold_counter: usize,
    hold_duration: usize, // Hold peaks for 1 second
    
    // Decay rate for peak following
    decay_rate: f32, // dB/sample decay after hold
    
    sample_rate: f32,
}

impl HeadroomMeter {
    /// Create new headroom meter
    pub fn new(sample_rate: f32) -> Self {
        let hold_duration = (sample_rate * 0.2) as usize; // 200ms hold - fast response
        Self {
            current_peak_l: 0.0,
            current_peak_r: 0.0,
            hold_peak_l: 0.0,
            hold_peak_r: 0.0,
            hold_counter: 0,
            hold_duration,
            decay_rate: 48.0 / sample_rate, // 48 dB/sec decay rate - fast decay
            sample_rate,
        }
    }
    
    /// Process a sample pair (left, right)
    pub fn process(&mut self, left: f32, right: f32) {
        let abs_l = left.abs();
        let abs_r = right.abs();
        
        // Update current peaks
        if abs_l > self.current_peak_l {
            self.current_peak_l = abs_l;
            self.hold_counter = 0; // Reset hold timer
        }
        if abs_r > self.current_peak_r {
            self.current_peak_r = abs_r;
            self.hold_counter = 0;
        }
        
        // Apply decay after hold period
        self.hold_counter += 1;
        if self.hold_counter > self.hold_duration {
            // Convert to dB, apply decay, convert back to linear
            let peak_l_db = self.amplitude_to_db(self.current_peak_l);
            let peak_r_db = self.amplitude_to_db(self.current_peak_r);
            
            let decayed_l_db = peak_l_db - self.decay_rate;
            let decayed_r_db = peak_r_db - self.decay_rate;
            
            self.current_peak_l = self.db_to_amplitude(decayed_l_db);
            self.current_peak_r = self.db_to_amplitude(decayed_r_db);
        }
    }
    
    /// Get current headroom measurement
    pub fn get_measurement(&self) -> HeadroomMeasurement {
        // Convert peaks to dBFS (use current_peak directly, not hold_peak)
        let peak_l_db = self.amplitude_to_db(self.current_peak_l);
        let peak_r_db = self.amplitude_to_db(self.current_peak_r);
        
        // Calculate headroom (distance from 0 dBFS)
        let headroom_l = 0.0 - peak_l_db;
        let headroom_r = 0.0 - peak_r_db;
        
        // Stereo headroom is the minimum (worst case)
        let headroom_stereo = headroom_l.min(headroom_r);
        
        HeadroomMeasurement {
            peak_l: peak_l_db,
            peak_r: peak_r_db,
            headroom_l,
            headroom_r,
            headroom_stereo,
        }
    }
    
    /// Reset all measurements
    pub fn reset(&mut self) {
        self.current_peak_l = 0.0;
        self.current_peak_r = 0.0;
        self.hold_peak_l = 0.0;
        self.hold_peak_r = 0.0;
        self.hold_counter = 0;
    }
    
    /// Convert linear amplitude to dBFS
    fn amplitude_to_db(&self, amplitude: f32) -> f32 {
        if amplitude > 0.0 {
            20.0 * amplitude.log10()
        } else {
            -90.0 // Floor at -90 dBFS
        }
    }
    
    /// Convert dBFS to linear amplitude
    fn db_to_amplitude(&self, db: f32) -> f32 {
        if db > -90.0 {
            10.0_f32.powf(db / 20.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headroom_calculation() {
        let mut meter = HeadroomMeter::new(48000.0);
        
        // Process a peak at -6 dBFS (0.5 linear)
        for _ in 0..1000 {
            meter.process(0.5, 0.5);
        }
        
        let measurement = meter.get_measurement();
        
        // Peak should be around -6 dBFS
        assert!((measurement.peak_l - (-6.0)).abs() < 0.1);
        
        // Headroom should be around 6 dB
        assert!((measurement.headroom_stereo - 6.0).abs() < 0.1);
    }
    
    #[test]
    fn test_headroom_reset() {
        let mut meter = HeadroomMeter::new(48000.0);
        
        // Process some peaks
        meter.process(0.8, 0.8);
        
        // Reset
        meter.reset();
        
        let measurement = meter.get_measurement();
        
        // Should be at floor
        assert!(measurement.peak_l < -80.0);
        assert!(measurement.headroom_stereo > 80.0);
    }
}
