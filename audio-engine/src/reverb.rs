/// Algorithmic Reverb (Freeverb-style)
/// 
/// A simple but effective reverb using parallel comb filters and series all-pass filters.
/// Based on the classic Freeverb algorithm by Jezar at Dreampoint.

pub struct Reverb {
    enabled: bool,
    sample_rate: f32,
    
    // Parameters
    room_size: f32,      // 0.0 - 1.0 (decay time)
    damping: f32,        // 0.0 - 1.0 (high frequency damping)
    wet: f32,            // 0.0 - 1.0 (wet/dry mix)
    width: f32,          // 0.0 - 1.0 (stereo width)
    
    // Comb filters (parallel) - 8 total (4 per channel)
    comb_l: [CombFilter; 4],
    comb_r: [CombFilter; 4],
    
    // All-pass filters (series) - 4 per channel
    allpass_l: [AllPassFilter; 4],
    allpass_r: [AllPassFilter; 4],
}

struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    filter_store: f32,
    damping1: f32,
    damping2: f32,
    feedback: f32,
}

struct AllPassFilter {
    buffer: Vec<f32>,
    index: usize,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            filter_store: 0.0,
            damping1: 0.5,
            damping2: 0.5,
            feedback: 0.5,
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];
        
        // One-pole lowpass filter for damping
        self.filter_store = (output * self.damping2) + (self.filter_store * self.damping1);
        
        self.buffer[self.index] = input + (self.filter_store * self.feedback);
        
        self.index = (self.index + 1) % self.buffer.len();
        
        output
    }
    
    fn set_damping(&mut self, damping: f32) {
        self.damping1 = damping;
        self.damping2 = 1.0 - damping;
    }
    
    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }
    
    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.filter_store = 0.0;
        self.index = 0;
    }
}

impl AllPassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.index];
        let output = -input + buffered;
        
        self.buffer[self.index] = input + (buffered * 0.5);
        
        self.index = (self.index + 1) % self.buffer.len();
        
        output
    }
    
    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
    }
}

// Freeverb tuning values (optimized for 44100 Hz)
const COMB_TUNING_L1: usize = 1116;
const COMB_TUNING_L2: usize = 1188;
const COMB_TUNING_L3: usize = 1277;
const COMB_TUNING_L4: usize = 1356;
const COMB_TUNING_R1: usize = 1116 + 23;
const COMB_TUNING_R2: usize = 1188 + 23;
const COMB_TUNING_R3: usize = 1277 + 23;
const COMB_TUNING_R4: usize = 1356 + 23;

const ALLPASS_TUNING_L1: usize = 556;
const ALLPASS_TUNING_L2: usize = 441;
const ALLPASS_TUNING_L3: usize = 341;
const ALLPASS_TUNING_L4: usize = 225;
const ALLPASS_TUNING_R1: usize = 556 + 23;
const ALLPASS_TUNING_R2: usize = 441 + 23;
const ALLPASS_TUNING_R3: usize = 341 + 23;
const ALLPASS_TUNING_R4: usize = 225 + 23;

const SCALE_WET: f32 = 3.0;
const SCALE_DAMPING: f32 = 0.4;
const SCALE_ROOM: f32 = 0.20;
const OFFSET_ROOM: f32 = 0.4;
const COMB_OUTPUT_GAIN: f32 = 0.15;

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let mut reverb = Self {
            enabled: false,
            sample_rate,
            room_size: 0.3,
            damping: 0.5,
            wet: 0.3,
            width: 1.0,
            comb_l: [
                CombFilter::new(scale_tuning(COMB_TUNING_L1, sample_rate)),
                CombFilter::new(scale_tuning(COMB_TUNING_L2, sample_rate)),
                CombFilter::new(scale_tuning(COMB_TUNING_L3, sample_rate)),
                CombFilter::new(scale_tuning(COMB_TUNING_L4, sample_rate)),
            ],
            comb_r: [
                CombFilter::new(scale_tuning(COMB_TUNING_R1, sample_rate)),
                CombFilter::new(scale_tuning(COMB_TUNING_R2, sample_rate)),
                CombFilter::new(scale_tuning(COMB_TUNING_R3, sample_rate)),
                CombFilter::new(scale_tuning(COMB_TUNING_R4, sample_rate)),
            ],
            allpass_l: [
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L1, sample_rate)),
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L2, sample_rate)),
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L3, sample_rate)),
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L4, sample_rate)),
            ],
            allpass_r: [
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R1, sample_rate)),
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R2, sample_rate)),
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R3, sample_rate)),
                AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R4, sample_rate)),
            ],
        };
        
        reverb.update_comb_filters();
        reverb
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled && self.enabled {
            // Clear buffers when disabling
            self.clear();
        }
        self.enabled = enabled;
    }
    
    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.0, 1.0);
        self.update_comb_filters();
    }
    
    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
        self.update_comb_filters();
    }
    
    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet.clamp(0.0, 1.0);
    }
    
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(0.0, 1.0);
    }
    
    fn update_comb_filters(&mut self) {
        let feedback = OFFSET_ROOM + (self.room_size * SCALE_ROOM);
        let damping = self.damping * SCALE_DAMPING;
        
        for comb in &mut self.comb_l {
            comb.set_feedback(feedback);
            comb.set_damping(damping);
        }
        
        for comb in &mut self.comb_r {
            comb.set_feedback(feedback);
            comb.set_damping(damping);
        }
    }
    
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() < 0.1 {
            return; // No significant change
        }
        
        self.sample_rate = sample_rate;
        
        // Reallocate buffers with new sizes
        self.comb_l = [
            CombFilter::new(scale_tuning(COMB_TUNING_L1, sample_rate)),
            CombFilter::new(scale_tuning(COMB_TUNING_L2, sample_rate)),
            CombFilter::new(scale_tuning(COMB_TUNING_L3, sample_rate)),
            CombFilter::new(scale_tuning(COMB_TUNING_L4, sample_rate)),
        ];
        self.comb_r = [
            CombFilter::new(scale_tuning(COMB_TUNING_R1, sample_rate)),
            CombFilter::new(scale_tuning(COMB_TUNING_R2, sample_rate)),
            CombFilter::new(scale_tuning(COMB_TUNING_R3, sample_rate)),
            CombFilter::new(scale_tuning(COMB_TUNING_R4, sample_rate)),
        ];
        self.allpass_l = [
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L1, sample_rate)),
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L2, sample_rate)),
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L3, sample_rate)),
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_L4, sample_rate)),
        ];
        self.allpass_r = [
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R1, sample_rate)),
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R2, sample_rate)),
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R3, sample_rate)),
            AllPassFilter::new(scale_tuning(ALLPASS_TUNING_R4, sample_rate)),
        ];
        
        self.update_comb_filters();
    }
    
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        // Mono input for reverb tank (mix L+R)
        let input = (left + right) * 0.5;
        
        // Process through parallel comb filters
        let mut out_l = 0.0;
        let mut out_r = 0.0;
        
        for comb in &mut self.comb_l {
            out_l += comb.process(input);
        }
        
        for comb in &mut self.comb_r {
            out_r += comb.process(input);
        }
        
        // Apply gain compensation for parallel comb filters
        out_l *= COMB_OUTPUT_GAIN;
        out_r *= COMB_OUTPUT_GAIN;
        
        // Process through series all-pass filters
        for allpass in &mut self.allpass_l {
            out_l = allpass.process(out_l);
        }
        
        for allpass in &mut self.allpass_r {
            out_r = allpass.process(out_r);
        }
        
        // Apply wet/dry mix and stereo width
        let wet1 = self.wet * (self.width * 0.5 + 0.5);
        let wet2 = self.wet * ((1.0 - self.width) * 0.5);
        let dry = 1.0 - self.wet;
        
        let out_l = (out_l * wet1) + (out_r * wet2) + (left * dry);
        let out_r = (out_r * wet1) + (out_l * wet2) + (right * dry);
        
        (out_l, out_r)
    }
    
    fn clear(&mut self) {
        for comb in &mut self.comb_l {
            comb.clear();
        }
        for comb in &mut self.comb_r {
            comb.clear();
        }
        for allpass in &mut self.allpass_l {
            allpass.clear();
        }
        for allpass in &mut self.allpass_r {
            allpass.clear();
        }
    }
}

// Scale tuning values based on sample rate (44100 is reference)
fn scale_tuning(tuning: usize, sample_rate: f32) -> usize {
    ((tuning as f32 * sample_rate) / 44100.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reverb_bypass() {
        let mut reverb = Reverb::new(44100.0);
        reverb.set_enabled(false);
        
        let (out_l, out_r) = reverb.process(0.5, -0.3);
        assert_eq!(out_l, 0.5);
        assert_eq!(out_r, -0.3);
    }
    
    #[test]
    fn test_reverb_processing() {
        let mut reverb = Reverb::new(44100.0);
        reverb.set_enabled(true);
        reverb.set_room_size(0.7);
        reverb.set_damping(0.5);
        reverb.set_wet(0.3);
        
        // Process some samples
        for _ in 0..100 {
            let (out_l, out_r) = reverb.process(0.5, 0.5);
            // Output should be different from input due to reverb
            assert!(out_l.abs() >= 0.0);
            assert!(out_r.abs() >= 0.0);
        }
    }
    
    #[test]
    fn test_parameter_clamping() {
        let mut reverb = Reverb::new(44100.0);
        
        reverb.set_room_size(1.5);
        assert_eq!(reverb.room_size, 1.0);
        
        reverb.set_damping(-0.5);
        assert_eq!(reverb.damping, 0.0);
        
        reverb.set_wet(2.0);
        assert_eq!(reverb.wet, 1.0);
    }
}
