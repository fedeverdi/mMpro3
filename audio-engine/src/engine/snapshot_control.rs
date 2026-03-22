/// Scene snapshot management - save/load complete engine state
use anyhow::Result;
use std::sync::{Arc, Mutex};
use crate::processing::Router;
use crate::ipc::{Response, messages::{SnapshotInfo, TrackParameters, MasterParameters, SubgroupParameters, AuxParameters, ParametricFilter, AuxSendData}};
use crate::effects::dynamics::compressor::Compressor;
use crate::effects::dynamics::gate::NoiseGate;
use crate::effects::dynamics::deesser::DeEsser;
use crate::effects::spatial::delay::Delay;
use crate::effects::spatial::reverb::Reverb;
use crate::effects::spatial::chorus::Chorus;
use crate::effects::tone::equalizer::FilterType;
use crate::effects::tone::exciter::Exciter;

/// Get snapshots directory path
pub fn get_snapshots_dir() -> std::path::PathBuf {
    // Get user data directory
    let data_dir = if cfg!(target_os = "macos") {
        dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
    } else if cfg!(target_os = "windows") {
        dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
    } else {
        dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
    };
    
    data_dir.join("mMpro3").join("Scenes")
}

/// Create snapshot from current engine state
pub fn create_snapshot(router: &Arc<Mutex<Router>>, name: String) -> Result<crate::engine::snapshot::EngineSnapshot> {
    let router = router.lock().unwrap();
    
    // Determine which engine track indices (0-based) to include in the snapshot.
    // If the frontend has told us which tracks exist via SetActiveTracksInfo, use that
    // list; otherwise fall back to saving every slot (backward-compatibility).
    let track_engine_indices: Vec<usize> = if !router.active_track_ids.is_empty() {
        router.active_track_ids.iter().filter_map(|&id| {
            let engine_idx = id.saturating_sub(1); // 1-based frontend ID → 0-based engine index
            if engine_idx < router.tracks.len() { Some(engine_idx) } else { None }
        }).collect()
    } else {
        (0..router.tracks.len()).collect()
    };

    // Build track_layout from the active track list (preserves 1-based IDs and types)
    let track_layout: Vec<crate::engine::snapshot::TrackLayoutEntry> =
        router.active_track_ids.iter().zip(router.active_track_types.iter())
            .map(|(&id, tp)| crate::engine::snapshot::TrackLayoutEntry {
                id,
                track_type: tp.clone(),
            })
            .collect();
    
    // Snapshot only the chosen tracks
    let tracks: Vec<crate::engine::snapshot::TrackSnapshot> = track_engine_indices.iter().filter_map(|&engine_idx| {
        router.tracks.get(engine_idx).map(|track| {
        // Determine source type and data
        let (source_type, signal_waveform, signal_frequency, file_path, file_artist, file_title, input_device, input_left_channel, input_right_channel) = match &track.source {
            crate::processing::routing::TrackSource::None => ("none".to_string(), None, None, None, None, None, None, None, None),
            crate::processing::routing::TrackSource::SignalGenerator => {
                if let Some(ref generator) = track.signal_generator {
                    ("signal".to_string(), Some(format!("{:?}", generator.waveform)),  Some(generator.frequency), None, None, None, None, None, None)
                } else {
                    ("none".to_string(), None, None, None, None, None, None, None, None)
                }
            },
            crate::processing::routing::TrackSource::FilePlayer => {
                if let Some(ref player) = track.file_player {
                    ("file".to_string(), None, None, 
                     Some(player.file_name.clone()),
                     player.file_artist.clone(),
                     player.file_title.clone(),
                     None, None, None)
                } else {
                    ("none".to_string(), None, None, None, None, None, None, None, None)
                }
            },
            crate::processing::routing::TrackSource::AudioInput => {
                ("input".to_string(), None, None, None, None, None, None, 
                 Some(track.input_channel_selection.left), 
                 Some(track.input_channel_selection.right))
            },
            crate::processing::routing::TrackSource::AuxReturn(_) => {
                // Not commonly saved in snapshots, treat as none for now
                ("none".to_string(), None, None, None, None, None, None, None, None)
           },
        };
        
        // Extract parametric EQ filters
        let parametric_eq_filters: Vec<crate::engine::snapshot::ParametricFilterSnapshot> = 
            track.parametric_eq.bands.iter().map(|f| {
                crate::engine::snapshot::ParametricFilterSnapshot {
                    filter_type: format!("{:?}", f.filter_type),
                    frequency: f.target_frequency,
                    gain: f.target_gain_db,
                    q: f.target_q,
                }
            }).collect();
        
        // Extract compressor if present in inserts
        let compressor = track.inserts.iter().find_map(|slot| {
            if let crate::processing::routing::InsertEffectData::Compressor(ref comp) = slot.effect {
                Some(crate::engine::snapshot::CompressorSnapshot {
                    enabled: slot.enabled,
                    threshold: comp.get_threshold(),
                    ratio: comp.get_ratio(),
                    attack: comp.get_attack(),
                    release: comp.get_release(),
                })
            } else {
                None
            }
        });
        
        // Extract gate if present in inserts
        let gate = track.inserts.iter().find_map(|slot| {
            if let crate::processing::routing::InsertEffectData::Gate(ref g) = slot.effect {
                Some(crate::engine::snapshot::GateSnapshot {
                    enabled: slot.enabled,
                    threshold: g.threshold_db,
                    range: g.range_db,
                    attack: g.attack_ms,
                    release: g.release_ms,
                })
            } else {
                None
            }
        });
        
        // Extract insert effects
        let insert_effects: Vec<crate::engine::snapshot::InsertEffectSnapshot> = 
            track.inserts.iter().enumerate().map(|(pos, slot)| {
                let parameters = match &slot.effect {
                    crate::processing::routing::InsertEffectData::Compressor(comp) => {
                        crate::engine::snapshot::InsertEffectParameters::Compressor {
                            threshold: comp.get_threshold(),
                            ratio: comp.get_ratio(),
                            attack: comp.get_attack(),
                            release: comp.get_release(),
                        }
                    },
                    crate::processing::routing::InsertEffectData::Gate(gate) => {
                        crate::engine::snapshot::InsertEffectParameters::Gate {
                            threshold: gate.threshold_db,
                            range: gate.range_db,
                            attack: gate.attack_ms,
                            release: gate.release_ms,
                        }
                    },
                    crate::processing::routing::InsertEffectData::Reverb(rev) => {
                        crate::engine::snapshot::InsertEffectParameters::Reverb {
                            room_size: rev.get_room_size(),
                            damping: rev.get_damping(),
                            wet: rev.get_wet(),
                            width: rev.get_width(),
                            pre_delay: rev.pre_delay,
                        }
                    },
                    crate::processing::routing::InsertEffectData::Delay(delay) => {
                        crate::engine::snapshot::InsertEffectParameters::Delay {
                            time_l: delay.get_delay_time_l_ms(),
                            time_r: delay.get_delay_time_r_ms(),
                            feedback: delay.get_feedback(),
                            mix: delay.get_mix(),
                        }
                    },
                    crate::processing::routing::InsertEffectData::Exciter(exc) => {
                        crate::engine::snapshot::InsertEffectParameters::Exciter {
                            amount: exc.amount,
                            frequency: exc.frequency,
                            mix: exc.mix,
                        }
                    },
                    crate::processing::routing::InsertEffectData::DeEsser(de) => {
                        crate::engine::snapshot::InsertEffectParameters::DeEsser {
                            threshold: de.threshold,
                            frequency: de.frequency,
                            range: de.range,
                        }
                    },
                    crate::processing::routing::InsertEffectData::Chorus(ch) => {
                        crate::engine::snapshot::InsertEffectParameters::Chorus {
                            rate: ch.rate,
                            depth: ch.depth,
                            mix: ch.mix,
                        }
                    },
                };
                
                crate::engine::snapshot::InsertEffectSnapshot {
                    id: slot.id,
                    effect_type: slot.effect_type.clone(),
                    enabled: slot.enabled,
                    position: pos,
                    parameters,
                }
            }).collect();
        
        // Extract aux sends (all of them, preserving pre_fader and muted state)
        let aux_sends: std::collections::HashMap<usize, crate::engine::snapshot::AuxSendSnapshot> = track.aux_sends.iter()
            .enumerate()
            .map(|(i, send)| (i, crate::engine::snapshot::AuxSendSnapshot {
                level: send.level,
                pre_fader: send.pre_fader,
                muted: send.muted,
            }))
            .collect();
        
        crate::engine::snapshot::TrackSnapshot {
            track_number: track.id,
            source_type,
            signal_waveform,
            signal_frequency,
            file_path,
            file_artist,
            file_title,
            input_device,
            input_left_channel,
            input_right_channel,
            gain: track.gain,
            volume: track.volume,
            pan: track.pan,
            muted: track.mute,
            soloed: track.solo,
            pad_enabled: track.pad_enabled,
            hpf_enabled: track.hpf_enabled,
            phase_inverted: track.phase_inverted,
            pfl_enabled: track.pfl_enabled,
            route_to_master: track.route_to_master,
            routed_subgroups: track.route_to_subgroups.clone(),
            eq_enabled: track.equalizer.enabled,
            eq_low: track.equalizer.low_shelf.target_gain_db,
            eq_low_mid: track.equalizer.low_mid.target_gain_db,
            eq_high_mid: track.equalizer.high_mid.target_gain_db,
            eq_high: track.equalizer.high_shelf.target_gain_db,
            parametric_eq_enabled: track.parametric_eq.enabled,
            parametric_eq_filters,
            compressor,
            gate,
            insert_effects,
            aux_sends,
        }
        })
    }).collect();
    
    // Extract master parametric EQ filters
    let master_parametric_eq_filters: Vec<crate::engine::snapshot::ParametricFilterSnapshot> = 
        router.master.parametric_eq.bands.iter().map(|f| {
            crate::engine::snapshot::ParametricFilterSnapshot {
                filter_type: format!("{:?}", f.filter_type),
                frequency: f.target_frequency,
                gain: f.target_gain_db,
                q: f.target_q,
            }
        }).collect();
    
    eprintln!("[Snapshot] Master parametric EQ: enabled={}, filters={}", 
              router.master.parametric_eq.enabled, 
              master_parametric_eq_filters.len());
    
    // Extract master compressor
    let master_compressor = if router.master.compressor_present {
        Some(crate::engine::snapshot::CompressorSnapshot {
            enabled: router.master.compressor.enabled,
            threshold: router.master.compressor.get_threshold(),
            ratio: router.master.compressor.get_ratio(),
            attack: router.master.compressor.get_attack(),
            release: router.master.compressor.get_release(),
        })
    } else {
        None
    };
    
    // Extract master limiter
    let master_limiter = if router.master.limiter_present {
        Some(crate::engine::snapshot::LimiterSnapshot {
            enabled: router.master.limiter.enabled,
            threshold: router.master.limiter.get_ceiling(),
            release: router.master.limiter.get_release(),
        })
    } else {
        None
    };
    
    // Extract master exciter (from delay - seems there's no separate exciter in MasterBus)
    let master_exciter = None; // TODO: Add if MasterBus gets exciter
    
    // Extract master stereo width
    let master_stereo_width = None; // TODO: Add if MasterBus gets stereo width processor
    
    // Extract subgroups
    let subgroups: Vec<crate::engine::snapshot::SubgroupSnapshot> = router.subgroups.iter().map(|sg| {
        crate::engine::snapshot::SubgroupSnapshot {
            id: sg.id,
            name: format!("Subgroup {}", sg.id + 1), // Simple naming
            volume: sg.gain,
            muted: sg.mute,
            route_to_master: sg.route_to_master,
            output_left_channel: sg.output_channel_selection.left,
            output_right_channel: sg.output_channel_selection.right,
            selected_output_device: sg.selected_output.clone(),
        }
    }).collect();
    
    // Extract aux buses
    let auxes: Vec<crate::engine::snapshot::AuxSnapshot> = router.aux_buses.iter().map(|aux| {
        let reverb = if aux.reverb.enabled {
            Some(crate::engine::snapshot::ReverbSnapshot {
                enabled: true,
                room_size: aux.reverb.get_room_size(),
                damping: aux.reverb.get_damping(),
                wet: aux.reverb.get_wet(),
                width: aux.reverb.get_width(),
                pre_delay: aux.reverb.pre_delay,
            })
        } else {
            None
        };
        
        let delay = if aux.delay.enabled {
            Some(crate::engine::snapshot::DelaySnapshot {
                enabled: true,
                time_l: aux.delay.get_delay_time_l_ms(),
                time_r: aux.delay.get_delay_time_r_ms(),
                feedback: aux.delay.get_feedback(),
                mix: aux.delay.get_mix(),
            })
        } else {
            None
        };
        
        crate::engine::snapshot::AuxSnapshot {
            id: aux.id,
            name: format!("Aux {}", aux.id + 1),
            volume: aux.gain,
            muted: aux.mute,
            route_to_master: aux.route_to_master,
            routed_subgroups: aux.route_to_subgroups.clone(),
            output_left_channel: aux.output_channel_selection.left,
            output_right_channel: aux.output_channel_selection.right,
            selected_output_device: aux.selected_output.clone(),
            reverb,
            delay,
        }
    }).collect();
    
    Ok(crate::engine::snapshot::EngineSnapshot {
        version: 1,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        name,
        pinned: false,
        track_layout,
        tracks,
        master: crate::engine::snapshot::MasterSnapshot {
            gain_left: router.master.gain_left,
            gain_right: router.master.gain_right,
            linked: router.master.linked,
            muted: router.master.mute,
            output_left_channel: router.master.output_channel_selection.left,
            output_right_channel: router.master.output_channel_selection.right,
            selected_output_device: None, // This is managed by AudioEngine, not Router
            parametric_eq_enabled: router.master.parametric_eq.enabled,
            parametric_eq_filters: master_parametric_eq_filters,
            compressor: master_compressor,
            limiter: master_limiter,
            exciter: master_exciter,
            stereo_width: master_stereo_width,
        },
        subgroups,
        auxes,
    })
}

/// Save snapshot to filesystem
pub fn save_snapshot_impl(router: &Arc<Mutex<Router>>, name: &str) -> Result<()> {
    let snapshots_dir = get_snapshots_dir();
    std::fs::create_dir_all(&snapshots_dir)?;
    
    let mut snapshot = create_snapshot(router, name.to_string())?;
    
    // Preserve existing pinned status if the file already exists
    let filepath = snapshots_dir.join(format!("{}.json", name));
    if filepath.exists() {
        if let Ok(existing_json) = std::fs::read_to_string(&filepath) {
            if let Ok(existing) = serde_json::from_str::<crate::engine::snapshot::EngineSnapshot>(&existing_json) {
                snapshot.pinned = existing.pinned;
            }
        }
    }
    
    let json = serde_json::to_string_pretty(&snapshot)?;
    std::fs::write(&filepath, json)?;
    
    eprintln!("[Engine] ✓ Snapshot saved: {}", name);
    Ok(())
}

/// Load snapshot from filesystem and apply to engine.
/// Returns the list of (track_index, file_path, artist, title) that need to be loaded by AudioEngine.
pub fn load_snapshot_impl(router: &Arc<Mutex<Router>>, name: &str) -> Result<Vec<(usize, String, Option<String>, Option<String>)>> {
    let snapshots_dir = get_snapshots_dir();
    let filename = format!("{}.json", name);
    let filepath = snapshots_dir.join(&filename);
    
    if !filepath.exists() {
        return Err(anyhow::anyhow!("Snapshot not found: {}", name));
    }
    
    let json = std::fs::read_to_string(&filepath)?;
    let snapshot: crate::engine::snapshot::EngineSnapshot = serde_json::from_str(&json)?;
    
    // Lock router and apply all snapshot state
    let mut router = router.lock().unwrap();
    
    // Collect file tracks to load after releasing the lock
    let mut files_to_load: Vec<(usize, String, Option<String>, Option<String>)> = Vec::new();

    // Apply tracks
    for track_snap in snapshot.tracks.iter() {
        if let Some(track) = router.tracks.get_mut(track_snap.track_number) {
            // Apply source
            track.source = match track_snap.source_type.as_str() {
                "signal" => crate::processing::routing::TrackSource::SignalGenerator,
                "file" => crate::processing::routing::TrackSource::FilePlayer,
                "input" => crate::processing::routing::TrackSource::AudioInput,
                _ => crate::processing::routing::TrackSource::None,
            };
            
            // Collect file tracks for later loading (can't decode audio while Router is locked)
            if track_snap.source_type == "file" {
                if let Some(ref path) = track_snap.file_path {
                    files_to_load.push((
                        track_snap.track_number,
                        path.clone(),
                        track_snap.file_artist.clone(),
                        track_snap.file_title.clone(),
                    ));
                }
            }
            
            // Apply signal generator settings
            if track_snap.source_type == "signal" {
                if let Some(ref mut generator) = track.signal_generator {
                    if let Some(ref waveform) = track_snap.signal_waveform {
                        generator.set_waveform(crate::processing::signal_gen::WaveformType::from_str(waveform));
                    }
                    if let Some(freq) = track_snap.signal_frequency {
                        generator.set_frequency(freq);
                    }
                }
            }
            
            // Apply file player settings
            if track_snap.source_type == "file" {
                if let Some(ref mut player) = track.file_player {
                    if let Some(ref path) = track_snap.file_path {
                        player.file_name = path.clone();
                    }
                    player.file_artist = track_snap.file_artist.clone();
                    player.file_title = track_snap.file_title.clone();
                }
            }
            
            // Apply input channels
            if track_snap.source_type == "input" {
                if let Some(left) = track_snap.input_left_channel {
                    track.input_channel_selection.left = left;
                }
                if let Some(right) = track_snap.input_right_channel {
                    track.input_channel_selection.right = right;
                }
            }
            
            // Apply basic controls
            track.gain = track_snap.gain;
            track.volume = track_snap.volume;
            track.pan = track_snap.pan;
            track.mute = track_snap.muted;
            track.solo = track_snap.soloed;
            track.pad_enabled = track_snap.pad_enabled;
            track.hpf_enabled = track_snap.hpf_enabled;
            track.phase_inverted = track_snap.phase_inverted;
            track.pfl_enabled = track_snap.pfl_enabled;
            
            // Apply routing
            track.route_to_master = track_snap.route_to_master;
            track.route_to_subgroups = track_snap.routed_subgroups.clone();
            
            // Apply EQ
            track.equalizer.enabled = track_snap.eq_enabled;
            track.equalizer.low_shelf.target_gain_db = track_snap.eq_low;
            track.equalizer.low_mid.target_gain_db = track_snap.eq_low_mid;
            track.equalizer.high_mid.target_gain_db = track_snap.eq_high_mid;
            track.equalizer.high_shelf.target_gain_db = track_snap.eq_high;
            
            // Apply parametric EQ
            track.parametric_eq.enabled = track_snap.parametric_eq_enabled;
            track.parametric_eq.bands.clear();
            for filter_snap in track_snap.parametric_eq_filters.iter() {
                track.parametric_eq.add_band(
                    FilterType::from_str(&filter_snap.filter_type),
                    filter_snap.frequency,
                    filter_snap.gain,
                    filter_snap.q,
                );
            }
            
            // Apply insert effects
            track.inserts.clear();
            for effect_snap in track_snap.insert_effects.iter() {
                let effect_data = match &effect_snap.parameters {
                    crate::engine::snapshot::InsertEffectParameters::Compressor { threshold, ratio, attack, release } => {
                        let mut comp = Compressor::new(48000.0);
                        comp.set_threshold(*threshold);
                        comp.set_ratio(*ratio);
                        comp.set_attack(*attack);
                        comp.set_release(*release);
                        comp.set_enabled(effect_snap.enabled);
                        crate::processing::routing::InsertEffectData::Compressor(comp)
                    },
                    crate::engine::snapshot::InsertEffectParameters::Gate { threshold, range, attack, release } => {
                        let mut gate = NoiseGate::new(48000.0);
                        gate.set_threshold(*threshold);
                        gate.set_range(*range);
                        gate.set_attack(*attack);
                        gate.set_release(*release);
                        gate.set_enabled(effect_snap.enabled);
                        crate::processing::routing::InsertEffectData::Gate(gate)
                    },
                    crate::engine::snapshot::InsertEffectParameters::Reverb { room_size, damping, wet, width, pre_delay } => {
                        let mut rev = Reverb::new(48000.0);
                        rev.set_room_size(*room_size);
                        rev.set_damping(*damping);
                        rev.set_wet(*wet);
                        rev.set_width(*width);
                        rev.set_pre_delay(*pre_delay);
                        crate::processing::routing::InsertEffectData::Reverb(rev)
                    },
                    crate::engine::snapshot::InsertEffectParameters::Delay { time_l, time_r, feedback, mix } => {
                        let mut delay = Delay::new(48000.0);
                        delay.set_delay_time_left(*time_l);
                        delay.set_delay_time_right(*time_r);
                        delay.set_feedback(*feedback);
                        delay.set_mix(*mix);
                        crate::processing::routing::InsertEffectData::Delay(delay)
                    },
                    crate::engine::snapshot::InsertEffectParameters::Exciter { amount, frequency, mix } => {
                        let mut exc = Exciter::new(48000.0);
                        exc.set_amount(*amount);
                        exc.set_frequency(*frequency);
                        exc.set_mix(*mix);
                        crate::processing::routing::InsertEffectData::Exciter(exc)
                    },
                    crate::engine::snapshot::InsertEffectParameters::DeEsser { threshold, frequency, range } => {
                        let mut de = DeEsser::new(48000.0);
                        de.set_threshold(*threshold);
                        de.set_frequency(*frequency);
                        de.set_range(*range);
                        crate::processing::routing::InsertEffectData::DeEsser(de)
                    },
                    crate::engine::snapshot::InsertEffectParameters::Chorus { rate, depth, mix } => {
                        let mut ch = Chorus::new(48000.0);
                        ch.set_rate(*rate);
                        ch.set_depth(*depth);
                        ch.set_mix(*mix);
                        crate::processing::routing::InsertEffectData::Chorus(ch)
                    },
                };
                
                track.inserts.push(crate::processing::routing::InsertSlot {
                    id: effect_snap.id,
                    effect_type: effect_snap.effect_type.clone(),
                    effect: effect_data,
                    enabled: effect_snap.enabled,
                });
            }
            
            // Apply aux sends
            for (aux_index, aux_snap) in track_snap.aux_sends.iter() {
                if let Some(send) = track.aux_sends.get_mut(*aux_index) {
                    send.level = aux_snap.level;
                    send.pre_fader = aux_snap.pre_fader;
                    send.muted = aux_snap.muted;
                }
            }
        }
    }
    
    // Apply master
    router.master.gain_left = snapshot.master.gain_left;
    router.master.gain_right = snapshot.master.gain_right;
    router.master.linked = snapshot.master.linked;
    router.master.mute = snapshot.master.muted;
    router.master.output_channel_selection.left = snapshot.master.output_left_channel;
    router.master.output_channel_selection.right = snapshot.master.output_right_channel;
    
    // Apply master parametric EQ
    router.master.parametric_eq.enabled = snapshot.master.parametric_eq_enabled;
    router.master.parametric_eq.bands.clear();
    for filter_snap in snapshot.master.parametric_eq_filters.iter() {
        router.master.parametric_eq.add_band(
            FilterType::from_str(&filter_snap.filter_type),
            filter_snap.frequency,
            filter_snap.gain,
            filter_snap.q,
        );
    }
    
    // Apply master compressor
    if let Some(ref comp_snap) = snapshot.master.compressor {
        router.master.compressor_present = true;
        router.master.compressor.enabled = comp_snap.enabled;
        router.master.compressor.set_threshold(comp_snap.threshold);
        router.master.compressor.set_ratio(comp_snap.ratio);
        router.master.compressor.set_attack(comp_snap.attack);
        router.master.compressor.set_release(comp_snap.release);
    } else {
        router.master.compressor_present = false;
    }
    
    // Apply master limiter
    if let Some(ref lim_snap) = snapshot.master.limiter {
        router.master.limiter_present = true;
        router.master.limiter.enabled = lim_snap.enabled;
        router.master.limiter.set_ceiling(lim_snap.threshold);
        router.master.limiter.set_release(lim_snap.release);
    } else {
        router.master.limiter_present = false;
    }
    
    // Apply subgroups
    for sg_snap in snapshot.subgroups.iter() {
        if let Some(subgroup) = router.subgroups.get_mut(sg_snap.id) {
            subgroup.gain = sg_snap.volume;
            subgroup.mute = sg_snap.muted;
            subgroup.route_to_master = sg_snap.route_to_master;
            subgroup.output_channel_selection.left = sg_snap.output_left_channel;
            subgroup.output_channel_selection.right = sg_snap.output_right_channel;
            if let Some(ref device) = sg_snap.selected_output_device {
                subgroup.selected_output = Some(device.clone());
            }
        }
    }
    
    // Apply aux buses
    for aux_snap in snapshot.auxes.iter() {
        if let Some(aux) = router.aux_buses.get_mut(aux_snap.id) {
            aux.gain = aux_snap.volume;
            aux.mute = aux_snap.muted;
            aux.route_to_master = aux_snap.route_to_master;
            aux.route_to_subgroups = aux_snap.routed_subgroups.clone();
            aux.output_channel_selection.left = aux_snap.output_left_channel;
            aux.output_channel_selection.right = aux_snap.output_right_channel;
            if let Some(ref device) = aux_snap.selected_output_device {
                aux.selected_output = Some(device.clone());
            }
            
            // Apply reverb
            if let Some(ref rev_snap) = aux_snap.reverb {
                aux.reverb.enabled = rev_snap.enabled;
                aux.reverb.set_room_size(rev_snap.room_size);
                aux.reverb.set_damping(rev_snap.damping);
                aux.reverb.set_wet(rev_snap.wet);
                aux.reverb.set_width(rev_snap.width);
                aux.reverb.set_pre_delay(rev_snap.pre_delay);
            } else {
                aux.reverb.enabled = false;
            }
            
            // Apply delay
            if let Some(ref delay_snap) = aux_snap.delay {
                aux.delay.enabled = delay_snap.enabled;
                aux.delay.set_delay_time_left(delay_snap.time_l);
                aux.delay.set_delay_time_right(delay_snap.time_r);
                aux.delay.set_feedback(delay_snap.feedback);
                aux.delay.set_mix(delay_snap.mix);
            } else {
                aux.delay.enabled = false;
            }
        }
    }
    
    // Restore active track layout so the next save reflects the loaded scene's track list
    if !snapshot.track_layout.is_empty() {
        router.active_track_ids = snapshot.track_layout.iter().map(|e| e.id).collect();
        router.active_track_types = snapshot.track_layout.iter().map(|e| e.track_type.clone()).collect();
    } else {
        // Old snapshot without track_layout: infer from saved tracks
        router.active_track_ids = snapshot.tracks.iter().map(|t| t.track_number + 1).collect();
        router.active_track_types = snapshot.tracks.iter()
            .map(|t| if t.source_type == "signal" { "signal".to_string() } else { "audio".to_string() })
            .collect();
    }

    Ok(files_to_load)
}

/// Build a full ParametersChanged response from current router state (for post-load broadcast)
pub fn build_full_parameters_changed(router: &Arc<Mutex<Router>>) -> Response {
    let router = router.lock().unwrap();

    // Build track parameters
    let tracks: Vec<TrackParameters> = router.tracks.iter().enumerate().map(|(i, track)| {
        let eq_filters: Vec<ParametricFilter> = track.parametric_eq.bands.iter().map(|f| {
            ParametricFilter {
                filter_type: format!("{:?}", f.filter_type).to_lowercase(),
                frequency: f.target_frequency,
                gain: f.target_gain_db,
                q: f.target_q,
            }
        }).collect();

        let aux_sends: Vec<AuxSendData> = track.aux_sends.iter().map(|s| {
            AuxSendData {
                level: s.level,
                pre_fader: s.pre_fader,
                muted: s.muted,
            }
        }).collect();

        // Build insert effects list
        let inserts: Vec<crate::ipc::messages::InsertEffectInfo> = track.inserts.iter().map(|slot| {
            let parameters = match &slot.effect {
                crate::processing::routing::InsertEffectData::Compressor(comp) => Some(serde_json::json!({
                    "threshold": comp.get_threshold(),
                    "ratio": comp.get_ratio(),
                    "attack": comp.get_attack(),
                    "release": comp.get_release(),
                })),
                crate::processing::routing::InsertEffectData::Gate(gate) => Some(serde_json::json!({
                    "threshold": gate.threshold_db,
                    "range": gate.range_db,
                    "attack": gate.attack_ms,
                    "release": gate.release_ms,
                })),
                _ => None,
            };
            crate::ipc::messages::InsertEffectInfo {
                id: slot.id,
                effect_type: slot.effect_type.clone(),
                enabled: slot.enabled,
                parameters,
            }
        }).collect();

        // Extract compressor parameters from inserts (first compressor found)
        let (comp_enabled, comp_threshold, comp_ratio, comp_attack, comp_release) =
            track.inserts.iter().find_map(|slot| {
                if let crate::processing::routing::InsertEffectData::Compressor(ref comp) = slot.effect {
                    Some((
                        Some(slot.enabled),
                        Some(comp.get_threshold()),
                        Some(comp.get_ratio()),
                        Some(comp.get_attack()),
                        Some(comp.get_release()),
                    ))
                } else { None }
            }).unwrap_or((None, None, None, None, None));

        // Extract gate parameters from inserts (first gate found)
        let (gate_enabled, gate_threshold, gate_range, gate_attack, gate_release) =
            track.inserts.iter().find_map(|slot| {
                if let crate::processing::routing::InsertEffectData::Gate(ref g) = slot.effect {
                    Some((
                        Some(slot.enabled),
                        Some(g.threshold_db),
                        Some(g.range_db),
                        Some(g.attack_ms),
                        Some(g.release_ms),
                    ))
                } else { None }
            }).unwrap_or((None, None, None, None, None));

        // Extract file info
        let (file_name, file_artist, file_title, is_stereo) =
            if let Some(ref player) = track.file_player {
                (
                    Some(player.file_name.clone()),
                    player.file_artist.clone(),
                    player.file_title.clone(),
                    Some(player.channels >= 2),
                )
            } else {
                (None, None, None, None)
            };

        TrackParameters {
            track: i,
            gain: Some(track.gain),
            volume: Some(track.volume),
            mute: Some(track.mute),
            solo: Some(track.solo),
            pan: Some(track.pan),
            route_to_master: Some(track.route_to_master),
            route_to_subgroups: Some(track.route_to_subgroups.clone()),
            pad_enabled: Some(track.pad_enabled),
            hpf_enabled: Some(track.hpf_enabled),
            phase_inverted: Some(track.phase_inverted),
            pfl_enabled: Some(track.pfl_enabled),
            compressor_enabled: comp_enabled,
            compressor_threshold_db: comp_threshold,
            compressor_ratio: comp_ratio,
            compressor_attack_ms: comp_attack,
            compressor_release_ms: comp_release,
            gate_enabled,
            gate_threshold_db: gate_threshold,
            gate_range_db: gate_range,
            gate_attack_ms: gate_attack,
            gate_release_ms: gate_release,
            eq_enabled: Some(track.equalizer.enabled),
            eq_low: Some(track.equalizer.low_shelf.target_gain_db),
            eq_low_mid: Some(track.equalizer.low_mid.target_gain_db),
            eq_high_mid: Some(track.equalizer.high_mid.target_gain_db),
            eq_high: Some(track.equalizer.high_shelf.target_gain_db),
            parametric_eq_enabled: Some(track.parametric_eq.enabled),
            eq_filters: Some(eq_filters),
            aux_sends: Some(aux_sends),
            file_name,
            file_artist,
            file_title,
            is_stereo,
            fft_data: None,
            inserts: Some(inserts),
        }
    }).collect();

    // Build master parameters
    let master_eq_filters: Vec<ParametricFilter> = router.master.parametric_eq.bands.iter().map(|f| {
        ParametricFilter {
            filter_type: format!("{:?}", f.filter_type).to_lowercase(),
            frequency: f.target_frequency,
            gain: f.target_gain_db,
            q: f.target_q,
        }
    }).collect();

    let master = MasterParameters {
        gain: None,
        gain_left: Some(router.master.gain_left),
        gain_right: Some(router.master.gain_right),
        mute: Some(router.master.mute),
        linked: Some(router.master.linked),
        eq_filters: Some(master_eq_filters),
        current_eq_preset: None,
        fx_effects: None,
        selected_output: None,
        available_output_devices: None,
    };

    // Build subgroup parameters
    let subgroups: Vec<SubgroupParameters> = router.subgroups.iter().map(|sg| {
        SubgroupParameters {
            subgroup: sg.id,
            gain: Some(sg.gain),
            mute: Some(sg.mute),
            route_to_master: Some(sg.route_to_master),
            selected_output: None,
        }
    }).collect();

    // Build aux parameters
    let auxes: Vec<AuxParameters> = router.aux_buses.iter().map(|aux| {
        AuxParameters {
            aux: aux.id,
            gain: Some(aux.gain),
            mute: Some(aux.mute),
            route_to_master: Some(aux.route_to_master),
            route_to_subgroups: None,
            output_enabled: None,
            output_channel_selection_left: None,
            output_channel_selection_right: None,
            selected_output: None,
            reverb: None,
            delay: None,
        }
    }).collect();

    Response::ParametersChanged {
        tracks: Some(tracks),
        master: Some(master),
        subgroups: Some(subgroups),
        auxes: Some(auxes),
    }
}

/// List all available snapshots
pub fn list_snapshots_impl() -> Option<Response> {
    let snapshots_dir = get_snapshots_dir();
    
    if !snapshots_dir.exists() {
        std::fs::create_dir_all(&snapshots_dir).ok()?;
    }
    
    let mut snapshots = Vec::new();
    
    for entry in std::fs::read_dir(&snapshots_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    // Read pinned field and track layout from the snapshot JSON
                    let snapshot_opt = std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|json| serde_json::from_str::<crate::engine::snapshot::EngineSnapshot>(&json).ok());
                    let pinned = snapshot_opt.as_ref().map(|s| s.pinned).unwrap_or(false);
                    // Derive track_ids / track_types from track_layout (new) or raw tracks (old snapshots)
                    let (track_ids, track_types) = if let Some(ref snap) = snapshot_opt {
                        if !snap.track_layout.is_empty() {
                            (
                                snap.track_layout.iter().map(|e| e.id).collect::<Vec<_>>(),
                                snap.track_layout.iter().map(|e| e.track_type.clone()).collect::<Vec<_>>(),
                            )
                        } else {
                            // Old snapshot without track_layout: infer from saved tracks
                            let ids: Vec<usize> = snap.tracks.iter().map(|t| t.track_number + 1).collect();
                            let types: Vec<String> = snap.tracks.iter()
                                .map(|t| if t.source_type == "signal" { "signal".to_string() } else { "audio".to_string() })
                                .collect();
                            (ids, types)
                        }
                    } else {
                        (Vec::new(), Vec::new())
                    };
                    snapshots.push(SnapshotInfo {
                        name: name.to_string(),
                        timestamp: metadata.modified()
                            .unwrap()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        size_bytes: metadata.len(),
                        pinned,
                        track_count: track_ids.len(),
                        track_ids,
                        track_types,
                    });
                }
            }
        }
    }
    
    // Sort by timestamp (newest first)
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Some(Response::SnapshotsList { snapshots })
}

/// Get snapshot data
pub fn get_snapshot_impl(name: &str) -> Option<Response> {
    let snapshots_dir = get_snapshots_dir();
    let filename = format!("{}.json", name);
    let filepath = snapshots_dir.join(&filename);
    
    if !filepath.exists() {
        eprintln!("[Engine] ✗ Snapshot not found: {}", name);
        return None;
    }
    
    let json = std::fs::read_to_string(&filepath).ok()?;
    let snapshot: crate::engine::snapshot::EngineSnapshot = serde_json::from_str(&json).ok()?;
    
    Some(Response::SnapshotData { snapshot })
}

/// Delete snapshot
pub fn delete_snapshot_impl(name: &str) -> Result<()> {
    let snapshots_dir = get_snapshots_dir();
    let filename = format!("{}.json", name);
    let filepath = snapshots_dir.join(&filename);
    
    if !filepath.exists() {
        return Err(anyhow::anyhow!("Snapshot not found: {}", name));
    }
    
    std::fs::remove_file(&filepath)?;
    
    eprintln!("[Engine] ✓ Snapshot deleted: {}", name);
    Ok(())
}

/// Rename snapshot
pub fn rename_snapshot_impl(old_name: &str, new_name: &str) -> Result<()> {
    let snapshots_dir = get_snapshots_dir();
    let old_filename = format!("{}.json", old_name);
    let new_filename = format!("{}.json", new_name);
    let old_filepath = snapshots_dir.join(&old_filename);
    let new_filepath = snapshots_dir.join(&new_filename);
    
    if !old_filepath.exists() {
        return Err(anyhow::anyhow!("Snapshot not found: {}", old_name));
    }
    
    if new_filepath.exists() {
        return Err(anyhow::anyhow!("Snapshot already exists: {}", new_name));
    }
    
    // Read, update name, write to new file
    let json = std::fs::read_to_string(&old_filepath)?;
    let mut snapshot: crate::engine::snapshot::EngineSnapshot = serde_json::from_str(&json)?;
    snapshot.name = new_name.to_string();
    
    let json = serde_json::to_string_pretty(&snapshot)?;
    std::fs::write(&new_filepath, json)?;
    std::fs::remove_file(&old_filepath)?;
    
    eprintln!("[Engine] ✓ Snapshot renamed: {} -> {}", old_name, new_name);
    Ok(())
}

/// Toggle pinned status on a snapshot (reads JSON, flips pinned, saves back)
pub fn pin_snapshot_impl(name: &str) -> Result<bool> {
    let snapshots_dir = get_snapshots_dir();
    let filepath = snapshots_dir.join(format!("{}.json", name));

    if !filepath.exists() {
        return Err(anyhow::anyhow!("Snapshot not found: {}", name));
    }

    let json = std::fs::read_to_string(&filepath)?;
    let mut snapshot: crate::engine::snapshot::EngineSnapshot = serde_json::from_str(&json)?;
    snapshot.pinned = !snapshot.pinned;
    let new_pinned = snapshot.pinned;

    let json = serde_json::to_string_pretty(&snapshot)?;
    std::fs::write(&filepath, json)?;

    eprintln!("[Engine] ✓ Snapshot {} pinned={}", name, new_pinned);
    Ok(new_pinned)
}
