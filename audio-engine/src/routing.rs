/// Audio routing engine
use crate::audio_io::ChannelSelection;
use crate::equalizer::{Equalizer, ParametricEqualizer, EQBand, FilterType};
use crate::file_player::AudioFilePlayer;
use crate::signal_gen::{SignalGenerator, WaveformType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackSource {
    None,
    AudioInput,
    SignalGenerator,
    FilePlayer,
}

// Ring buffer size for waveform display (2048 samples = ~42ms @ 48kHz)
const WAVEFORM_BUFFER_SIZE: usize = 2048;

pub struct Track {
    pub id: usize,
    pub source: TrackSource,
    pub gain: f32,      // Input gain/trim (linear, typically 0.25 to 4.0 for -12dB to +12dB)
    pub volume: f32,    // Fader volume (linear, 0.0 to ~4.0 for -∞ to +12dB)
    pub mute: bool,
    pub pan: f32, // -1.0 (left) to 1.0 (right)
    pub route_to_master: bool, // Whether to send output to master bus
    pub route_to_subgroups: Vec<usize>, // List of subgroup IDs to route to
    pub pad_enabled: bool, // -24dB attenuation before gain
    pub hpf_enabled: bool, // High-pass filter @ 80Hz (between PAD and gain)
    
    // Audio input
    pub input_channel_selection: ChannelSelection,
    
    // Signal generator
    pub signal_generator: Option<SignalGenerator>,
    
    // File player
    pub file_player: Option<AudioFilePlayer>,
    
    // Equalizer (4-band fixed)
    pub equalizer: Equalizer,
    
    // Parametric EQ (dynamic filters)
    pub parametric_eq: ParametricEqualizer,
    
    // HPF filter (80Hz high-pass)
    hpf_filter: EQBand,
    
    // Processing state
    pub level_l: f32,
    pub level_r: f32,
    
    // Waveform ring buffer (for visualization)
    waveform_buffer_l: Vec<f32>,
    waveform_buffer_r: Vec<f32>,
    waveform_write_index: usize,
}

impl Track {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            source: TrackSource::None,
            gain: 1.0,      // Unity gain (0dB)
            volume: 1.0,    // Unity volume (0dB)
            mute: false,
            pan: 0.0,
            route_to_master: true, // Route to master by default
            route_to_subgroups: Vec::new(), // No subgroups by default
            pad_enabled: false, // PAD off by default
            hpf_enabled: false, // HPF off by default
            input_channel_selection: ChannelSelection::stereo(),
            signal_generator: None,
            file_player: None,
            equalizer: Equalizer::new(48000.0),
            parametric_eq: ParametricEqualizer::new(48000.0),
            hpf_filter: EQBand::new(FilterType::HighPass, 80.0, 48000.0),
            level_l: 0.0,
            level_r: 0.0,
            waveform_buffer_l: vec![0.0; WAVEFORM_BUFFER_SIZE],
            waveform_buffer_r: vec![0.0; WAVEFORM_BUFFER_SIZE],
            waveform_write_index: 0,
        }
    }

    /// Set track source to audio input
    pub fn set_audio_input(&mut self, channel_selection: ChannelSelection) {
        self.source = TrackSource::AudioInput;
        self.input_channel_selection = channel_selection;
        self.signal_generator = None;
        self.file_player = None;
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

    /// Process audio for this track
    /// Returns (left, right) stereo samples
    pub fn process(&mut self, input_frame: Option<&[f32]>) -> (f32, f32) {
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
        };

        // Apply 4-band EQ processing
        let (left, right) = self.equalizer.process(left, right);
        
        // Apply parametric EQ processing
        let (mut left, mut right) = self.parametric_eq.process(left, right);

        // Apply PAD attenuation (-24dB) if enabled (before gain)
        // -24dB = 10^(-24/20) ≈ 0.063095734
        if self.pad_enabled {
            const PAD_ATTENUATION: f32 = 0.063095734;
            left *= PAD_ATTENUATION;
            right *= PAD_ATTENUATION;
        }

        // Apply HPF (80Hz high-pass filter) if enabled (between PAD and gain)
        let (left, right) = if self.hpf_enabled {
            self.hpf_filter.process(left, right)
        } else {
            (left, right)
        };

        // Apply input gain/trim
        let (mut left, mut right) = (left * self.gain, right * self.gain);

        // Apply fader volume
        left *= self.volume;
        right *= self.volume;

        // Apply pan
        if self.pan < 0.0 {
            // Pan left: reduce right channel
            right *= 1.0 + self.pan;
        } else if self.pan > 0.0 {
            // Pan right: reduce left channel
            left *= 1.0 - self.pan;
        }

        // Update levels for metering (peak hold)
        self.level_l = self.level_l.max(left.abs());
        self.level_r = self.level_r.max(right.abs());

        // Capture samples in waveform ring buffer
        self.waveform_buffer_l[self.waveform_write_index] = left;
        self.waveform_buffer_r[self.waveform_write_index] = right;
        self.waveform_write_index = (self.waveform_write_index + 1) % WAVEFORM_BUFFER_SIZE;

        // Apply mute after level calculation (so meters still work)
        if self.mute {
            (0.0, 0.0)
        } else {
            (left, right)
        }
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

    /// Reset peak levels for metering
    pub fn reset_levels(&mut self) {
        self.level_l = 0.0;
        self.level_r = 0.0;
    }
}

pub struct MasterBus {
    pub gain: f32,
    pub mute: bool,
    pub output_channel_selection: ChannelSelection,
    pub level_l: f32,
    pub level_r: f32,
}

impl MasterBus {
    pub fn new() -> Self {
        Self {
            gain: 1.0,
            mute: false,
            output_channel_selection: ChannelSelection::stereo(),
            level_l: 0.0,
            level_r: 0.0,
        }
    }

    /// Mix all tracks and apply master processing
    pub fn process(&mut self, tracks: &mut [Track], input_frame: Option<&[f32]>) -> (f32, f32) {
        if self.mute {
            return (0.0, 0.0);
        }

        let mut mix_l = 0.0;
        let mut mix_r = 0.0;

        // Sum all tracks (only those routed to master)
        for track in tracks.iter_mut() {
            if track.route_to_master {
                let (l, r) = track.process(input_frame);
                mix_l += l;
                mix_r += r;
            } else {
                // Still process the track to update meters, but don't add to mix
                track.process(input_frame);
            }
        }

        // Apply master gain
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
            route_to_master: false, // Direct output by default
            output_channel_selection: ChannelSelection::stereo(),
            level_l: 0.0,
            level_r: 0.0,
        }
    }

    /// Process all tracks routed to this subgroup
    pub fn process(&mut self, tracks: &mut [Track], input_frame: Option<&[f32]>) -> (f32, f32) {
        if self.mute {
            return (0.0, 0.0);
        }

        let mut mix_l = 0.0;
        let mut mix_r = 0.0;

        // Sum all tracks routed to this subgroup
        for track in tracks.iter_mut() {
            if track.route_to_subgroups.contains(&self.id) {
                let (l, r) = track.process(input_frame);
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

pub struct Router {
    pub tracks: Vec<Track>,
    pub subgroups: Vec<SubgroupBus>,
    pub master: MasterBus,
}

impl Router {
    pub fn new(num_tracks: usize) -> Self {
        let tracks = (0..num_tracks).map(Track::new).collect();
        
        Self {
            tracks,
            subgroups: Vec::new(),
            master: MasterBus::new(),
        }
    }

    /// Add a new subgroup
    pub fn add_subgroup(&mut self) -> usize {
        let id = self.subgroups.len();
        self.subgroups.push(SubgroupBus::new(id));
        eprintln!("[Router] Subgroup {} created", id);
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
            eprintln!("[Router] Subgroup {} removed and re-indexed", id);
        }
    }

    /// Get subgroup by id
    pub fn get_subgroup_mut(&mut self, id: usize) -> Option<&mut SubgroupBus> {
        self.subgroups.get_mut(id)
    }

    /// Process one frame of audio
    pub fn process_frame(&mut self, input_frame: Option<&[f32]>) -> (f32, f32) {
        // Process all subgroups first (they consume track outputs)
        let mut subgroup_outputs: Vec<(f32, f32)> = Vec::new();
        for subgroup in self.subgroups.iter_mut() {
            let output = subgroup.process(&mut self.tracks, input_frame);
            subgroup_outputs.push(output);
        }

        // Process master bus (direct tracks + subgroups routed to master)
        let master_output = self.master.process(&mut self.tracks, input_frame);

        // Add subgroups routed to master
        let mut final_l = master_output.0;
        let mut final_r = master_output.1;
        for (i, subgroup) in self.subgroups.iter().enumerate() {
            if subgroup.route_to_master {
                final_l += subgroup_outputs[i].0;
                final_r += subgroup_outputs[i].1;
            }
        }

        (final_l, final_r)
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
        eprintln!("[Router] All file players stopped");
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
