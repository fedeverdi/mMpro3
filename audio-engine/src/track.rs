/// Track source management utilities
use anyhow::{anyhow, Result};
use crate::audio_io::ChannelSelection;
use crate::file_player::AudioFilePlayer;
use crate::routing::Router;
use crate::signal_gen::WaveformType;

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
        eprintln!("[Track {}] Source: Signal ({:?} @ {}Hz)", track, wave, frequency);
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
    eprintln!("[Track {}] Attempting to load file: {}", track, file_path);
    eprintln!("[Track {}] Router has {} tracks", track, router.tracks.len());
    
    if let Some(t) = router.get_track_mut(track) {
        eprintln!("[Track {}] Track found in router", track);
        let mut player = AudioFilePlayer::new();
        player.set_output_sample_rate(sample_rate);
        eprintln!("[Track {}] Loading file with output sample rate: {}", track, sample_rate);
        
        match player.load_file(file_path) {
            Ok(_) => {
                // Debug: log loaded samples info
                let sample_count = player.samples.len();
                let duration_frames = sample_count / player.channels as usize;
                let duration_sec = duration_frames as f32 / player.sample_rate as f32;
                eprintln!("[Track {}] File loaded: {} samples, {} channels, {} Hz, {:.2}s duration", 
                    track, sample_count, player.channels, player.sample_rate, duration_sec);
                
                t.set_file_player(player);
                eprintln!("[Track {}] Source: File ({})", track, file_path);
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
