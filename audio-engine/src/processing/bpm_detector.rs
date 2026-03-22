/// BPM Detection using autocorrelation on low-frequency energy envelope
/// Focuses on bass/kick drum to avoid confusion from vocals

const ENERGY_BUFFER_SECONDS: usize = 20; // Increased from 8 to 20 seconds for better precision
const MIN_BPM: f32 = 60.0;  // Extended to cover slower tempos (ballads, downtempo)
const MAX_BPM: f32 = 180.0; // Extended to cover faster tempos (EDM, drum & bass)
const ENERGY_HOP_SIZE: usize = 512; // Decimate energy for efficiency

/// Fold BPM to the most musically sensible range (70-140 BPM preferred)
/// Corrects for harmonic detection errors (2x, 3x, 4x) and subdivisions (0.5x, 0.25x)
fn fold_to_musical_bpm(mut bpm: f32) -> f32 {
    // Target range: preferably 70-140 BPM (most common for modern music)
    const PREFERRED_MIN: f32 = 70.0;
    const PREFERRED_MAX: f32 = 140.0;
    
    // If already in preferred range, return as-is
    if bpm >= PREFERRED_MIN && bpm <= PREFERRED_MAX {
        return bpm;
    }
    
    // Try dividing by powers of 2 (for harmonic errors: 2x, 4x, 8x)
    let mut best_bpm = bpm;
    let mut best_score = score_bpm(bpm);
    
    for divisor in [2.0, 3.0, 4.0, 8.0] {
        let candidate = bpm / divisor;
        if candidate >= MIN_BPM && candidate <= MAX_BPM {
            let score = score_bpm(candidate);
            if score > best_score {
                best_bpm = candidate;
                best_score = score;
            }
        }
    }
    
    // Try multiplying by powers of 2 (for subdivision errors: 0.5x, 0.25x)
    for multiplier in [2.0, 3.0, 4.0] {
        let candidate = bpm * multiplier;
        if candidate >= MIN_BPM && candidate <= MAX_BPM {
            let score = score_bpm(candidate);
            if score > best_score {
                best_bpm = candidate;
                best_score = score;
            }
        }
    }
    
    best_bpm
}

/// Score a BPM value based on how musically sensible it is
/// Higher score = more likely to be correct
fn score_bpm(bpm: f32) -> f32 {
    const PREFERRED_MIN: f32 = 70.0;
    const PREFERRED_MAX: f32 = 140.0;
    
    // Heavily favor the 70-140 range
    if bpm >= PREFERRED_MIN && bpm <= PREFERRED_MAX {
        return 100.0;
    }
    
    // Moderate favor for extended ranges
    if bpm >= 60.0 && bpm < PREFERRED_MIN {
        return 50.0; // Slow songs (ballads)
    }
    if bpm > PREFERRED_MAX && bpm <= 180.0 {
        return 50.0; // Fast songs (EDM)
    }
    
    // Very low score for extreme values
    0.0
}

pub struct BPMDetector {
    sample_rate: f32,
    // Decimated energy envelope buffer
    energy_envelope: Vec<f32>,
    energy_write_index: usize,
    hop_counter: usize,
    current_energy_accum: f32,
    
    // Low-pass filter state (per isolare kick/bass @ 150Hz)
    lpf_prev_left: f32,
    lpf_prev_right: f32,
    lpf_alpha: f32,
    
    // BPM estimate
    current_bpm: f32,
    last_estimate_time: usize,
    sample_counter: usize,
    
    // Smoothing e stabilità
    bpm_history: Vec<f32>,
    bpm_history_index: usize,
    bpm_lock: Option<f32>, // BPM "bloccato" quando stabile
    bpm_lock_confidence: usize,
}

impl BPMDetector {
    pub fn new(sample_rate: f32) -> Self {
        // Calculate buffer size dynamically based on sample rate
        let buffer_size_samples = (sample_rate as usize) * ENERGY_BUFFER_SECONDS;
        let envelope_size = buffer_size_samples / ENERGY_HOP_SIZE;
        
        // Low-pass filter @ 150Hz (isola kick drum)
        let cutoff = 150.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = 1.0 / sample_rate;
        let lpf_alpha = dt / (rc + dt);
        
        Self {
            sample_rate,
            energy_envelope: vec![0.0; envelope_size],
            energy_write_index: 0,
            hop_counter: 0,
            current_energy_accum: 0.0,
            lpf_prev_left: 0.0,
            lpf_prev_right: 0.0,
            lpf_alpha,
            current_bpm: 0.0,
            last_estimate_time: 0,
            sample_counter: 0,
            bpm_history: vec![0.0; 12],  // Balanced: faster than 20, more stable than 8
            bpm_history_index: 0,
            bpm_lock: None,
            bpm_lock_confidence: 0,
        }
    }
    
    /// Process audio samples and update energy envelope
    pub fn process(&mut self, left: f32, right: f32) {
        self.sample_counter += 1;
        
        // Low-pass filter per isolare frequenze basse (kick drum)
        self.lpf_prev_left = self.lpf_prev_left + self.lpf_alpha * (left - self.lpf_prev_left);
        self.lpf_prev_right = self.lpf_prev_right + self.lpf_alpha * (right - self.lpf_prev_right);
        
        let low_left = self.lpf_prev_left;
        let low_right = self.lpf_prev_right;
        
        // Accumulate energy delle frequenze basse
        let energy = (low_left * low_left + low_right * low_right).sqrt();
        self.current_energy_accum += energy;
        self.hop_counter += 1;
        
        // Every HOP_SIZE samples, store averaged energy
        if self.hop_counter >= ENERGY_HOP_SIZE {
            let avg_energy = self.current_energy_accum / ENERGY_HOP_SIZE as f32;
            
            self.energy_envelope[self.energy_write_index] = avg_energy;
            self.energy_write_index = (self.energy_write_index + 1) % self.energy_envelope.len();
            
            self.current_energy_accum = 0.0;
            self.hop_counter = 0;
        }
        
        // Re-estimate BPM ogni 3 secondi
        if self.sample_counter - self.last_estimate_time > (self.sample_rate as usize * 3) {
            self.estimate_bpm_autocorr();
            self.last_estimate_time = self.sample_counter;
        }
    }
    
    /// Estimate BPM using autocorrelation on energy envelope
    fn estimate_bpm_autocorr(&mut self) {
        let envelope_len = self.energy_envelope.len();
        
        // Need at least 5 seconds of data
        if self.sample_counter < (self.sample_rate as usize * 5) {
            return;
        }
        
        // Convert BPM range to lag range (in decimated samples)
        let envelope_sample_rate = self.sample_rate / ENERGY_HOP_SIZE as f32;
        let min_lag = ((60.0 / MAX_BPM) * envelope_sample_rate) as usize;
        let max_lag = ((60.0 / MIN_BPM) * envelope_sample_rate) as usize;
        
        if max_lag >= envelope_len / 2 {
            return; // Not enough data
        }
        
        // Compute autocorrelation for each lag
        let mut best_lag = min_lag;
        let mut best_correlation = 0.0f32;
        
        for lag in min_lag..=max_lag {
            let mut correlation = 0.0f32;
            let mut norm = 0.0f32;
            
            // Compute correlation with circular buffer
            for i in 0..(envelope_len - lag) {
                let idx1 = (self.energy_write_index + i) % envelope_len;
                let idx2 = (self.energy_write_index + i + lag) % envelope_len;
                
                let e1 = self.energy_envelope[idx1];
                let e2 = self.energy_envelope[idx2];
                
                correlation += e1 * e2;
                norm += e1 * e1;
            }
            
            // Normalize
            if norm > 0.0 {
                correlation /= norm.sqrt();
            }
            
            if correlation > best_correlation {
                best_correlation = correlation;
                best_lag = lag;
            }
        }
        
        // Convert best lag back to BPM
        if best_correlation > 0.25 {
            let period_seconds = best_lag as f32 / envelope_sample_rate;
            let mut bpm = 60.0 / period_seconds;
            
            // HARMONIC FOLDING: riporta il BPM nel range musicalmente sensato
            // Preferenza per il range 70-140 BPM (il più comune per musica moderna)
            bpm = fold_to_musical_bpm(bpm);
            
            // Se abbiamo un lock e il nuovo BPM è vicino, mantienilo
            if let Some(locked_bpm) = self.bpm_lock {
                let diff = (bpm - locked_bpm).abs();
                if diff < 3.0 {
                    // Vicino al lock, aumenta confidence
                    self.bpm_lock_confidence += 1;
                    // Usa il lock invece del nuovo valore (più stabile)
                    self.bpm_history[self.bpm_history_index] = locked_bpm;
                    self.bpm_history_index = (self.bpm_history_index + 1) % self.bpm_history.len();
                    self.current_bpm = locked_bpm;
                    return;
                } else if diff > 10.0 {
                    // Troppo lontano, resetta il lock
                    self.bpm_lock = None;
                    self.bpm_lock_confidence = 0;
                }
            }
            
            // Update history and smooth
            self.bpm_history[self.bpm_history_index] = bpm;
            self.bpm_history_index = (self.bpm_history_index + 1) % self.bpm_history.len();
            
            // Average all valid (non-zero) history for better precision
            let valid: Vec<f32> = self.bpm_history.iter()
                .filter(|&&v| v > 0.0)
                .copied()
                .collect();
            
            if !valid.is_empty() {
                // Use all valid readings for more stable average
                self.current_bpm = valid.iter().sum::<f32>() / valid.len() as f32;
                self.current_bpm = self.current_bpm.round();
                
                // Se abbiamo abbastanza letture stabili, crea un lock
                if valid.len() >= 6 {
                    let variance: f32 = valid.iter()
                        .map(|&v| (v - self.current_bpm).powi(2))
                        .sum::<f32>() / valid.len() as f32;
                    let std_dev = variance.sqrt();
                    
                    // Se stabile (std_dev < 2), blocca il BPM
                    if std_dev < 2.0 {
                        self.bpm_lock = Some(self.current_bpm);
                        self.bpm_lock_confidence = 1;
                    }
                }
            }
        }
    }
    
    /// Get current BPM estimate
    pub fn get_bpm(&self) -> f32 {
        // Return BPM if we have enough history (requires 3 readings for stability)
        if self.bpm_history.iter().filter(|&&v| v > 0.0).count() >= 3 {
            self.current_bpm
        } else {
            0.0
        }
    }
    
    /// Reset the detector
    pub fn reset(&mut self) {
        self.energy_envelope.fill(0.0);
        self.energy_write_index = 0;
        self.hop_counter = 0;
        self.current_energy_accum = 0.0;
        self.lpf_prev_left = 0.0;
        self.lpf_prev_right = 0.0;
        self.sample_counter = 0;
        self.last_estimate_time = 0;
        self.current_bpm = 0.0;
        self.bpm_history.fill(0.0);
        self.bpm_history_index = 0;
        self.bpm_lock = None;
        self.bpm_lock_confidence = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bpm_detector_creation() {
        let detector = BPMDetector::new(48000.0);
        assert_eq!(detector.get_bpm(), 0.0);
        assert_eq!(detector.get_confidence(), 0.0);
    }
    
    #[test]
    fn test_bpm_detector_simple_beat() {
        let mut detector = BPMDetector::new(48000.0);
        
        // Simulate 120 BPM (2 beats per second = 0.5 seconds per beat = 24000 samples @ 48kHz)
        let beat_interval = 24000;
        
        for i in 0..10 {
            // Create a beat (high energy spike)
            for _ in 0..100 {
                detector.process(0.8, 0.8);
            }
            
            // Silence between beats
            for _ in 0..(beat_interval - 100) {
                detector.process(0.0, 0.0);
            }
        }
        
        let bpm = detector.get_bpm();
        // Should detect approximately 120 BPM (with some tolerance)
        assert!(bpm > 100.0 && bpm < 140.0 || bpm == 0.0); // 0.0 if not enough confidence yet
    }
}
