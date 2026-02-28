/// Parametric equalizer with biquad filters
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    LowShelf,
    Peaking,
    HighShelf,
    LowPass,
    HighPass,
}

/// Biquad filter coefficients
#[derive(Debug, Clone, Copy)]
struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

/// Biquad filter state (per channel)
#[derive(Debug, Clone, Copy)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, input: f32, coeffs: &BiquadCoeffs) -> f32 {
        let output = coeffs.b0 * input + coeffs.b1 * self.x1 + coeffs.b2 * self.x2
            - coeffs.a1 * self.y1 - coeffs.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Single EQ band with biquad filter
#[derive(Debug, Clone)]
pub struct EQBand {
    filter_type: FilterType,
    // Target parameters (set by user)
    target_frequency: f32,
    target_gain_db: f32,
    target_q: f32,
    // Current parameters (smoothed)
    current_frequency: f32,
    current_gain_db: f32,
    current_q: f32,
    sample_rate: f32,
    coeffs: BiquadCoeffs,
    state_l: BiquadState,
    state_r: BiquadState,
    enabled: bool,
    // Smoothing coefficient (for parameter interpolation)
    smooth_coeff: f32,
    // Counter to update coefficients every N samples (optimization)
    update_counter: usize,
}

impl EQBand {
    pub fn new(filter_type: FilterType, frequency: f32, sample_rate: f32) -> Self {
        // Calculate smoothing coefficient for ~5ms smoothing time
        // smooth_coeff = exp(-1.0 / (smoothing_time * sample_rate))
        let smoothing_time = 0.005; // 5ms
        let smooth_coeff = (-1.0 / (smoothing_time * sample_rate)).exp();
        
        let mut band = Self {
            filter_type,
            target_frequency: frequency,
            target_gain_db: 0.0,
            target_q: 0.707, // Butterworth Q
            current_frequency: frequency,
            current_gain_db: 0.0,
            current_q: 0.707,
            sample_rate,
            coeffs: BiquadCoeffs {
                b0: 1.0,
                b1: 0.0,
                b2: 0.0,
                a1: 0.0,
                a2: 0.0,
            },
            state_l: BiquadState::new(),
            state_r: BiquadState::new(),
            enabled: true,
            smooth_coeff,
            update_counter: 0,
        };
        band.update_coefficients();
        band
    }

    pub fn set_gain(&mut self, gain_db: f32) {
        self.target_gain_db = gain_db.clamp(-24.0, 24.0);
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.target_frequency = frequency.clamp(20.0, 20000.0);
    }

    pub fn set_q(&mut self, q: f32) {
        self.target_q = q.clamp(0.1, 10.0);
    }

    pub fn set_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
        // Force recalculation of coefficients with current parameters
        self.update_coefficients();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Recalculate smoothing coefficient
        let smoothing_time = 0.005; // 5ms
        self.smooth_coeff = (-1.0 / (smoothing_time * sample_rate)).exp();
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        let w0 = 2.0 * PI * self.current_frequency / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * self.current_q);
        let a = 10_f32.powf(self.current_gain_db / 40.0); // sqrt of gain

        match self.filter_type {
            FilterType::LowShelf => {
                let a_plus_1 = a + 1.0;
                let a_minus_1 = a - 1.0;
                let sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

                let b0 = a * (a_plus_1 - a_minus_1 * cos_w0 + sqrt_a_alpha);
                let b1 = 2.0 * a * (a_minus_1 - a_plus_1 * cos_w0);
                let b2 = a * (a_plus_1 - a_minus_1 * cos_w0 - sqrt_a_alpha);
                let a0 = a_plus_1 + a_minus_1 * cos_w0 + sqrt_a_alpha;
                let a1 = -2.0 * (a_minus_1 + a_plus_1 * cos_w0);
                let a2 = a_plus_1 + a_minus_1 * cos_w0 - sqrt_a_alpha;

                self.coeffs.b0 = b0 / a0;
                self.coeffs.b1 = b1 / a0;
                self.coeffs.b2 = b2 / a0;
                self.coeffs.a1 = a1 / a0;
                self.coeffs.a2 = a2 / a0;
            }
            FilterType::Peaking => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha / a;

                self.coeffs.b0 = b0 / a0;
                self.coeffs.b1 = b1 / a0;
                self.coeffs.b2 = b2 / a0;
                self.coeffs.a1 = a1 / a0;
                self.coeffs.a2 = a2 / a0;
            }
            FilterType::HighShelf => {
                let a_plus_1 = a + 1.0;
                let a_minus_1 = a - 1.0;
                let sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

                let b0 = a * (a_plus_1 + a_minus_1 * cos_w0 + sqrt_a_alpha);
                let b1 = -2.0 * a * (a_minus_1 + a_plus_1 * cos_w0);
                let b2 = a * (a_plus_1 + a_minus_1 * cos_w0 - sqrt_a_alpha);
                let a0 = a_plus_1 - a_minus_1 * cos_w0 + sqrt_a_alpha;
                let a1 = 2.0 * (a_minus_1 - a_plus_1 * cos_w0);
                let a2 = a_plus_1 - a_minus_1 * cos_w0 - sqrt_a_alpha;

                self.coeffs.b0 = b0 / a0;
                self.coeffs.b1 = b1 / a0;
                self.coeffs.b2 = b2 / a0;
                self.coeffs.a1 = a1 / a0;
                self.coeffs.a2 = a2 / a0;
            }
            FilterType::LowPass => {
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;

                self.coeffs.b0 = b0 / a0;
                self.coeffs.b1 = b1 / a0;
                self.coeffs.b2 = b2 / a0;
                self.coeffs.a1 = a1 / a0;
                self.coeffs.a2 = a2 / a0;
            }
            FilterType::HighPass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;

                self.coeffs.b0 = b0 / a0;
                self.coeffs.b1 = b1 / a0;
                self.coeffs.b2 = b2 / a0;
                self.coeffs.a1 = a1 / a0;
                self.coeffs.a2 = a2 / a0;
            }
        }
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        // Smooth parameters towards targets using exponential smoothing
        // current = current * smooth_coeff + target * (1 - smooth_coeff)
        let one_minus_smooth = 1.0 - self.smooth_coeff;
        self.current_frequency = self.current_frequency * self.smooth_coeff + self.target_frequency * one_minus_smooth;
        self.current_gain_db = self.current_gain_db * self.smooth_coeff + self.target_gain_db * one_minus_smooth;
        self.current_q = self.current_q * self.smooth_coeff + self.target_q * one_minus_smooth;

        // Update coefficients every 32 samples to avoid expensive recalculation at every sample
        // This is a good balance between smoothness and performance
        self.update_counter += 1;
        if self.update_counter >= 32 {
            self.update_counter = 0;
            self.update_coefficients();
        }

        let left_out = self.state_l.process(left, &self.coeffs);
        let right_out = self.state_r.process(right, &self.coeffs);

        (left_out, right_out)
    }

    pub fn reset(&mut self) {
        self.state_l.reset();
        self.state_r.reset();
    }
}

/// 4-band parametric equalizer
#[derive(Debug, Clone)]
pub struct Equalizer {
    pub low_shelf: EQBand,
    pub low_mid: EQBand,
    pub high_mid: EQBand,
    pub high_shelf: EQBand,
    pub enabled: bool,
}

impl Equalizer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            low_shelf: EQBand::new(FilterType::LowShelf, 80.0, sample_rate),
            low_mid: EQBand::new(FilterType::Peaking, 400.0, sample_rate),
            high_mid: EQBand::new(FilterType::Peaking, 2500.0, sample_rate),
            high_shelf: EQBand::new(FilterType::HighShelf, 8000.0, sample_rate),
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset();
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.low_shelf.set_sample_rate(sample_rate);
        self.low_mid.set_sample_rate(sample_rate);
        self.high_mid.set_sample_rate(sample_rate);
        self.high_shelf.set_sample_rate(sample_rate);
    }

    pub fn set_low_shelf(&mut self, gain_db: f32) {
        self.low_shelf.set_gain(gain_db);
    }

    pub fn set_low_mid(&mut self, gain_db: f32) {
        self.low_mid.set_gain(gain_db);
    }

    pub fn set_high_mid(&mut self, gain_db: f32) {
        self.high_mid.set_gain(gain_db);
    }

    pub fn set_high_shelf(&mut self, gain_db: f32) {
        self.high_shelf.set_gain(gain_db);
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        let (left, right) = self.low_shelf.process(left, right);
        let (left, right) = self.low_mid.process(left, right);
        let (left, right) = self.high_mid.process(left, right);
        let (left, right) = self.high_shelf.process(left, right);

        (left, right)
    }

    pub fn reset(&mut self) {
        self.low_shelf.reset();
        self.low_mid.reset();
        self.high_mid.reset();
        self.high_shelf.reset();
    }
}

/// Dynamic parametric equalizer with unlimited bands
#[derive(Debug, Clone)]
pub struct ParametricEqualizer {
    bands: Vec<EQBand>,
    sample_rate: f32,
    pub enabled: bool,
}

impl ParametricEqualizer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bands: Vec::new(),
            sample_rate,
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset();
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for band in &mut self.bands {
            band.set_sample_rate(sample_rate);
        }
    }

    /// Clear all bands
    pub fn clear(&mut self) {
        self.bands.clear();
    }

    /// Add a new band with specified parameters
    pub fn add_band(&mut self, filter_type: FilterType, frequency: f32, gain_db: f32, q: f32) {
        let mut band = EQBand::new(filter_type, frequency, self.sample_rate);
        band.set_gain(gain_db);
        band.set_q(q);
        self.bands.push(band);
    }

    /// Update a specific band
    pub fn update_band(&mut self, index: usize, filter_type: FilterType, frequency: f32, gain_db: f32, q: f32) {
        if let Some(band) = self.bands.get_mut(index) {
            band.set_type(filter_type);
            band.set_frequency(frequency);
            band.set_gain(gain_db);
            band.set_q(q);
        }
    }

    /// Remove a band by index
    pub fn remove_band(&mut self, index: usize) {
        if index < self.bands.len() {
            self.bands.remove(index);
        }
    }

    /// Get the number of bands
    pub fn band_count(&self) -> usize {
        self.bands.len()
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled || self.bands.is_empty() {
            return (left, right);
        }

        let mut left_out = left;
        let mut right_out = right;

        for band in &mut self.bands {
            (left_out, right_out) = band.process(left_out, right_out);
        }

        (left_out, right_out)
    }

    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_creation() {
        let eq = Equalizer::new(44100.0);
        assert!(eq.enabled);
    }

    #[test]
    fn test_eq_passthrough() {
        let mut eq = Equalizer::new(44100.0);
        eq.set_enabled(false);
        let (left, right) = eq.process(0.5, -0.5);
        assert_eq!(left, 0.5);
        assert_eq!(right, -0.5);
    }

    #[test]
    fn test_parametric_eq() {
        let mut peq = ParametricEqualizer::new(44100.0);
        assert_eq!(peq.band_count(), 0);
        
        peq.add_band(FilterType::Peaking, 1000.0, 6.0, 1.0);
        assert_eq!(peq.band_count(), 1);
        
        let (left, right) = peq.process(1.0, 1.0);
        assert!(left != 0.0 && right != 0.0);
    }
}
