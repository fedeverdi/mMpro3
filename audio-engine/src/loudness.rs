// EBU R128 / ITU-R BS.1770 Loudness Metering
// Implements broadcast-quality loudness measurement

use std::collections::VecDeque;

const BLOCK_SIZE: usize = 512; // Processing block size
const MOMENTARY_WINDOW: f64 = 0.4; // 400ms
const SHORT_TERM_WINDOW: f64 = 3.0; // 3 seconds

/// Loudness measurements result
#[derive(Debug, Clone, Copy)]
pub struct LoudnessMeasurements {
    pub momentary: f32,
    pub short_term: f32,
    pub integrated: f32,
    pub loudness_range: f32,
    pub true_peak_dbtp: f32,
}

/// K-weighted filters for perceptual loudness measurement
struct KWeightFilter {
    // Stage 1: High-shelf filter @1680Hz (pre-filter)
    shelf_b0: f64, shelf_b1: f64, shelf_b2: f64,
    shelf_a1: f64, shelf_a2: f64,
    shelf_x1_l: f64, shelf_x2_l: f64,
    shelf_y1_l: f64, shelf_y2_l: f64,
    shelf_x1_r: f64, shelf_x2_r: f64,
    shelf_y1_r: f64, shelf_y2_r: f64,
    
    // Stage 2: High-pass filter @38Hz (RLB filter)
    hp_b0: f64, hp_b1: f64, hp_b2: f64,
    hp_a1: f64, hp_a2: f64,
    hp_x1_l: f64, hp_x2_l: f64,
    hp_y1_l: f64, hp_y2_l: f64,
    hp_x1_r: f64, hp_x2_r: f64,
    hp_y1_r: f64, hp_y2_r: f64,
}

impl KWeightFilter {
    fn new(sample_rate: f64) -> Self {
        // Pre-filter (high-shelf at 1680 Hz, +4dB)
        let f0 = 1681.974450955533;
        let g = 3.999843853973347;
        let q = 0.7071752369554196;
        
        let k = (std::f64::consts::PI * f0 / sample_rate).tan();
        let vh = 10_f64.powf(g / 20.0);
        let vb = vh.powf(0.4996667741545416);
        
        let a0 = 1.0 + k / q + k * k;
        let shelf_b0 = (vh + vb * k / q + k * k) / a0;
        let shelf_b1 = 2.0 * (k * k - vh) / a0;
        let shelf_b2 = (vh - vb * k / q + k * k) / a0;
        let shelf_a1 = 2.0 * (k * k - 1.0) / a0;
        let shelf_a2 = (1.0 - k / q + k * k) / a0;
        
        // RLB filter (high-pass at 38 Hz)
        let f0_hp = 38.13547087602444;
        let q_hp = 0.5003270373238773;
        let k_hp = (std::f64::consts::PI * f0_hp / sample_rate).tan();
        
        let a0_hp = 1.0 + k_hp / q_hp + k_hp * k_hp;
        let hp_b0 = 1.0 / a0_hp;
        let hp_b1 = -2.0 / a0_hp;
        let hp_b2 = 1.0 / a0_hp;
        let hp_a1 = 2.0 * (k_hp * k_hp - 1.0) / a0_hp;
        let hp_a2 = (1.0 - k_hp / q_hp + k_hp * k_hp) / a0_hp;
        
        Self {
            shelf_b0, shelf_b1, shelf_b2, shelf_a1, shelf_a2,
            shelf_x1_l: 0.0, shelf_x2_l: 0.0,
            shelf_y1_l: 0.0, shelf_y2_l: 0.0,
            shelf_x1_r: 0.0, shelf_x2_r: 0.0,
            shelf_y1_r: 0.0, shelf_y2_r: 0.0,
            
            hp_b0, hp_b1, hp_b2, hp_a1, hp_a2,
            hp_x1_l: 0.0, hp_x2_l: 0.0,
            hp_y1_l: 0.0, hp_y2_l: 0.0,
            hp_x1_r: 0.0, hp_x2_r: 0.0,
            hp_y1_r: 0.0, hp_y2_r: 0.0,
        }
    }
    
    fn process(&mut self, left: f64, right: f64) -> (f64, f64) {
        // Process left channel
        // Stage 1: High-shelf
        let shelf_out_l = self.shelf_b0 * left + self.shelf_b1 * self.shelf_x1_l + self.shelf_b2 * self.shelf_x2_l
                        - self.shelf_a1 * self.shelf_y1_l - self.shelf_a2 * self.shelf_y2_l;
        self.shelf_x2_l = self.shelf_x1_l;
        self.shelf_x1_l = left;
        self.shelf_y2_l = self.shelf_y1_l;
        self.shelf_y1_l = shelf_out_l;
        
        // Stage 2: High-pass
        let hp_out_l = self.hp_b0 * shelf_out_l + self.hp_b1 * self.hp_x1_l + self.hp_b2 * self.hp_x2_l
                     - self.hp_a1 * self.hp_y1_l - self.hp_a2 * self.hp_y2_l;
        self.hp_x2_l = self.hp_x1_l;
        self.hp_x1_l = shelf_out_l;
        self.hp_y2_l = self.hp_y1_l;
        self.hp_y1_l = hp_out_l;
        
        // Process right channel
        // Stage 1: High-shelf
        let shelf_out_r = self.shelf_b0 * right + self.shelf_b1 * self.shelf_x1_r + self.shelf_b2 * self.shelf_x2_r
                        - self.shelf_a1 * self.shelf_y1_r - self.shelf_a2 * self.shelf_y2_r;
        self.shelf_x2_r = self.shelf_x1_r;
        self.shelf_x1_r = right;
        self.shelf_y2_r = self.shelf_y1_r;
        self.shelf_y1_r = shelf_out_r;
        
        // Stage 2: High-pass
        let hp_out_r = self.hp_b0 * shelf_out_r + self.hp_b1 * self.hp_x1_r + self.hp_b2 * self.hp_x2_r
                     - self.hp_a1 * self.hp_y1_r - self.hp_a2 * self.hp_y2_r;
        self.hp_x2_r = self.hp_x1_r;
        self.hp_x1_r = shelf_out_r;
        self.hp_y2_r = self.hp_y1_r;
        self.hp_y1_r = hp_out_r;
        
        (hp_out_l, hp_out_r)
    }
}

/// Block power measurement with gating
struct BlockPower {
    power: f64,
    timestamp: f64,
}

/// EBU R128 Loudness Meter
pub struct LoudnessMeter {
    sample_rate: f64,
    k_filter: KWeightFilter,
    
    // Block accumulation
    block_left_sum: f64,
    block_right_sum: f64,
    block_sample_count: usize,
    
    // Block history for windows
    block_history: VecDeque<BlockPower>,
    
    // Integrated loudness (gated)
    integrated_blocks: Vec<f64>,
    
    // Momentary/Short-term windows
    momentary_blocks: usize,
    short_term_blocks: usize,
    
    // Statistics
    samples_processed: u64,
    
    // True peak detection
    true_peak_left: f64,
    true_peak_right: f64,
}

impl LoudnessMeter {
    pub fn new(sample_rate: f64) -> Self {
        let momentary_blocks = ((MOMENTARY_WINDOW * sample_rate / BLOCK_SIZE as f64).ceil() as usize).max(1);
        let short_term_blocks = ((SHORT_TERM_WINDOW * sample_rate / BLOCK_SIZE as f64).ceil() as usize).max(1);
        
        Self {
            sample_rate,
            k_filter: KWeightFilter::new(sample_rate),
            block_left_sum: 0.0,
            block_right_sum: 0.0,
            block_sample_count: 0,
            block_history: VecDeque::new(),
            integrated_blocks: Vec::new(),
            momentary_blocks,
            short_term_blocks,
            samples_processed: 0,
            true_peak_left: 0.0,
            true_peak_right: 0.0,
        }
    }
    
    /// Process a stereo sample
    pub fn process(&mut self, left: f32, right: f32) {
        // K-weight filtering
        let (weighted_l, weighted_r) = self.k_filter.process(left as f64, right as f64);
        
        // Accumulate squared values (power)
        self.block_left_sum += weighted_l * weighted_l;
        self.block_right_sum += weighted_r * weighted_r;
        self.block_sample_count += 1;
        
        // True peak detection (simple approach - needs oversampling for true spec compliance)
        let abs_l = left.abs() as f64;
        let abs_r = right.abs() as f64;
        if abs_l > self.true_peak_left {
            self.true_peak_left = abs_l;
        }
        if abs_r > self.true_peak_right {
            self.true_peak_right = abs_r;
        }
        
        self.samples_processed += 1;
        
        // Complete block?
        if self.block_sample_count >= BLOCK_SIZE {
            self.finalize_block();
        }
    }
    
    fn finalize_block(&mut self) {
        if self.block_sample_count == 0 {
            return;
        }
        
        // Mean square power for block
        let mean_square_l = self.block_left_sum / self.block_sample_count as f64;
        let mean_square_r = self.block_right_sum / self.block_sample_count as f64;
        
        // Stereo power (simple average - could use channel weighting)
        let block_power = (mean_square_l + mean_square_r) / 2.0;
        
        let timestamp = self.samples_processed as f64 / self.sample_rate;
        
        self.block_history.push_back(BlockPower { power: block_power, timestamp });
        
        // Keep history for short-term window (longest window needed)
        while self.block_history.len() > self.short_term_blocks * 2 {
            self.block_history.pop_front();
        }
        
        // Store for integrated calculation
        self.integrated_blocks.push(block_power);
        
        // Reset accumulators
        self.block_left_sum = 0.0;
        self.block_right_sum = 0.0;
        self.block_sample_count = 0;
    }
    
    /// Get momentary loudness (400ms, ITU-R BS.1771)
    pub fn get_momentary(&self) -> f64 {
        self.get_window_loudness(self.momentary_blocks)
    }
    
    /// Get short-term loudness (3s, EBU R128)
    pub fn get_short_term(&self) -> f64 {
        self.get_window_loudness(self.short_term_blocks)
    }
    
    fn get_window_loudness(&self, num_blocks: usize) -> f64 {
        if self.block_history.is_empty() {
            return -f64::INFINITY;
        }
        
        let blocks_to_use = num_blocks.min(self.block_history.len());
        let start_idx = self.block_history.len().saturating_sub(blocks_to_use);
        
        let mut sum = 0.0;
        for i in start_idx..self.block_history.len() {
            sum += self.block_history[i].power;
        }
        
        let mean_power = sum / blocks_to_use as f64;
        
        if mean_power > 0.0 {
            -0.691 + 10.0 * mean_power.log10() // Convert to LUFS
        } else {
            -f64::INFINITY
        }
    }
    
    /// Get integrated loudness with gating (EBU R128)
    pub fn get_integrated(&self) -> f64 {
        if self.integrated_blocks.is_empty() {
            return -f64::INFINITY;
        }
        
        // Step 1: Absolute gate at -70 LUFS
        let absolute_threshold = 10_f64.powf((-70.0 + 0.691) / 10.0);
        let mut gated_blocks: Vec<f64> = self.integrated_blocks.iter()
            .filter(|&&p| p >= absolute_threshold)
            .copied()
            .collect();
        
        if gated_blocks.is_empty() {
            return -f64::INFINITY;
        }
        
        // Step 2: Relative gate at -10 LU below ungated mean
        let ungated_mean: f64 = gated_blocks.iter().sum::<f64>() / gated_blocks.len() as f64;
        let ungated_lufs = -0.691 + 10.0 * ungated_mean.log10();
        let relative_threshold = 10_f64.powf((ungated_lufs - 10.0 + 0.691) / 10.0);
        
        gated_blocks.retain(|&p| p >= relative_threshold);
        
        if gated_blocks.is_empty() {
            return -f64::INFINITY;
        }
        
        let gated_mean: f64 = gated_blocks.iter().sum::<f64>() / gated_blocks.len() as f64;
        -0.691 + 10.0 * gated_mean.log10()
    }
    
    /// Get loudness range (LRA, EBU R128)
    pub fn get_loudness_range(&self) -> f64 {
        if self.integrated_blocks.len() < 10 {
            return 0.0;
        }
        
        // Convert blocks to loudness values
        let mut loudness_values: Vec<f64> = self.integrated_blocks.iter()
            .map(|&p| if p > 0.0 { -0.691 + 10.0 * p.log10() } else { -f64::INFINITY })
            .filter(|&l| l.is_finite())
            .collect();
        
        if loudness_values.is_empty() {
            return 0.0;
        }
        
        loudness_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // LRA is difference between 95th and 10th percentile
        let idx_10 = (loudness_values.len() as f64 * 0.10) as usize;
        let idx_95 = (loudness_values.len() as f64 * 0.95) as usize;
        
        loudness_values[idx_95] - loudness_values[idx_10]
    }
    
    /// Get true peak in dBTP
    pub fn get_true_peak_dbtp(&self) -> (f64, f64) {
        let left_dbtp = if self.true_peak_left > 0.0 {
            20.0 * self.true_peak_left.log10()
        } else {
            -f64::INFINITY
        };
        
        let right_dbtp = if self.true_peak_right > 0.0 {
            20.0 * self.true_peak_right.log10()
        } else {
            -f64::INFINITY
        };
        
        (left_dbtp, right_dbtp)
    }
    
    /// Get all loudness measurements at once
    pub fn get_measurements(&self) -> LoudnessMeasurements {
        let (true_peak_l, true_peak_r) = self.get_true_peak_dbtp();
        LoudnessMeasurements {
            momentary: self.get_momentary() as f32,
            short_term: self.get_short_term() as f32,
            integrated: self.get_integrated() as f32,
            loudness_range: self.get_loudness_range() as f32,
            true_peak_dbtp: true_peak_l.max(true_peak_r) as f32,
        }
    }
    
    /// Reset all measurements
    pub fn reset(&mut self) {
        self.block_left_sum = 0.0;
        self.block_right_sum = 0.0;
        self.block_sample_count = 0;
        self.block_history.clear();
        self.integrated_blocks.clear();
        self.samples_processed = 0;
        self.true_peak_left = 0.0;
        self.true_peak_right = 0.0;
    }
}
