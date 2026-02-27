/// Audio routing engine
use crate::audio_io::ChannelSelection;
use crate::equalizer::{Equalizer, ParametricEqualizer};
use crate::file_player::AudioFilePlayer;
use crate::signal_gen::{SignalGenerator, WaveformType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackSource {
    None,
    AudioInput,
    SignalGenerator,
    FilePlayer,
}

pub struct Track {
    pub id: usize,
    pub source: TrackSource,
    pub gain: f32,      // Input gain/trim (linear, typically 0.25 to 4.0 for -12dB to +12dB)
    pub volume: f32,    // Fader volume (linear, 0.0 to ~4.0 for -∞ to +12dB)
    pub mute: bool,
    pub pan: f32, // -1.0 (left) to 1.0 (right)
    
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
    
    // Processing state
    pub level_l: f32,
    pub level_r: f32,
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
            input_channel_selection: ChannelSelection::stereo(),
            signal_generator: None,
            file_player: None,
            equalizer: Equalizer::new(48000.0),
            parametric_eq: ParametricEqualizer::new(48000.0),
            level_l: 0.0,
            level_r: 0.0,
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
        eprintln!("[Track {}] EQ: Low={:.1}dB, LowMid={:.1}dB, HighMid={:.1}dB, High={:.1}dB", 
            self.id, low, low_mid, high_mid, high);
    }

    /// Enable or disable EQ
    pub fn set_eq_enabled(&mut self, enabled: bool) {
        self.equalizer.set_enabled(enabled);
        eprintln!("[Track {}] EQ Enabled: {}", self.id, enabled);
    }

    /// Play file
    pub fn play_file(&mut self, output_sample_rate: u32) -> anyhow::Result<()> {
        if let Some(player) = &mut self.file_player {
            player.set_output_sample_rate(output_sample_rate);
            player.play();
            eprintln!("[Track {}] File playback started (playing={}, file_rate={}, output_rate={}, ratio={:.4})", 
                self.id, player.playing, player.sample_rate, player.output_sample_rate,
                player.sample_rate as f64 / player.output_sample_rate as f64);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no file loaded", self.id))
        }
    }

    /// Pause file
    pub fn pause_file(&mut self) -> anyhow::Result<()> {
        if let Some(player) = &mut self.file_player {
            player.pause();
            eprintln!("[Track {}] File playback paused", self.id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no file loaded", self.id))
        }
    }

    /// Stop file
    pub fn stop_file(&mut self) -> anyhow::Result<()> {
        if let Some(player) = &mut self.file_player {
            player.stop();
            eprintln!("[Track {}] File playback stopped", self.id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no file loaded", self.id))
        }
    }

    /// Process audio for this track
    /// Returns (left, right) stereo samples
    pub fn process(&mut self, input_frame: Option<&[f32]>) -> (f32, f32) {
        if self.mute {
            return (0.0, 0.0);
        }

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

        // Apply input gain/trim first
        left *= self.gain;
        right *= self.gain;

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

        (left, right)
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

        // Sum all tracks
        for track in tracks.iter_mut() {
            let (l, r) = track.process(input_frame);
            mix_l += l;
            mix_r += r;
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

pub struct Router {
    pub tracks: Vec<Track>,
    pub master: MasterBus,
}

impl Router {
    pub fn new(num_tracks: usize) -> Self {
        let tracks = (0..num_tracks).map(Track::new).collect();
        
        Self {
            tracks,
            master: MasterBus::new(),
        }
    }

    /// Process one frame of audio
    pub fn process_frame(&mut self, input_frame: Option<&[f32]>) -> (f32, f32) {
        self.master.process(&mut self.tracks, input_frame)
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
