/// Professional Algorithmic Reverb (Lexicon PCM70-style)
/// 
/// High-quality reverb algorithm inspired by the Lexicon PCM70.
/// Features:
/// - Pre-delay with variable time
/// - Early reflections for spatial realism
/// - Modulated diffusion network to prevent metallic coloration
/// - Feedback Delay Network (FDN) with Hadamard matrix
/// - Sophisticated damping filters
/// - High density late reverb

pub struct Reverb {
    enabled: bool,
    sample_rate: f32,
    
    // Parameters
    decay_time: f32,     // 0.1 - 20.0 seconds
    pre_delay: f32,      // 0.0 - 0.5 seconds
    diffusion: f32,      // 0.0 - 1.0 (diffusion amount)
    damping: f32,        // 0.0 - 1.0 (high frequency damping)
    wet: f32,            // 0.0 - 1.0 (wet/dry mix)
    width: f32,          // 0.0 - 1.0 (stereo width)
    
    // Legacy parameter mapping (for compatibility)
    room_size: f32,      // Maps to decay_time
    
    // Pre-delay buffers
    pre_delay_l: DelayLine,
    pre_delay_r: DelayLine,
    
    // Early reflections
    early_l: EarlyReflections,
    early_r: EarlyReflections,
    
    // Input diffusion (4 modulated all-pass filters per channel)
    input_diffuser_l: [ModulatedAllPass; 4],
    input_diffuser_r: [ModulatedAllPass; 4],
    
    // Feedback Delay Network (8 delay lines with cross-feedback)
    fdn_delays: [DelayLine; 8],
    fdn_dampers: [DampingFilter; 8],
    fdn_outputs: [f32; 8],
    
    // Output diffusion
    output_diffuser_l: [AllPassFilter; 2],
    output_diffuser_r: [AllPassFilter; 2],
    
    // LFO for modulation
    lfo_phase: [f32; 8],
    lfo_rate: [f32; 8],
}

// ============================================================================
// DELAY LINE
// ============================================================================

struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    max_delay_samples: usize,
}

impl DelayLine {
    fn new(max_delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_delay_samples + 1],
            write_pos: 0,
            max_delay_samples,
        }
    }
    
    fn write(&mut self, input: f32) {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }
    
    fn read(&self, delay_samples: usize) -> f32 {
        let delay = delay_samples.min(self.max_delay_samples);
        let read_pos = (self.write_pos + self.buffer.len() - delay) % self.buffer.len();
        self.buffer[read_pos]
    }
    
    fn read_interpolated(&self, delay_samples: f32) -> f32 {
        let delay = delay_samples.min(self.max_delay_samples as f32);
        let delay_int = delay.floor() as usize;
        let frac = delay - delay_int as f32;
        
        let read_pos1 = (self.write_pos + self.buffer.len() - delay_int) % self.buffer.len();
        let read_pos2 = (read_pos1 + self.buffer.len() - 1) % self.buffer.len();
        
        // Linear interpolation
        self.buffer[read_pos1] * (1.0 - frac) + self.buffer[read_pos2] * frac
    }
    
    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

// ============================================================================
// EARLY REFLECTIONS
// ============================================================================

struct EarlyReflections {
    taps: Vec<(usize, f32)>, // (delay_samples, gain)
    delay: DelayLine,
}

impl EarlyReflections {
    fn new(sample_rate: f32, is_right: bool) -> Self {
        // Early reflection pattern inspired by realistic room acoustics
        // Times in milliseconds, converted to samples
        let ms_to_samples = |ms: f32| (ms * sample_rate / 1000.0) as usize;
        
        // Slightly different patterns for L/R to create spatial width
        let offset = if is_right { 0.3 } else { 0.0 };
        
        let taps = vec![
            (ms_to_samples(15.0 + offset), 0.8),
            (ms_to_samples(22.5 + offset), -0.6),
            (ms_to_samples(28.3 + offset), 0.5),
            (ms_to_samples(35.7 + offset), -0.45),
            (ms_to_samples(41.2 + offset), 0.4),
            (ms_to_samples(48.8 + offset), -0.35),
            (ms_to_samples(55.0 + offset), 0.3),
            (ms_to_samples(63.5 + offset), -0.25),
        ];
        
        let max_delay = taps.iter().map(|(d, _)| *d).max().unwrap_or(0) + 100;
        
        Self {
            taps,
            delay: DelayLine::new(max_delay),
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        self.delay.write(input);
        
        let mut output = 0.0;
        for &(delay, gain) in &self.taps {
            output += self.delay.read(delay) * gain;
        }
        
        output * 0.3 // Scale down to prevent clipping
    }
    
    fn clear(&mut self) {
        self.delay.clear();
    }
}

// ============================================================================
// ALLPASS FILTER (Static)
// ============================================================================

struct AllPassFilter {
    buffer: Vec<f32>,
    index: usize,
    gain: f32,
}

impl AllPassFilter {
    fn new(size: usize, gain: f32) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            gain,
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.index];
        let output = -input + buffered;
        
        self.buffer[self.index] = input + (buffered * self.gain);
        
        self.index = (self.index + 1) % self.buffer.len();
        
        output
    }
    
    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
    }
}

// ============================================================================
// MODULATED ALLPASS FILTER
// ============================================================================

struct ModulatedAllPass {
    delay: DelayLine,
    base_delay: usize,
    mod_depth: f32,
    gain: f32,
}

impl ModulatedAllPass {
    fn new(base_delay: usize, mod_depth_samples: f32, gain: f32) -> Self {
        let max_delay = base_delay + mod_depth_samples.ceil() as usize + 1;
        
        Self {
            delay: DelayLine::new(max_delay),
            base_delay,
            mod_depth: mod_depth_samples,
            gain,
        }
    }
    
    fn process(&mut self, input: f32, lfo: f32) -> f32 {
        // LFO modulates delay time (range: -1 to 1)
        let delay_samples = self.base_delay as f32 + (lfo * self.mod_depth);
        
        let buffered = self.delay.read_interpolated(delay_samples);
        let output = -input + buffered;
        
        self.delay.write(input + (buffered * self.gain));
        
        output
    }
    
    fn clear(&mut self) {
        self.delay.clear();
    }
}

// ============================================================================
// DAMPING FILTER (One-pole lowpass + shelving)
// ============================================================================

struct DampingFilter {
    lowpass_state: f32,
    lowpass_coeff: f32,
}

impl DampingFilter {
    fn new() -> Self {
        Self {
            lowpass_state: 0.0,
            lowpass_coeff: 0.5,
        }
    }
    
    fn set_damping(&mut self, damping: f32) {
        // Convert damping parameter to filter coefficient
        // Higher damping = more high frequency attenuation
        self.lowpass_coeff = 0.1 + (1.0 - damping) * 0.88;
    }
    
    fn process(&mut self, input: f32) -> f32 {
        // One-pole lowpass filter
        self.lowpass_state = input * (1.0 - self.lowpass_coeff) 
                           + self.lowpass_state * self.lowpass_coeff;
        self.lowpass_state
    }
    
    fn clear(&mut self) {
        self.lowpass_state = 0.0;
    }
}

// ============================================================================
// FDN DELAY TIMES (Prime numbers for minimal periodicity)
// ============================================================================

const FDN_DELAYS: [usize; 8] = [
    2069, 2293, 2467, 2699, 2909, 3163, 3371, 3581
];

const INPUT_DIFFUSER_L: [usize; 4] = [142, 107, 379, 277];
const INPUT_DIFFUSER_R: [usize; 4] = [151, 113, 389, 283];

const OUTPUT_DIFFUSER_L: [usize; 2] = [663, 441];
const OUTPUT_DIFFUSER_R: [usize; 2] = [673, 449];

// Hadamard matrix for FDN (scaled by 1/sqrt(8))
const HADAMARD_SCALE: f32 = 0.353553390593; // 1/sqrt(8)

const HADAMARD_8X8: [[f32; 8]; 8] = [
    [ 1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0],
    [ 1.0, -1.0,  1.0, -1.0,  1.0, -1.0,  1.0, -1.0],
    [ 1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0],
    [ 1.0,  1.0,  1.0,  1.0, -1.0, -1.0, -1.0, -1.0],
    [ 1.0, -1.0,  1.0, -1.0, -1.0,  1.0, -1.0,  1.0],
    [ 1.0,  1.0, -1.0, -1.0, -1.0, -1.0,  1.0,  1.0],
    [ 1.0, -1.0, -1.0,  1.0, -1.0,  1.0,  1.0, -1.0],
];

// LFO rates for modulation (Hz)
const LFO_RATES: [f32; 8] = [
    0.13, 0.17, 0.23, 0.31, 0.37, 0.43, 0.53, 0.61
];

// ============================================================================
// REVERB IMPLEMENTATION
// ============================================================================

fn scale_delay(base: usize, sample_rate: f32) -> usize {
    ((base as f32 * sample_rate) / 44100.0) as usize
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let max_pre_delay = (sample_rate * 0.5) as usize; // 500ms max
        
        // Initialize FDN delays scaled to sample rate
        let fdn_delays = [
            DelayLine::new(scale_delay(FDN_DELAYS[0], sample_rate)),
            DelayLine::new(scale_delay(FDN_DELAYS[1], sample_rate)),
            DelayLine::new(scale_delay(FDN_DELAYS[2], sample_rate)),
            DelayLine::new(scale_delay(FDN_DELAYS[3], sample_rate)),
            DelayLine::new(scale_delay(FDN_DELAYS[4], sample_rate)),
            DelayLine::new(scale_delay(FDN_DELAYS[5], sample_rate)),
            DelayLine::new(scale_delay(FDN_DELAYS[6], sample_rate)),
            DelayLine::new(scale_delay(FDN_DELAYS[7], sample_rate)),
        ];
        
        // Initialize damping filters
        let fdn_dampers = [
            DampingFilter::new(),
            DampingFilter::new(),
            DampingFilter::new(),
            DampingFilter::new(),
            DampingFilter::new(),
            DampingFilter::new(),
            DampingFilter::new(),
            DampingFilter::new(),
        ];
        
        // Initialize modulated input diffusers
        let input_diffuser_l = [
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_L[0], sample_rate), 8.0, 0.75),
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_L[1], sample_rate), 8.0, 0.75),
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_L[2], sample_rate), 8.0, 0.625),
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_L[3], sample_rate), 8.0, 0.625),
        ];
        
        let input_diffuser_r = [
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_R[0], sample_rate), 8.0, 0.75),
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_R[1], sample_rate), 8.0, 0.75),
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_R[2], sample_rate), 8.0, 0.625),
            ModulatedAllPass::new(scale_delay(INPUT_DIFFUSER_R[3], sample_rate), 8.0, 0.625),
        ];
        
        // Initialize output diffusers
        let output_diffuser_l = [
            AllPassFilter::new(scale_delay(OUTPUT_DIFFUSER_L[0], sample_rate), 0.5),
            AllPassFilter::new(scale_delay(OUTPUT_DIFFUSER_L[1], sample_rate), 0.5),
        ];
        
        let output_diffuser_r = [
            AllPassFilter::new(scale_delay(OUTPUT_DIFFUSER_R[0], sample_rate), 0.5),
            AllPassFilter::new(scale_delay(OUTPUT_DIFFUSER_R[1], sample_rate), 0.5),
        ];
        
        // Initialize LFO phases
        let lfo_phase = [0.0; 8];
        let lfo_rate = LFO_RATES;
        
        let mut reverb = Self {
            enabled: false,
            sample_rate,
            decay_time: 2.0,
            pre_delay: 0.0,
            diffusion: 0.7,
            damping: 0.5,
            wet: 0.3,
            width: 1.0,
            room_size: 0.3, // Maps to decay_time for compatibility
            pre_delay_l: DelayLine::new(max_pre_delay),
            pre_delay_r: DelayLine::new(max_pre_delay),
            early_l: EarlyReflections::new(sample_rate, false),
            early_r: EarlyReflections::new(sample_rate, true),
            input_diffuser_l,
            input_diffuser_r,
            fdn_delays,
            fdn_dampers,
            fdn_outputs: [0.0; 8],
            output_diffuser_l,
            output_diffuser_r,
            lfo_phase,
            lfo_rate,
        };
        
        reverb.update_damping();
        reverb
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled && self.enabled {
            self.clear();
        }
        self.enabled = enabled;
    }
    
    pub fn set_decay_time(&mut self, decay_time: f32) {
        self.decay_time = decay_time.clamp(0.1, 20.0);
    }
    
    pub fn set_pre_delay(&mut self, pre_delay: f32) {
        self.pre_delay = pre_delay.clamp(0.0, 0.5);
    }
    
    pub fn set_diffusion(&mut self, diffusion: f32) {
        self.diffusion = diffusion.clamp(0.0, 1.0);
    }
    
    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.0, 1.0);
        // Map room_size (0-1) to decay_time (0.1-10 seconds)
        self.decay_time = 0.1 + (room_size * 9.9);
    }
    
    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
        self.update_damping();
    }
    
    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet.clamp(0.0, 1.0);
    }
    
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(0.0, 1.0);
    }
    
    // Getter methods for serialization
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn get_room_size(&self) -> f32 {
        self.room_size
    }
    
    pub fn get_damping(&self) -> f32 {
        self.damping
    }
    
    pub fn get_wet(&self) -> f32 {
        self.wet
    }
    
    pub fn get_width(&self) -> f32 {
        self.width
    }
    
    fn update_damping(&mut self) {
        for damper in &mut self.fdn_dampers {
            damper.set_damping(self.damping);
        }
    }
    
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() < 0.1 {
            return;
        }
        
        // Recreate reverb with new sample rate
        *self = Self::new(sample_rate);
    }
    
    
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        // ========================================
        // STAGE 1: Pre-delay
        // ========================================
        let pre_delay_samples = (self.pre_delay * self.sample_rate) as usize;
        
        self.pre_delay_l.write(left);
        self.pre_delay_r.write(right);
        
        let delayed_l = if pre_delay_samples > 0 {
            self.pre_delay_l.read(pre_delay_samples)
        } else {
            left
        };
        
        let delayed_r = if pre_delay_samples > 0 {
            self.pre_delay_r.read(pre_delay_samples)
        } else {
            right
        };
        
        // ========================================
        // STAGE 2: Early reflections
        // ========================================
        let early_l = self.early_l.process(delayed_l);
        let early_r = self.early_r.process(delayed_r);
        
        // ========================================
        // STAGE 3: Input diffusion (modulated)
        // ========================================
        // Update LFO phases
        for i in 0..8 {
            self.lfo_phase[i] += (self.lfo_rate[i] * 2.0 * std::f32::consts::PI) / self.sample_rate;
            if self.lfo_phase[i] > 2.0 * std::f32::consts::PI {
                self.lfo_phase[i] -= 2.0 * std::f32::consts::PI;
            }
        }
        
        let mut diffused_l = delayed_l * self.diffusion;
        let mut diffused_r = delayed_r * self.diffusion;
        
        for i in 0..4 {
            let lfo_val = self.lfo_phase[i].sin();
            diffused_l = self.input_diffuser_l[i].process(diffused_l, lfo_val);
        }
        
        for i in 0..4 {
            let lfo_val = self.lfo_phase[i + 4].sin();
            diffused_r = self.input_diffuser_r[i].process(diffused_r, lfo_val);
        }
        
        // Mix L+R for reverb tank input
        let tank_input = (diffused_l + diffused_r) * 0.5;
        
        // ========================================
        // STAGE 4: Feedback Delay Network (FDN)
        // ========================================
        // Calculate decay coefficient based on decay time
        // RT60 formula: g = 10^(-3 * delay_time / (sample_rate * decay_time))
        let decay_coefficient = {
            let avg_delay = FDN_DELAYS.iter().sum::<usize>() as f32 / FDN_DELAYS.len() as f32;
            let delay_time = avg_delay / 44100.0; // Convert to seconds
            (-3.0 * delay_time / self.decay_time).exp()
        };
        
        // Read from delay lines and apply damping
        for i in 0..8 {
            let delay_samples = scale_delay(FDN_DELAYS[i], self.sample_rate);
            let delayed = self.fdn_delays[i].read(delay_samples);
            self.fdn_outputs[i] = self.fdn_dampers[i].process(delayed);
        }
        
        // Apply Hadamard matrix (orthogonal feedback matrix)
        let mut fdn_feedback = [0.0f32; 8];
        for i in 0..8 {
            let mut sum = 0.0;
            for j in 0..8 {
                sum += HADAMARD_8X8[i][j] * self.fdn_outputs[j];
            }
            fdn_feedback[i] = sum * HADAMARD_SCALE * decay_coefficient;
        }
        
        // Write back to delay lines with new input
        for i in 0..8 {
            self.fdn_delays[i].write(tank_input + fdn_feedback[i]);
        }
        
        // Sum FDN outputs for L/R
        let mut late_l = 0.0;
        let mut late_r = 0.0;
        
        // Distribute FDN outputs to L/R channels
        for i in 0..8 {
            if i % 2 == 0 {
                late_l += self.fdn_outputs[i];
            } else {
                late_r += self.fdn_outputs[i];
            }
        }
        
        late_l *= 0.25;
        late_r *= 0.25;
        
        // ========================================
        // STAGE 5: Output diffusion
        // ========================================
        for diffuser in &mut self.output_diffuser_l {
            late_l = diffuser.process(late_l);
        }
        
        for diffuser in &mut self.output_diffuser_r {
            late_r = diffuser.process(late_r);
        }
        
        // ========================================
        // STAGE 6: Mix early + late reverb
        // ========================================
        let reverb_l = early_l + late_l;
        let reverb_r = early_r + late_r;
        
        // ========================================
        // STAGE 7: Wet/dry mix and stereo width
        // ========================================
        let wet1 = self.wet * (self.width * 0.5 + 0.5);
        let wet2 = self.wet * ((1.0 - self.width) * 0.5);
        let dry = 1.0 - self.wet;
        
        let final_l = (reverb_l * wet1) + (reverb_r * wet2) + (left * dry);
        let final_r = (reverb_r * wet1) + (reverb_l * wet2) + (right * dry);
        
        // Soft clip to prevent harsh clipping
        let soft_clip = |x: f32| {
            if x > 1.0 {
                1.0
            } else if x < -1.0 {
                -1.0
            } else {
                x
            }
        };
        
        (soft_clip(final_l), soft_clip(final_r))
    }
    
    fn clear(&mut self) {
        self.pre_delay_l.clear();
        self.pre_delay_r.clear();
        self.early_l.clear();
        self.early_r.clear();
        
        for diffuser in &mut self.input_diffuser_l {
            diffuser.clear();
        }
        for diffuser in &mut self.input_diffuser_r {
            diffuser.clear();
        }
        
        for delay in &mut self.fdn_delays {
            delay.clear();
        }
        for damper in &mut self.fdn_dampers {
            damper.clear();
        }
        
        for diffuser in &mut self.output_diffuser_l {
            diffuser.clear();
        }
        for diffuser in &mut self.output_diffuser_r {
            diffuser.clear();
        }
        
        self.fdn_outputs = [0.0; 8];
        self.lfo_phase = [0.0; 8];
    }
    
    /// Reset reverb buffers (public API)
    pub fn reset(&mut self) {
        self.clear();
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

// Scale tuning values based on sample rate (44100 is reference)
fn scale_tuning(tuning: usize, sample_rate: f32) -> usize {
    ((tuning as f32 * sample_rate) / 44100.0) as usize
}

// ============================================================================
// TESTS
// ============================================================================

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
        reverb.set_wet(0.5);
        reverb.set_room_size(0.5);
        
        // Process some samples
        let (out_l, out_r) = reverb.process(1.0, 0.0);
        
        // Output should be different from input when wet > 0
        assert!(out_l != 1.0 || out_r != 0.0);
        
        // Output should be within reasonable bounds
        assert!(out_l.abs() <= 1.5);
        assert!(out_r.abs() <= 1.5);
    }
    
    #[test]
    fn test_delay_line() {
        let mut delay = DelayLine::new(100);
        
        delay.write(1.0);
        delay.write(0.5);
        delay.write(0.0);
        
        // Read 2 samples back
        let output = delay.read(2);
        assert_eq!(output, 1.0);
    }
    
    #[test]
    fn test_early_reflections() {
        let mut early = EarlyReflections::new(44100.0, false);
        
        // Process impulse
        let output = early.process(1.0);
        
        // Should produce some output
        assert!(output != 0.0);
        assert!(output.abs() <= 1.0);
    }
    
    #[test]
    fn test_parameter_clamping() {
        let mut reverb = Reverb::new(44100.0);
        
        // Test room_size clamping
        reverb.set_room_size(2.0);
        assert_eq!(reverb.get_room_size(), 1.0);
        
        reverb.set_room_size(-0.5);
        assert_eq!(reverb.get_room_size(), 0.0);
        
        // Test wet clamping
        reverb.set_wet(1.5);
        assert_eq!(reverb.get_wet(), 1.0);
        
        reverb.set_wet(-0.5);
        assert_eq!(reverb.get_wet(), 0.0);
    }
    
    #[test]
    fn test_stereo_width() {
        let mut reverb = Reverb::new(44100.0);
        reverb.set_enabled(true);
        reverb.set_wet(1.0);
        
        // Full stereo width
        reverb.set_width(1.0);
        let (l1, r1) = reverb.process(1.0, 0.0);
        
        // Reset and test mono (no width)
        reverb.reset();
        reverb.set_width(0.0);
        let (l2, r2) = reverb.process(1.0, 0.0);
        
        // With width=0, L and R should be more similar
        let diff1 = (l1 - r1).abs();
        let diff2 = (l2 - r2).abs();
        assert!(diff2 < diff1 || diff2.abs() < 0.01);
    }
}
