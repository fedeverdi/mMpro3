use crate::engine::AudioEngine;
use crate::ipc::{Command, Response, TrackParameters};

impl AudioEngine {
    pub fn handle_track_source_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::SetTrackSourceInput {
                track,
                left_channel,
                right_channel,
                device_name,
            } => {
                let _ = self.set_track_source_input(track, left_channel, right_channel, device_name.clone());
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        solo: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        pfl_enabled: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: Some(format!("Audio Input ({})", device_name.unwrap_or_else(|| "Default".to_string()))),
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                        inserts: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackSourceSignal {
                track,
                waveform,
                frequency,
            } => {
                let _ = self.set_track_source_signal(track, &waveform, frequency);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        solo: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        pfl_enabled: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: Some(format!("Signal Generator ({} @ {} Hz)", waveform, frequency)),
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                        inserts: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSignalFrequency { track, frequency } => {
                // Fire-and-forget command, no response needed
                let _ = self.set_signal_frequency(track, frequency);
                None
            }
            Command::SetSignalWaveform { track, waveform } => {
                // Fire-and-forget command, no response needed
                let _ = self.set_signal_waveform(track, &waveform);
                None
            }
            Command::ClearTrackSource { track } => {
                let _ = self.clear_track_source(track);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        solo: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        pfl_enabled: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: Some("".to_string()),
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                        inserts: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackSourceFile { track, file_path, artist, title, playlist_id, playlist_name, playlist_index } => {
                match self.set_track_source_file(track, &file_path, artist.as_deref(), title.as_deref(), playlist_id.as_deref(), playlist_name.as_deref(), playlist_index) {
                    Ok(_) => {
                        // Read track parameters after file is loaded
                        let mut router = self.router.lock().unwrap();
                        let (gain, is_stereo) = if let Some(t) = router.get_track_mut(track) {
                            let stereo = if let Some(ref player) = t.file_player {
                                player.channels >= 2
                            } else {
                                false
                            };
                            (Some(t.gain), Some(stereo))
                        } else {
                            (None, None)
                        };
                        drop(router);
                        
                        Some(Response::ParametersChanged {
                            tracks: Some(vec![TrackParameters {
                                track,
                                gain,
                                volume: None,
                                mute: None,
                                solo: None,
                                pan: None,
                                route_to_master: None,
                                route_to_subgroups: None,
                                pad_enabled: None,
                                hpf_enabled: None,
                                phase_inverted: None,
                                pfl_enabled: None,
                                compressor_enabled: None,
                                compressor_threshold_db: None,
                                compressor_ratio: None,
                                compressor_attack_ms: None,
                                compressor_release_ms: None,
                                gate_enabled: None,
                                gate_threshold_db: None,
                                gate_range_db: None,
                                gate_attack_ms: None,
                                gate_release_ms: None,
                                eq_enabled: None,
                                eq_low: None,
                                eq_low_mid: None,
                                eq_high_mid: None,
                                eq_high: None,
                                parametric_eq_enabled: None,
                                eq_filters: None,
                                aux_sends: None,
                                file_name: Some(file_path.clone()),
                                file_artist: artist.clone(),
                                file_title: title.clone(),
                                is_stereo,
                                fft_data: None,
                                inserts: None,
                            }]),
                            subgroups: None,
                            auxes: None,
                            master: None,
                        })
                    }
                    Err(e) => {
                        eprintln!("[Engine] SetTrackSourceFile FAILED for track {}: {}", track, e);
                        None
                    }
                }
            }
            Command::PlayFile { track, file_path, artist, title } => {
                match self.play_file(track, file_path.as_deref(), artist.as_deref(), title.as_deref()) {
                    Ok(_) => {
                        // Read track parameters after file is loaded
                        let mut router = self.router.lock().unwrap();
                        let (gain, is_stereo, file_name, file_artist, file_title) = if let Some(t) = router.get_track_mut(track) {
                            let stereo = if let Some(ref player) = t.file_player {
                                player.channels >= 2
                            } else {
                                false
                            };
                            
                            // If file_path is provided, use it and the provided metadata
                            // Otherwise, read metadata from existing file_player
                            let (fname, fartist, ftitle) = if file_path.is_some() {
                                // Extract filename from provided path
                                let fname = if let Some(ref path) = file_path {
                                    std::path::Path::new(path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| path.clone())
                                } else {
                                    String::new()
                                };
                                
                                // If artist/title not provided but file_player exists, preserve existing metadata
                                let (final_artist, final_title) = if artist.is_none() && title.is_none() {
                                    if let Some(ref player) = t.file_player {
                                        (player.file_artist.clone(), player.file_title.clone())
                                    } else {
                                        (None, None)
                                    }
                                } else {
                                    (artist, title)
                                };
                                
                                (Some(fname), final_artist, final_title)
                            } else {
                                // Read from existing file_player
                                if let Some(ref player) = t.file_player {
                                    (
                                        Some(player.file_name.clone()),
                                        player.file_artist.clone(),
                                        player.file_title.clone()
                                    )
                                } else {
                                    (Some(String::new()), None, None)
                                }
                            };
                            
                            (Some(t.gain), Some(stereo), fname, fartist, ftitle)
                        } else {
                            (None, None, Some(String::new()), None, None)
                        };
                        drop(router);
                        
                        Some(Response::ParametersChanged {
                            tracks: Some(vec![TrackParameters {
                                track,
                                gain,
                                volume: None,
                                mute: None,
                                solo: None,
                                pan: None,
                                route_to_master: None,
                                route_to_subgroups: None,
                                pad_enabled: None,
                                hpf_enabled: None,
                                phase_inverted: None,
                                pfl_enabled: None,
                                compressor_enabled: None,
                                compressor_threshold_db: None,
                                compressor_ratio: None,
                                compressor_attack_ms: None,
                                compressor_release_ms: None,
                                gate_enabled: None,
                                gate_threshold_db: None,
                                gate_range_db: None,
                                gate_attack_ms: None,
                                gate_release_ms: None,
                                eq_enabled: None,
                                eq_low: None,
                                eq_low_mid: None,
                                eq_high_mid: None,
                                eq_high: None,
                                parametric_eq_enabled: None,
                                eq_filters: None,
                                aux_sends: None,
                                file_name,
                                file_artist,
                                file_title,
                                is_stereo,
                                fft_data: None,
                                inserts: None,
                            }]),
                            subgroups: None,
                            auxes: None,
                            master: None,
                        })
                    }
                    Err(e) => {
                        eprintln!("[Engine] PlayFile FAILED for track {}: {}", track, e);
                        None
                    }
                }
            }
            Command::SetTrackSourceAuxReturn { track, aux } => {
                match self.set_track_source_aux_return(track, aux) {
                    Ok(_) => None,
                    Err(e) => Some(Response::Error {
                        message: e.to_string(),
                    }),
                }
            }
            _ => None,
        }
    }
}
