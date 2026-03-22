/// Track control helper functions for AudioEngine
/// These functions provide the implementation for track management operations

use anyhow::Result;
use crate::processing::{Router, WaveformType, InsertEffectData};
use crate::ipc::ParametricFilter;
use crate::processing::track;
use crate::effects::tone::equalizer;
use std::sync::{Arc, Mutex};

// Helper functions that work with Router directly

pub fn set_signal_frequency_impl(router: &Arc<Mutex<Router>>, track: usize, frequency: f32) -> Result<()> {
    let mut router = router.lock().unwrap();
    if let Some(t) = router.get_track_mut(track) {
        if let Some(ref mut generator) = t.signal_generator {
            generator.set_frequency(frequency);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no signal generator", track))
        }
    } else {
        Err(anyhow::anyhow!("Track {} not found", track))
    }
}

pub fn set_signal_waveform_impl(router: &Arc<Mutex<Router>>, track: usize, waveform: &str) -> Result<()> {
    let mut router = router.lock().unwrap();
    if let Some(t) = router.get_track_mut(track) {
        if let Some(ref mut generator) = t.signal_generator {
            let wave = match waveform.to_lowercase().as_str() {
                "sine" => WaveformType::Sine,
                "square" => WaveformType::Square,
                "sawtooth" => WaveformType::Sawtooth,
                "triangle" => WaveformType::Triangle,
                "whitenoise" => WaveformType::WhiteNoise,
                "pinknoise" => WaveformType::PinkNoise,
                _ => return Err(anyhow::anyhow!("Unknown waveform type: {}", waveform)),
            };
            generator.set_waveform(wave);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} has no signal generator", track))
        }
    } else {
        Err(anyhow::anyhow!("Track {} not found", track))
    }
}

pub fn get_waveform_data_impl(router: &Arc<Mutex<Router>>, track: usize, num_points: usize) -> Result<(Vec<f32>, f32, u32)> {
    let router = router.lock().unwrap();
    let data = track::get_waveform_data(&router, track, num_points)?;
    
    if let Some(t) = router.get_track(track) {
        if let Some(player) = &t.file_player {
            let duration = player.get_duration();
            let sample_rate = player.sample_rate;
            return Ok((data, duration, sample_rate));
        }
    }
    
    Err(anyhow::anyhow!("Track {} has no file loaded", track))
}

pub fn set_parametric_eq_filters_impl(router: &Arc<Mutex<Router>>, track: usize, filters: &[ParametricFilter]) {
    use equalizer::FilterType;
    
    let mut router = router.lock().unwrap();
    if let Some(t) = router.tracks.get_mut(track) {
        t.parametric_eq.clear();
        
        for filter in filters {
            let filter_type = match filter.filter_type.as_str() {
                "lowshelf" => FilterType::LowShelf,
                "highshelf" => FilterType::HighShelf,
                "peaking" => FilterType::Peaking,
                "lowpass" => FilterType::LowPass,
                "highpass" => FilterType::HighPass,
                _ => {
                    eprintln!("[Track {}] Unknown filter type: {}", track, filter.filter_type);
                    continue;
                }
            };
            
            t.parametric_eq.add_band(filter_type, filter.frequency, filter.gain, filter.q);
        }
    } else {
        eprintln!("[Engine] Invalid track number: {}", track);
    }
}

pub fn set_parametric_eq_enabled_impl(router: &Arc<Mutex<Router>>, track: usize, enabled: bool) {
    let mut router = router.lock().unwrap();
    if let Some(t) = router.tracks.get_mut(track) {
        t.parametric_eq.set_enabled(enabled);
    } else {
        eprintln!("[Engine] Invalid track number: {}", track);
    }
}

pub fn clear_parametric_eq_impl(router: &Arc<Mutex<Router>>, track: usize) {
    let mut router = router.lock().unwrap();
    if let Some(t) = router.tracks.get_mut(track) {
        t.parametric_eq.clear();
    } else {
        eprintln!("[Engine] Invalid track number: {}", track);
    }
}

pub fn set_track_route_to_subgroup_impl(router: &Arc<Mutex<Router>>, track: usize, subgroup: usize, route: bool) {
    let mut router = router.lock().unwrap();
    if let Some(t) = router.get_track_mut(track) {
        if route {
            // Add subgroup to routing if not already present
            if !t.route_to_subgroups.contains(&subgroup) {
                t.route_to_subgroups.push(subgroup);
            }
        } else {
            // Remove subgroup from routing
            t.route_to_subgroups.retain(|&sg| sg != subgroup);
        }
    }
}

pub fn set_track_aux_send_impl(router: &Arc<Mutex<Router>>, track: usize, aux: usize, level: f32, pre_fader: bool, muted: bool) {
    let mut router = router.lock().unwrap();
    if let Some(t) = router.get_track_mut(track) {
        if aux < t.aux_sends.len() {
            t.aux_sends[aux].level = level.max(0.0);
            t.aux_sends[aux].pre_fader = pre_fader;
            t.aux_sends[aux].muted = muted;
        }
    }
}

/// Set track source to aux return
pub fn set_track_source_aux_return_impl(router: &Arc<Mutex<Router>>, track: usize, aux: usize) -> Result<()> {
    use crate::processing::routing::TrackSource;
    
    let mut router = router.lock().unwrap();
    if let Some(t) = router.get_track_mut(track) {
        t.source = TrackSource::AuxReturn(aux);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Track {} not found", track))
    }
}

/// Assign a pre-loaded file player to a track
pub fn set_track_source_file_impl(
    router: &Arc<Mutex<Router>>,
    track: usize,
    player: crate::processing::file_player::AudioFilePlayer,
    playlist_id: Option<String>,
    playlist_name: Option<String>,
    playlist_index: Option<usize>,
) -> Result<()> {
    use crate::processing::routing::TrackSource;
    
    let mut router = router.lock().unwrap();
    if let Some(t) = router.get_track_mut(track) {
        t.set_file_player(player);
        t.source = TrackSource::FilePlayer;
        
        // Set playlist state
        t.playlist_id = playlist_id;
        t.playlist_name = playlist_name;
        t.playlist_current_index = playlist_index;
        
        println!("[Engine] Track {} loaded file from playlist: id={:?}, name={:?}, index={:?}", 
            track, t.playlist_id, t.playlist_name, t.playlist_current_index);
        
        Ok(())
    } else {
        Err(anyhow::anyhow!("Track {} not found", track))
    }
}

