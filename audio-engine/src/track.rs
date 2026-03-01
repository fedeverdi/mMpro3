/// Track source management utilities
use anyhow::{anyhow, Result};
use crate::audio_io::ChannelSelection;
use crate::file_player::AudioFilePlayer;
use crate::routing::{Router, TrackSource};
use crate::signal_gen::WaveformType;

/// Clear track source (set to None)
pub fn clear_source(
    router: &mut Router,
    track: usize,
) -> Result<()> {
    if let Some(t) = router.get_track_mut(track) {
        t.source = TrackSource::None;
        t.signal_generator = None;
        t.file_player = None;
        eprintln!("[Track {}] Source: None (cleared)", track);
        Ok(())
    } else {
        Err(anyhow!("Track {} not found", track))
    }
}

/// Set track source to audio input
pub fn set_source_input(
    router: &mut Router,
    track: usize,
    left_ch: u16,
    right_ch: u16,
) -> Result<()> {
    if let Some(t) = router.get_track_mut(track) {
        let channel_sel = ChannelSelection::new(left_ch, right_ch);
        t.set_audio_input(channel_sel);
        eprintln!("[Track {}] Source: Audio Input (L={}, R={})", track, left_ch, right_ch);
        Ok(())
    } else {
        Err(anyhow!("Track {} not found", track))
    }
}

/// Set track source to signal generator
pub fn set_source_signal(
    router: &mut Router,
    track: usize,
    waveform: &str,
    frequency: f32,
    sample_rate: u32,
) -> Result<()> {
    if let Some(t) = router.get_track_mut(track) {
        let wave = match waveform.to_lowercase().as_str() {
            "sine" => WaveformType::Sine,
            "square" => WaveformType::Square,
            "sawtooth" => WaveformType::Sawtooth,
            "triangle" => WaveformType::Triangle,
            "white" => WaveformType::WhiteNoise,
            "pink" => WaveformType::PinkNoise,
            _ => return Err(anyhow!("Unknown waveform: {}", waveform)),
        };
        t.set_signal_generator(wave, frequency, sample_rate as f32);
        Ok(())
    } else {
        Err(anyhow!("Track {} not found", track))
    }
}

/// Set track source to file player
pub fn set_source_file(
    router: &mut Router,
    track: usize,
    file_path: &str,
    sample_rate: u32,
) -> Result<()> {
    
    if let Some(t) = router.get_track_mut(track) {
        let mut player = AudioFilePlayer::new();
        player.set_output_sample_rate(sample_rate);
        
        match player.load_file(file_path) {
            Ok(_) => {
                // Debug: log loaded samples info
                let sample_count = player.samples.len();
                let duration_frames = sample_count / player.channels as usize;
                let duration_sec = duration_frames as f32 / player.sample_rate as f32;
                
                t.set_file_player(player);
                Ok(())
            }
            Err(e) => {
                eprintln!("[Track {}] ERROR loading file: {}", track, e);
                Err(e)
            }
        }
    } else {
        eprintln!("[Track {}] ERROR: track not found (router has {} tracks)", track, router.tracks.len());
        Err(anyhow!("Track {} not found", track))
    }
}

/// Set track EQ parameters (4-band parametric)
pub fn set_eq(
    router: &mut Router,
    track: usize,
    low: f32,
    low_mid: f32,
    high_mid: f32,
    high: f32,
) {
    if let Some(t) = router.get_track_mut(track) {
        t.set_eq(low, low_mid, high_mid, high);
    }
}

/// Enable or disable track EQ
pub fn set_eq_enabled(router: &mut Router, track: usize, enabled: bool) {
    if let Some(t) = router.get_track_mut(track) {
        t.set_eq_enabled(enabled);
    }
}

/// Play file on track
pub fn play_file(router: &mut Router, track: usize, sample_rate: u32) -> Result<()> {
    if let Some(t) = router.get_track_mut(track) {
        t.play_file(sample_rate)
    } else {
        Err(anyhow!("Track {} not found", track))
    }
}

/// Pause file on track
pub fn pause_file(router: &mut Router, track: usize) -> Result<()> {
    if let Some(t) = router.get_track_mut(track) {
        t.pause_file()
    } else {
        Err(anyhow!("Track {} not found", track))
    }
}

/// Stop file on track
pub fn stop_file(router: &mut Router, track: usize) -> Result<()> {
    if let Some(t) = router.get_track_mut(track) {
        t.stop_file()
    } else {
        Err(anyhow!("Track {} not found", track))
    }
}

/// Set track gain (input trim)
pub fn set_gain(router: &mut Router, track: usize, gain: f32) {
    if let Some(t) = router.get_track_mut(track) {
        t.gain = gain.max(0.0); // No upper limit, but can't be negative
        let gain_db = if gain > 0.0 { 20.0 * gain.log10() } else { -90.0 };
    }
}

/// Set track volume (fader)
pub fn set_volume(router: &mut Router, track: usize, volume: f32) {
    if let Some(t) = router.get_track_mut(track) {
        t.volume = volume.max(0.0); // No upper limit, but can't be negative
        let volume_db = if volume > 0.0 { 20.0 * volume.log10() } else { -90.0 };
    }
}

/// Set track mute
pub fn set_mute(router: &mut Router, track: usize, mute: bool) {
    if let Some(t) = router.get_track_mut(track) {
        t.mute = mute;
    }
}

/// Set track routing to master bus
pub fn set_route_to_master(router: &mut Router, track: usize, route: bool) {
    if let Some(t) = router.get_track_mut(track) {
        t.route_to_master = route;
    }
}

/// Set track pan (-1.0 left, 0.0 center, 1.0 right)
pub fn set_pan(router: &mut Router, track: usize, pan: f32) {
    if let Some(t) = router.get_track_mut(track) {
        t.pan = pan.clamp(-1.0, 1.0);
    }
}

/// Set track PAD (-24dB attenuation before gain)
pub fn set_pad(router: &mut Router, track: usize, enabled: bool) {
    if let Some(t) = router.get_track_mut(track) {
        t.pad_enabled = enabled;
        eprintln!("[Track {}] PAD: {}", track, if enabled { "ON (-24dB)" } else { "OFF" });
    }
}

/// Set track HPF (80Hz high-pass filter between PAD and gain)
pub fn set_hpf(router: &mut Router, track: usize, enabled: bool) {
    if let Some(t) = router.get_track_mut(track) {
        t.hpf_enabled = enabled;
        eprintln!("[Track {}] HPF: {}", track, if enabled { "ON (80Hz)" } else { "OFF" });
    }
}
