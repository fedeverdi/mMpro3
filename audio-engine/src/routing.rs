/// Audio routing engine
use crate::audio_io::ChannelSelection;
use crate::compressor::Compressor;
use crate::delay::Delay;
use crate::equalizer::{Equalizer, ParametricEqualizer, EQBand, FilterType};
use crate::file_player::AudioFilePlayer;
use crate::gate::NoiseGate;
use crate::limiter::Limiter;
use crate::loudness::LoudnessMeter;
use crate::dynamic_range::DynamicRangeMeter;
use crate::phase_correlation::PhaseCorrelationMeter;
use crate::stereo_width::StereoWidthMeter;
use crate::headroom::HeadroomMeter;
use crate::reverb::Reverb;
use crate::signal_gen::{SignalGenerator, WaveformType};
use rustfft::{FftPlanner, num_complex::Complex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackSource {
    None,
    AudioInput,
    SignalGenerator,
    FilePlayer,
    AuxReturn(usize), // Aux bus ID for aux return/effect return
}

// Ring buffer size for waveform display (2048 samples = ~42ms @ 48kHz)
const WAVEFORM_BUFFER_SIZE: usize = 2048;

// FFT size for spectrum analysis (2048 samples = ~42ms @ 48kHz)
// Balanced between update rate and CPU usage
const FFT_SIZE: usize = 2048;

// Maximum number of aux buses
const MAX_AUX_BUSES: usize = 6;

/// Auxiliary send from a track to an aux bus
#[derive(Debug, Clone)]
pub struct AuxSend {
    pub level: f32,      // Send level (linear, 0.0 to ~4.0)
    pub pre_fader: bool, // If true, send is before track fader; if false, after
    pub muted: bool,     // Mute this send
}

impl Default for AuxSend {
    fn default() -> Self {
        Self {
            level: 0.0,      // -∞ dB
            pre_fader: false, // Post-fader by default
            muted: true,     // Muted by default
        }
    }
}

pub struct FFTAnalyzer {
    buffer_left: Vec<f32>,
    buffer_right: Vec<f32>,
    position: usize,
    fft_ready: bool,
    // Pre-calculated window function
    window: Vec<f32>,
    // Pre-allocated FFT buffers
    fft_left: Vec<Complex<f32>>,
    fft_right: Vec<Complex<f32>>,
    // Cached FFT plan
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
}

impl FFTAnalyzer {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        
        // Pre-calculate Blackman-Harris window (better sidelobe suppression than Hann)
        // Blackman-Harris has ~92dB sidelobe attenuation vs Hann's ~31dB
        let window: Vec<f32> = (0..FFT_SIZE)
            .map(|i| {
                let n = i as f32;
                let N = (FFT_SIZE - 1) as f32;
                let a0 = 0.35875;
                let a1 = 0.48829;
                let a2 = 0.14128;
                let a3 = 0.01168;
                a0 - a1 * f32::cos(2.0 * std::f32::consts::PI * n / N)
                   + a2 * f32::cos(4.0 * std::f32::consts::PI * n / N)
                   - a3 * f32::cos(6.0 * std::f32::consts::PI * n / N)
            })
            .collect();
        
        // Pre-allocate FFT buffers
        let fft_left = vec![Complex::new(0.0, 0.0); FFT_SIZE];
        let fft_right = vec![Complex::new(0.0, 0.0); FFT_SIZE];
        
        Self {
            buffer_left: vec![0.0; FFT_SIZE],
            buffer_right: vec![0.0; FFT_SIZE],
            position: 0,
            fft_ready: false,
            window,
            fft_left,
            fft_right,
            fft,
        }
    }

    /// Add samples to the FFT buffer
    pub fn push_samples(&mut self, left: f32, right: f32) {
        self.buffer_left[self.position] = left;
        self.buffer_right[self.position] = right;
        self.position += 1;

        if self.position >= FFT_SIZE {
            self.position = 0;
            self.fft_ready = true;
        }
    }

    /// Reset the FFT buffer (useful when switching signal sources)
    pub fn reset(&mut self) {
        self.buffer_left.fill(0.0);
        self.buffer_right.fill(0.0);
        self.position = 0;
        self.fft_ready = false;
    }

    /// Perform FFT analysis and return magnitude spectrum (half of FFT_SIZE due to symmetry)
    pub fn analyze(&mut self) -> Option<(Vec<f32>, Vec<f32>)> {
        if !self.fft_ready {
            return None;
        }

        self.fft_ready = false;

        // Apply window to left channel (reuse pre-allocated buffer)
        for i in 0..FFT_SIZE {
            self.fft_left[i] = Complex::new(self.buffer_left[i] * self.window[i], 0.0);
        }

        // Apply window to right channel (reuse pre-allocated buffer)
        for i in 0..FFT_SIZE {
            self.fft_right[i] = Complex::new(self.buffer_right[i] * self.window[i], 0.0);
        }

        // Perform FFT (in-place on pre-allocated buffers)
        self.fft.process(&mut self.fft_left);
        self.fft.process(&mut self.fft_right);

        // Calculate magnitude spectrum (only first half due to symmetry)
        // Blackman-Harris window has coherent gain of ~0.36
        // We use a calibrated compensation factor to make 0 dBFS sine wave = 0 dB on display
        let bins_count = FFT_SIZE / 2;
        let window_compensation = 2.0; // Calibrated for proper 0 dB reference
        let normalization = (window_compensation * 2.0) / FFT_SIZE as f32;
        
        let left_magnitudes: Vec<f32> = self.fft_left[..bins_count]
            .iter()
            .map(|c| {
                let magnitude = (c.re * c.re + c.im * c.im).sqrt();
                magnitude * normalization
            })
            .collect();

        let right_magnitudes: Vec<f32> = self.fft_right[..bins_count]
            .iter()
            .map(|c| {
                let magnitude = (c.re * c.re + c.im * c.im).sqrt();
                magnitude * normalization
            })
            .collect();

        Some((left_magnitudes, right_magnitudes))
    }
}

pub struct Track {
    pub id: usize,
    pub source: TrackSource,
    pub gain: f32,      // Input gain/trim (linear, typically 0.25 to 4.0 for -12dB to +12dB)
    pub volume: f32,    // Fader volume (linear, 0.0 to ~4.0 for -∞ to +12dB)
    pub mute: bool,
    pub pan: f32, // -1.0 (left) to 1.0 (right)
    pub route_to_master: bool, // Whether to send output to master bus
    pub route_to_subgroups: Vec<usize>, // List of subgroup IDs to route to
    pub aux_sends: Vec<AuxSend>, // Aux sends (up to MAX_AUX_BUSES)
    pub pad_enabled: bool, // -24dB attenuation before gain
    pub hpf_enabled: bool, // High-pass filter @ 80Hz (between PAD and gain)
    pub phase_inverted: bool, // Phase inversion (180° polarity flip)
    
    // Audio input
    pub input_channel_selection: ChannelSelection,
    
    // Signal generator
    pub signal_generator: Option<SignalGenerator>,
    
    // File player
    pub file_player: Option<AudioFilePlayer>,
    
    // Compressor (before EQ)
    pub compressor: Compressor,
    
    // Noise Gate (after gain)
    pub gate: NoiseGate,
    
    // Equalizer (4-band fixed)
    pub equalizer: Equalizer,
    
    // Parametric EQ (dynamic filters)
    pub parametric_eq: ParametricEqualizer,
    
    // HPF filter (80Hz high-pass)
    hpf_filter: EQBand,
    
    // Processing state
    pub level_l: f32,
    pub level_r: f32,
    
    // Phase correlation (-1 to +1)
    pub phase_correlation: f32,
    
    // Aux send outputs (stereo pairs for each aux bus)
    pub aux_outputs: Vec<(f32, f32)>,
    
    // Waveform ring buffer (for visualization)
    waveform_buffer_l: Vec<f32>,
    waveform_buffer_r: Vec<f32>,
    waveform_write_index: usize,
    
    // Phase correlation calculation counter (calculate every N samples)
    phase_correlation_counter: usize,
}

impl Track {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            source: TrackSource::None,
            gain: 0.251189,      // -12 dB (safe default to prevent clipping)
            volume: 1.0,    // Unity volume (0dB)
            mute: false,
            pan: 0.0,
            route_to_master: true, // Route to master by default
            route_to_subgroups: Vec::new(), // No subgroups by default
            aux_sends: vec![AuxSend::default(); MAX_AUX_BUSES], // Initialize all aux sends
            pad_enabled: false, // PAD off by default
            hpf_enabled: false, // HPF off by default
            phase_inverted: false, // Phase inversion off by default
            input_channel_selection: ChannelSelection::stereo(),
            signal_generator: None,
            file_player: None,
            compressor: Compressor::new(48000.0),
            gate: NoiseGate::new(48000.0),
            equalizer: Equalizer::new(48000.0),
            parametric_eq: ParametricEqualizer::new(48000.0),
            hpf_filter: EQBand::new(FilterType::HighPass, 80.0, 48000.0),
            level_l: 0.0,
            level_r: 0.0,
            phase_correlation: 0.0,
            aux_outputs: vec![(0.0, 0.0); MAX_AUX_BUSES], // Initialize all aux outputs
            waveform_buffer_l: vec![0.0; WAVEFORM_BUFFER_SIZE],
            waveform_buffer_r: vec![0.0; WAVEFORM_BUFFER_SIZE],
            waveform_write_index: 0,
            phase_correlation_counter: 0,
        }
    }

    /// Set track source to audio input
    pub fn set_audio_input(&mut self, channel_selection: ChannelSelection) {
        self.source = TrackSource::AudioInput;
        self.input_channel_selection = channel_selection;
        self.signal_generator = None;
        self.file_player = None;
        
        // Set default gain to 0.5 for microphone input (to prevent clipping on hot signals)
        self.gain = 0.5;
    }

    /// Set track source to signal generator
    pub fn set_signal_generator(&mut self, waveform: WaveformType, frequency: f32, sample_rate: f32) {
        self.source = TrackSource::SignalGenerator;
        self.signal_generator = Some(SignalGenerator::new(waveform, frequency, sample_rate));
        self.file_player = None;
    }

    /// Set track source to file player
    pub fn set_file_player(&mut self, player: AudioFilePlayer) {
        self.source = TrackSource::FilePlayer;
        self.file_player = Some(player);
        self.signal_generator = None;
    }

    /// Set EQ parameters
    pub fn set_eq(&mut self, low: f32, low_mid: f32, high_mid: f32, high: f32) {
        self.equalizer.set_low_shelf(low);
        self.equalizer.set_low_mid(low_mid);
        self.equalizer.set_high_mid(high_mid);
        self.equalizer.set_high_shelf(high);
    }

    /// Enable or disable EQ
    pub fn set_eq_enabled(&mut self, enabled: bool) {
        self.equalizer.set_enabled(enabled);
    }

    /// Play file
    pub fn play_file(&mut self, output_sample_rate: u32) -> anyhow::Result<()> {
        if let Some(player) = &mut self.file_player {
            player.set_output_sample_rate(output_sample_rate);
            player.play();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no file loaded", self.id))
        }
    }

    /// Pause file
    pub fn pause_file(&mut self) -> anyhow::Result<()> {
        if let Some(player) = &mut self.file_player {
            player.pause();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no file loaded", self.id))
        }
    }

    /// Stop file
    pub fn stop_file(&mut self) -> anyhow::Result<()> {
        if let Some(player) = &mut self.file_player {
            player.stop();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no file loaded", self.id))
        }
    }

    /// Update sample rate for all components
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        // Update sample rate for all processors
        self.compressor.set_sample_rate(sample_rate);
        self.gate.set_sample_rate(sample_rate);
        self.equalizer.set_sample_rate(sample_rate);
        self.parametric_eq.set_sample_rate(sample_rate);
        self.hpf_filter.set_sample_rate(sample_rate);
        
        // CRITICAL: Reset all effect buffers to clear old audio at previous sample rate
        // This prevents pitch/speed artifacts when switching devices
        self.compressor.reset();
        self.gate.reset();
        self.equalizer.reset();
        self.parametric_eq.reset();
        self.hpf_filter.reset();
        
        // Update file player output sample rate
        if let Some(player) = &mut self.file_player {
            player.set_output_sample_rate(sample_rate as u32);
        }
        
        // Update signal generator sample rate
        if let Some(generator) = &mut self.signal_generator {
            generator.set_sample_rate(sample_rate);
        }
    }

    /// Process audio for this track
    /// Returns (left, right) stereo samples
    pub fn process(&mut self, input_frame: Option<&[f32]>, aux_returns: &[(f32, f32)]) -> (f32, f32) {
        let (left, right) = match self.source {
            TrackSource::None => (0.0, 0.0),
            
            TrackSource::AudioInput => {
                if let Some(input) = input_frame {
                    let left_idx = self.input_channel_selection.left as usize;
                    let right_idx = self.input_channel_selection.right as usize;
                    
                    let l = input.get(left_idx).copied().unwrap_or(0.0);
                    let r = input.get(right_idx).copied().unwrap_or(0.0);
                    (l, r)
                } else {
                    (0.0, 0.0)
                }
            }
            
            TrackSource::SignalGenerator => {
                if let Some(generator) = &mut self.signal_generator {
                    let sample = generator.next_sample();
                    // Debug first few samples
                    static SAMPLE_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                    let count = SAMPLE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    (sample, sample)
                } else {
                    (0.0, 0.0)
                }
            }
            
            TrackSource::FilePlayer => {
                if let Some(player) = &mut self.file_player {
                    player.next_frame()
                } else {
                    (0.0, 0.0)
                }
            }
            
            TrackSource::AuxReturn(aux_id) => {
                // Get aux bus output from previous frame (aux returns)
                aux_returns.get(aux_id).copied().unwrap_or((0.0, 0.0))
            }
        };

        // ===== INPUT STAGE (Preamp Section) =====
        // 0. PHASE INVERSION: Flip polarity (180° phase shift)
        let (left, right) = if self.phase_inverted {
            (-left, -right)
        } else {
            (left, right)
        };

        // 1. PAD: Attenuate very hot signals before preamp (-24dB)
        let (mut left, mut right) = if self.pad_enabled {
            const PAD_ATTENUATION: f32 = 0.063095734; // 10^(-24/20)
            (left * PAD_ATTENUATION, right * PAD_ATTENUATION)
        } else {
            (left, right)
        };

        // 2. GAIN/TRIM: Preamp amplification (input stage)
        left *= self.gain;
        right *= self.gain;

        // 3. HPF: High-pass filter @ 80Hz (removes rumble/sub-sonic noise)
        let (left, right) = if self.hpf_enabled {
            self.hpf_filter.process(left, right)
        } else {
            (left, right)
        };

        // ===== INSERT/DYNAMICS SECTION =====
        // 4. GATE: Noise gate (eliminates unwanted noise)
        let (left, right) = self.gate.process(left, right);

        // 5. COMPRESSOR: Dynamic range control
        let (left, right) = self.compressor.process(left, right);

        // 6. EQ: Tone shaping (4-band shelving/bell)
        let (left, right) = self.equalizer.process(left, right);
        
        // 7. PARAMETRIC EQ: Surgical frequency control
        let (left, right) = self.parametric_eq.process(left, right);

        // ===== OUTPUT STAGE (Channel Section) =====
        // 8. PAN: Stereo positioning
        let (mut left, mut right) = if self.pan < 0.0 {
            // Pan left: reduce right channel
            (left, right * (1.0 + self.pan))
        } else if self.pan > 0.0 {
            // Pan right: reduce left channel
            (left * (1.0 - self.pan), right)
        } else {
            (left, right)
        };

        // Store pre-fader signal for aux sends
        let pre_fader_l = left;
        let pre_fader_r = right;

        // 9. FADER: Final level control
        left *= self.volume;
        right *= self.volume;

        // ===== AUX SENDS SECTION =====
        // Calculate all aux send outputs (PRE or POST fader based on send configuration)
        for (i, aux_send) in self.aux_sends.iter().enumerate() {
            if aux_send.muted {
                self.aux_outputs[i] = (0.0, 0.0);
            } else {
                let (source_l, source_r) = if aux_send.pre_fader {
                    (pre_fader_l, pre_fader_r)
                } else {
                    (left, right)
                };
                self.aux_outputs[i] = (source_l * aux_send.level, source_r * aux_send.level);
            }
        }

        // Update levels for metering (peak hold)
        self.level_l = self.level_l.max(left.abs());
        self.level_r = self.level_r.max(right.abs());

        // Capture samples in waveform ring buffer
        self.waveform_buffer_l[self.waveform_write_index] = left;
        self.waveform_buffer_r[self.waveform_write_index] = right;
        self.waveform_write_index = (self.waveform_write_index + 1) % WAVEFORM_BUFFER_SIZE;
        
        // Calculate phase correlation MUCH less frequently (every 4096 samples = ~85ms @ 48kHz)
        // to avoid overloading the audio thread
        self.phase_correlation_counter += 1;
        if self.phase_correlation_counter >= 4096 {
            self.phase_correlation_counter = 0;
            self.phase_correlation = Self::calculate_phase_correlation(&self.waveform_buffer_l, &self.waveform_buffer_r);
        }

        // Apply mute after level calculation (so meters still work)
        let output = if self.mute {
            // NO eprintln! here - it blocks the audio thread!
            (0.0, 0.0)
        } else {
            (left, right)
        };
        
        output
    }

    /// Get waveform buffer for visualization (returns samples in chronological order)
    pub fn get_waveform_buffer(&self, max_samples: usize) -> Vec<f32> {
        let count = max_samples.min(WAVEFORM_BUFFER_SIZE);
        let mut result = Vec::with_capacity(count);
        
        // Calculate start position (oldest sample in ring buffer)
        let start_index = self.waveform_write_index;
        
        // Copy samples in chronological order, downsampling if needed
        let step = WAVEFORM_BUFFER_SIZE / count;
        
        for i in 0..count {
            let buffer_index = (start_index + i * step) % WAVEFORM_BUFFER_SIZE;
            // Mix left and right channels for mono display
            let sample = (self.waveform_buffer_l[buffer_index] + self.waveform_buffer_r[buffer_index]) * 0.5;
            result.push(sample);
        }
        
        result
    }
    
    /// Calculate phase correlation between left and right channels
    /// Returns a value between -1 (completely out of phase) and +1 (completely in phase/mono)
    /// Formula: correlation = sum(L * R) / sqrt(sum(L^2) * sum(R^2))
    /// Optimized: calculate only on last 512 samples instead of all 2048 to reduce CPU load
    fn calculate_phase_correlation(buffer_l: &[f32], buffer_r: &[f32]) -> f32 {
        // Use only last 512 samples for faster calculation (still gives accurate result)
        let sample_count = 512.min(buffer_l.len());
        let start_idx = buffer_l.len().saturating_sub(sample_count);
        
        let mut sum_lr = 0.0_f64;  // Sum of L * R
        let mut sum_l2 = 0.0_f64;  // Sum of L^2
        let mut sum_r2 = 0.0_f64;  // Sum of R^2
        
        for i in start_idx..buffer_l.len() {
            let l = buffer_l[i] as f64;
            let r = buffer_r[i] as f64;
            
            sum_lr += l * r;
            sum_l2 += l * l;
            sum_r2 += r * r;
        }
        
        // Avoid division by zero
        if sum_l2 < 1e-10 || sum_r2 < 1e-10 {
            return 0.0; // No signal or very weak signal
        }
        
        let correlation = sum_lr / (sum_l2 * sum_r2).sqrt();
        
        // Clamp result to valid range [-1, +1] (floating point precision can cause slight overflow)
        correlation.clamp(-1.0, 1.0) as f32
    }

    /// Reset peak levels for metering
    pub fn reset_levels(&mut self) {
        self.level_l = 0.0;
        self.level_r = 0.0;
    }
}

pub struct MasterBus {
    pub gain: f32,
    pub mute: bool,
    pub parametric_eq: ParametricEqualizer,
    pub output_channel_selection: ChannelSelection,
    pub level_l: f32,
    pub level_r: f32,
    
    // Master FX Chain (applied in series after EQ)
    pub compressor: Compressor,
    pub limiter: Limiter,
    pub delay: Delay,
    pub reverb: Reverb,
}

impl MasterBus {
    pub fn new() -> Self {
        Self {
            gain: 1.0,
            mute: false,
            parametric_eq: ParametricEqualizer::new(48000.0),
            output_channel_selection: ChannelSelection::stereo(),
            level_l: 0.0,
            level_r: 0.0,
            compressor: Compressor::new(48000.0),
            limiter: Limiter::new(48000.0),
            delay: Delay::new(48000.0),
            reverb: Reverb::new(48000.0),
        }
    }
    
    /// Set sample rate for all master FX
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        // Update sample rate
        self.parametric_eq.set_sample_rate(sample_rate);
        self.compressor.set_sample_rate(sample_rate);
        self.limiter.set_sample_rate(sample_rate);
        self.delay.set_sample_rate(sample_rate);
        self.reverb.set_sample_rate(sample_rate);
        
        // CRITICAL: Reset all effect buffers to clear old audio at previous sample rate
        // This prevents pitch/speed artifacts when switching devices
        self.parametric_eq.reset();
        self.compressor.reset();
        self.limiter.reset();
        self.delay.reset();
        self.reverb.reset();
    }

    /// Mix all tracks and apply master processing (using cached track outputs)
    pub fn process_cached(&mut self, tracks: &[Track], track_outputs: &[(f32, f32)]) -> (f32, f32) {
        if self.mute {
            return (0.0, 0.0);
        }

        let mut mix_l = 0.0;
        let mut mix_r = 0.0;

        // Sum all tracks routed to master (parallel routing - tracks can go to both master AND subgroups)
        for (i, track) in tracks.iter().enumerate() {
            if track.route_to_master {
                let (l, r) = track_outputs[i];
                mix_l += l;
                mix_r += r;
            }
        }

        // Apply master parametric EQ BEFORE gain (typical mixing practice)
        let (mix_l, mix_r) = self.parametric_eq.process(mix_l, mix_r);

        // Apply master gain
        let mix_l = mix_l * self.gain;
        let mix_r = mix_r * self.gain;

        // ===== MASTER FX CHAIN (in series) =====
        // 1. Compressor: Dynamic range control
        let (mix_l, mix_r) = self.compressor.process(mix_l, mix_r);
        
        // 2. Reverb: Spatial effect
        let (mix_l, mix_r) = self.reverb.process(mix_l, mix_r);
        
        // 3. Delay: Time-based effect
        let (mix_l, mix_r) = self.delay.process(mix_l, mix_r);
        
        // 4. Limiter: Brick-wall limiting to prevent clipping (last, catches all peaks)
        let (mix_l, mix_r) = self.limiter.process(mix_l, mix_r);

        // Update levels (peak hold) - AFTER all processing
        self.level_l = self.level_l.max(mix_l.abs());
        self.level_r = self.level_r.max(mix_r.abs());

        (mix_l, mix_r)
    }

    /// Reset peak levels for metering
    pub fn reset_levels(&mut self) {
        self.level_l = 0.0;
        self.level_r = 0.0;
    }
}

impl Default for MasterBus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SubgroupBus {
    pub id: usize,
    pub gain: f32,
    pub mute: bool,
    pub route_to_master: bool,
    pub output_enabled: bool, // Controls direct output (independent from route_to_master)
    pub output_channel_selection: ChannelSelection,
    pub level_l: f32,
    pub level_r: f32,
}

impl SubgroupBus {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            gain: 1.0,
            mute: false,
            route_to_master: false,
            output_enabled: false, // No direct output by default
            output_channel_selection: ChannelSelection::stereo(),
            level_l: 0.0,
            level_r: 0.0,
        }
    }

    /// Process all tracks and aux buses routed to this subgroup (using cached outputs)
    pub fn process_cached(&mut self, tracks: &[Track], track_outputs: &[(f32, f32)], aux_buses: &[AuxBus], aux_outputs: &[(f32, f32)]) -> (f32, f32) {
        if self.mute {
            return (0.0, 0.0);
        }

        let mut mix_l = 0.0;
        let mut mix_r = 0.0;

        // Sum all tracks routed to this subgroup
        for (i, track) in tracks.iter().enumerate() {
            if track.route_to_subgroups.contains(&self.id) {
                let (l, r) = track_outputs[i];
                mix_l += l;
                mix_r += r;
            }
        }

        // Sum all aux buses routed to this subgroup
        for (i, aux_bus) in aux_buses.iter().enumerate() {
            if aux_bus.route_to_subgroups.contains(&self.id) && i < aux_outputs.len() {
                let (l, r) = aux_outputs[i];
                mix_l += l;
                mix_r += r;
            }
        }

        // Apply subgroup gain
        mix_l *= self.gain;
        mix_r *= self.gain;

        // Update levels (peak hold)
        self.level_l = self.level_l.max(mix_l.abs());
        self.level_r = self.level_r.max(mix_r.abs());

        (mix_l, mix_r)
    }

    /// Reset peak levels for metering
    pub fn reset_levels(&mut self) {
        self.level_l = 0.0;
        self.level_r = 0.0;
    }
}

/// Auxiliary bus with reverb and delay effects
pub struct AuxBus {
    pub id: usize,
    pub gain: f32,
    pub mute: bool,
    pub route_to_master: bool,
    pub route_to_subgroups: Vec<usize>, // Which subgroups to route this aux to
    pub output_enabled: bool, // Controls direct output (independent from route_to_master)
    pub output_channel_selection: ChannelSelection,
    
    // FX Chain
    pub reverb: Reverb,
    pub delay: Delay,
    
    // Levels for metering
    pub level_l: f32,
    pub level_r: f32,
}

impl AuxBus {
    pub fn new(id: usize, sample_rate: f32) -> Self {
        Self {
            id,
            gain: 1.0,
            mute: false,
            route_to_master: false, // Not routed to master by default
            route_to_subgroups: Vec::new(), // No subgroup routing by default
            output_enabled: false, // No direct output by default
            output_channel_selection: ChannelSelection::stereo(),
            reverb: Reverb::new(sample_rate),
            delay: Delay::new(sample_rate),
            level_l: 0.0,
            level_r: 0.0,
        }
    }

    /// Process aux bus with all sends mixed together
    pub fn process(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        if self.mute {
            return (0.0, 0.0);
        }

        let mut l = input_l;
        let mut r = input_r;

        // Apply reverb (returns dry signal if disabled)
        let reverb_out = self.reverb.process(l, r);
        l = reverb_out.0;
        r = reverb_out.1;

        // Apply delay (returns dry signal if disabled)
        let delay_out = self.delay.process(l, r);
        l = delay_out.0;
        r = delay_out.1;

        // Apply gain
        l *= self.gain;
        r *= self.gain;

        // Update levels (peak hold)
        self.level_l = self.level_l.max(l.abs());
        self.level_r = self.level_r.max(r.abs());

        (l, r)
    }

    /// Reset peak levels for metering
    pub fn reset_levels(&mut self) {
        self.level_l = 0.0;
        self.level_r = 0.0;
    }

    /// Set sample rate for all effects
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.reverb.set_sample_rate(sample_rate);
        self.delay.set_sample_rate(sample_rate);
        
        // CRITICAL: Reset effect buffers to clear old audio at previous sample rate
        self.reverb.reset();
        self.delay.reset();
    }
}

pub struct Router {
    pub tracks: Vec<Track>,
    pub subgroups: Vec<SubgroupBus>,
    pub aux_buses: Vec<AuxBus>,
    pub master: MasterBus,
    pub fft_analyzer: FFTAnalyzer,
    pub loudness_meter: LoudnessMeter,
    pub dynamic_range_meter: DynamicRangeMeter,
    pub phase_correlation_meter: PhaseCorrelationMeter,
    pub stereo_width_meter: StereoWidthMeter,
    pub headroom_meter: HeadroomMeter,
    // Master bus output (before adding direct subgroups) for FFT analysis
    pub last_master_output: (f32, f32),
    // Cached outputs for multi-channel routing (populated by process_frame)
    pub last_subgroup_outputs: Vec<(f32, f32)>,
    pub last_aux_outputs: Vec<(f32,f32)>,
    // Pre-allocated buffers to avoid allocations in audio thread (PERFORMANCE CRITICAL)
    track_outputs_buffer: Vec<(f32, f32)>,
    aux_outputs_buffer: Vec<(f32, f32)>,
    subgroup_outputs_buffer: Vec<(f32, f32)>,
}

impl Router {
    pub fn new(num_tracks: usize) -> Self {
        let tracks = (0..num_tracks).map(Track::new).collect();
        
        // Initialize 6 aux buses by default
        let aux_buses = (0..MAX_AUX_BUSES).map(|i| AuxBus::new(i, 48000.0)).collect();
        
        Self {
            tracks,
            subgroups: Vec::new(),
            aux_buses,
            master: MasterBus::new(),
            fft_analyzer: FFTAnalyzer::new(),
            loudness_meter: LoudnessMeter::new(48000.0),
            dynamic_range_meter: DynamicRangeMeter::new(48000.0),
            phase_correlation_meter: PhaseCorrelationMeter::new(48000.0),
            stereo_width_meter: StereoWidthMeter::new(48000.0),
            headroom_meter: HeadroomMeter::new(48000.0),
            last_master_output: (0.0, 0.0),
            last_subgroup_outputs: Vec::new(),
            last_aux_outputs: Vec::new(),
            // Pre-allocate buffers with initial capacity (will grow as needed)
            track_outputs_buffer: Vec::with_capacity(num_tracks),
            aux_outputs_buffer: Vec::with_capacity(MAX_AUX_BUSES),
            subgroup_outputs_buffer: Vec::with_capacity(8), // Start with capacity for 8 subgroups
        }
    }

    /// Add a new subgroup
    pub fn add_subgroup(&mut self) -> usize {
        let id = self.subgroups.len();
        self.subgroups.push(SubgroupBus::new(id));
        id
    }

    /// Remove a subgroup
    pub fn remove_subgroup(&mut self, id: usize) {
        if id < self.subgroups.len() {
            self.subgroups.remove(id);
            // Re-index remaining subgroups
            for (new_id, subgroup) in self.subgroups.iter_mut().enumerate() {
                subgroup.id = new_id;
            }
            // Remove this subgroup from all track routings
            for track in self.tracks.iter_mut() {
                track.route_to_subgroups.retain(|&sg_id| sg_id != id);
                // Update IDs greater than removed one
                for sg_id in track.route_to_subgroups.iter_mut() {
                    if *sg_id > id {
                        *sg_id -= 1;
                    }
                }
            }
        }
    }

    /// Get subgroup by id
    pub fn get_subgroup_mut(&mut self, id: usize) -> Option<&mut SubgroupBus> {
        self.subgroups.get_mut(id)
    }

    /// Process one frame of audio
    pub fn process_frame(&mut self, input_frame: Option<&[f32]>) -> (f32, f32) {
        // PERFORMANCE OPTIMIZATION: Reuse pre-allocated buffers instead of allocating every frame
        // This eliminates ~144,000 allocations per second at 48kHz!
        
        // Process all tracks once and cache their outputs
        // Note: Tracks with AuxReturn source read from last_aux_outputs (1 frame latency)
        self.track_outputs_buffer.clear();
        for track in self.tracks.iter_mut() {
            let output = track.process(input_frame, &self.last_aux_outputs);
            self.track_outputs_buffer.push(output);
        }

        // Process all aux buses (sum all track aux sends for each aux)
        self.aux_outputs_buffer.clear();
        for aux_index in 0..self.aux_buses.len() {
            // Sum all track aux sends for this aux bus
            let mut aux_sum_l = 0.0;
            let mut aux_sum_r = 0.0;
            for track in self.tracks.iter() {
                if aux_index < track.aux_outputs.len() {
                    aux_sum_l += track.aux_outputs[aux_index].0;
                    aux_sum_r += track.aux_outputs[aux_index].1;
                }
            }
            
            // Process through aux bus (reverb, delay, gain)
            let aux_output = self.aux_buses[aux_index].process(aux_sum_l, aux_sum_r);
            self.aux_outputs_buffer.push(aux_output);
        }

        // Process all subgroups (using cached track and aux outputs)
        self.subgroup_outputs_buffer.clear();
        for subgroup in self.subgroups.iter_mut() {
            let output = subgroup.process_cached(&self.tracks, &self.track_outputs_buffer, &self.aux_buses, &self.aux_outputs_buffer);
            self.subgroup_outputs_buffer.push(output);
        }

        // Cache outputs for multi-channel routing (copy from buffers)
        self.last_subgroup_outputs.clear();
        self.last_subgroup_outputs.extend_from_slice(&self.subgroup_outputs_buffer);
        self.last_aux_outputs.clear();
        self.last_aux_outputs.extend_from_slice(&self.aux_outputs_buffer);

        // Process master bus (using cached track outputs)
        // Master processes all tracks with route_to_master enabled (parallel with subgroups)
        let mut master_output = self.master.process_cached(&self.tracks, &self.track_outputs_buffer);

        // Add subgroups routed to master INTO the master (apply master gain)
        for (i, subgroup) in self.subgroups.iter().enumerate() {
            if subgroup.route_to_master {
                master_output.0 += self.subgroup_outputs_buffer[i].0 * self.master.gain;
                master_output.1 += self.subgroup_outputs_buffer[i].1 * self.master.gain;
            }
        }
        
        // Add aux buses routed to master INTO the master (apply master gain)
        for (i, aux_bus) in self.aux_buses.iter().enumerate() {
            if aux_bus.route_to_master {
                master_output.0 += self.aux_outputs_buffer[i].0 * self.master.gain;
                master_output.1 += self.aux_outputs_buffer[i].1 * self.master.gain;
            }
        }
        
        // Update master levels to include routed subgroups and aux buses
        self.master.level_l = self.master.level_l.max(master_output.0.abs());
        self.master.level_r = self.master.level_r.max(master_output.1.abs());

        // Save master bus output for FFT analysis (before adding direct subgroups/aux)
        self.last_master_output = master_output;

        // Return ONLY master output (direct subgroups/aux are written separately in output callback)
        master_output
    }

    /// Get track by id
    pub fn get_track_mut(&mut self, id: usize) -> Option<&mut Track> {
        self.tracks.get_mut(id)
    }

    /// Stop all file players
    pub fn stop_all_files(&mut self) {
        for track in self.tracks.iter_mut() {
            if track.file_player.is_some() {
                let _ = track.stop_file();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_creation() {
        let track = Track::new(0);
        assert_eq!(track.id, 0);
        assert_eq!(track.source, TrackSource::None);
        assert_eq!(track.gain, 1.0);
        assert!(!track.mute);
    }

    #[test]
    fn test_router_creation() {
        let router = Router::new(4);
        assert_eq!(router.tracks.len(), 4);
    }

    #[test]
    fn test_muted_track() {
        let mut track = Track::new(0);
        track.mute = true;
        track.gain = 1.0;
        let (l, r) = track.process(None);
        assert_eq!(l, 0.0);
        assert_eq!(r, 0.0);
    }
}
