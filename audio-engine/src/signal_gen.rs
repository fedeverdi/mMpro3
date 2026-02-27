/// Signal generators for audio testing and synthesis
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveformType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    WhiteNoise,
    PinkNoise,
}

pub struct SignalGenerator {
    waveform: WaveformType,
    frequency: f32,
    phase: f32,
    sample_rate: f32,
    // Pink noise filter state
    pink_b0: f32,
    pink_b1: f32,
    pink_b2: f32,
    pink_b3: f32,
    pink_b4: f32,
    pink_b5: f32,
    pink_b6: f32,
}

impl SignalGenerator {
    pub fn new(waveform: WaveformType, frequency: f32, sample_rate: f32) -> Self {
        Self {
            waveform,
            frequency,
            phase: 0.0,
            sample_rate,
            pink_b0: 0.0,
            pink_b1: 0.0,
            pink_b2: 0.0,
            pink_b3: 0.0,
            pink_b4: 0.0,
            pink_b5: 0.0,
            pink_b6: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn set_waveform(&mut self, waveform: WaveformType) {
        self.waveform = waveform;
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Generate next sample
    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
            WaveformType::Sine => self.generate_sine(),
            WaveformType::Square => self.generate_square(),
            WaveformType::Sawtooth => self.generate_sawtooth(),
            WaveformType::Triangle => self.generate_triangle(),
            WaveformType::WhiteNoise => self.generate_white_noise(),
            WaveformType::PinkNoise => self.generate_pink_noise(),
        };

        // Update phase for oscillators
        if self.waveform != WaveformType::WhiteNoise && self.waveform != WaveformType::PinkNoise {
            self.phase += self.frequency / self.sample_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        sample
    }

    fn generate_sine(&self) -> f32 {
        (self.phase * 2.0 * PI).sin()
    }

    fn generate_square(&self) -> f32 {
        if self.phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    fn generate_sawtooth(&self) -> f32 {
        2.0 * self.phase - 1.0
    }

    fn generate_triangle(&self) -> f32 {
        if self.phase < 0.5 {
            4.0 * self.phase - 1.0
        } else {
            3.0 - 4.0 * self.phase
        }
    }

    fn generate_white_noise(&self) -> f32 {
        // Simple white noise using rand-like approach
        // In production, use a proper RNG
        let x = (self.phase * 12.9898).sin() * 43758.5453;
        2.0 * (x - x.floor()) - 1.0
    }

    fn generate_pink_noise(&mut self) -> f32 {
        // Paul Kellet's refined pink noise algorithm
        let white = self.generate_white_noise();
        
        self.pink_b0 = 0.99886 * self.pink_b0 + white * 0.0555179;
        self.pink_b1 = 0.99332 * self.pink_b1 + white * 0.0750759;
        self.pink_b2 = 0.96900 * self.pink_b2 + white * 0.1538520;
        self.pink_b3 = 0.86650 * self.pink_b3 + white * 0.3104856;
        self.pink_b4 = 0.55000 * self.pink_b4 + white * 0.5329522;
        self.pink_b5 = -0.7616 * self.pink_b5 - white * 0.0168980;
        
        let pink = self.pink_b0 + self.pink_b1 + self.pink_b2 + self.pink_b3 
                   + self.pink_b4 + self.pink_b5 + self.pink_b6 + white * 0.5362;
        
        self.pink_b6 = white * 0.115926;
        
        pink * 0.11 // Scale to -1..1 range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_generator() {
        let mut generator = SignalGenerator::new(WaveformType::Sine, 440.0, 48000.0);
        let sample = generator.next_sample();
        assert!(sample >= -1.0 && sample <= 1.0);
    }

    #[test]
    fn test_waveform_change() {
        let mut generator = SignalGenerator::new(WaveformType::Sine, 440.0, 48000.0);
        generator.set_waveform(WaveformType::Square);
        let sample = generator.next_sample();
        assert!(sample == 1.0 || sample == -1.0);
    }
}
