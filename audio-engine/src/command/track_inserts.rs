use crate::engine::AudioEngine;
use crate::ipc::{Command, Response, TrackParameters, InsertEffectInfo};
use crate::processing::{InsertEffectData, routing};

impl AudioEngine {
    pub fn handle_track_inserts_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::AddTrackInsert { track, effect_type, position } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    // Generate unique ID for this insert
                    let insert_id = t.inserts.iter().map(|s| s.id).max().unwrap_or(0) + 1;
                    
                    // Create insert based on effect type
                    use routing::InsertSlot;
                    let sample_rate = 48000.0; // TODO: Get from actual sample rate
                    let new_insert = match effect_type.as_str() {
                        "gate" => InsertSlot::new_gate(insert_id, sample_rate),
                        "compressor" => InsertSlot::new_compressor(insert_id, sample_rate),
                        "reverb" => InsertSlot::new_reverb(insert_id, sample_rate),
                        "delay" => InsertSlot::new_delay(insert_id, sample_rate),
                        "exciter" => InsertSlot::new_exciter(insert_id, sample_rate),
                        "deesser" => InsertSlot::new_deesser(insert_id, sample_rate),
                        "chorus" => InsertSlot::new_chorus(insert_id, sample_rate),
                        _ => {
                            drop(router);
                            return Some(Response::Error {
                                message: format!("Unknown effect type: {}", effect_type),
                            });
                        }
                    };
                    
                    // Insert at position or append
                    if let Some(pos) = position {
                        if pos <= t.inserts.len() {
                            t.inserts.insert(pos, new_insert);
                        } else {
                            t.inserts.push(new_insert);
                        }
                    } else {
                        t.inserts.push(new_insert);
                    }
                    
                    // Build insert list for response
                    let inserts: Vec<InsertEffectInfo> = t.inserts.iter().map(|slot| {
                        InsertEffectInfo {
                            id: slot.id,
                            effect_type: slot.effect_type.clone(),
                            enabled: slot.enabled,
                        }
                    }).collect();
                    
                    drop(router);
                    
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
                            file_name: None,
                            file_artist: None,
                            file_title: None,
                            is_stereo: None,
                            fft_data: None,
                            inserts: Some(inserts),
                        }]),
                        subgroups: None,
                        auxes: None,
                        master: None,
                    })
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::RemoveTrackInsert { track, insert_id } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    // Find and remove insert by ID
                    if let Some(pos) = t.inserts.iter().position(|slot| slot.id == insert_id) {
                        t.inserts.remove(pos);
                        
                        // Build insert list for response
                        let inserts: Vec<InsertEffectInfo> = t.inserts.iter().map(|slot| {
                            InsertEffectInfo {
                                id: slot.id,
                                effect_type: slot.effect_type.clone(),
                                enabled: slot.enabled,
                            }
                        }).collect();
                        
                        drop(router);
                        
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
                                file_name: None,
                                file_artist: None,
                                file_title: None,
                                is_stereo: None,
                                fft_data: None,
                                inserts: Some(inserts),
                            }]),
                            subgroups: None,
                            auxes: None,
                            master: None,
                        })
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::MoveTrackInsert { track, insert_id, new_position } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    // Find insert by ID
                    if let Some(old_pos) = t.inserts.iter().position(|slot| slot.id == insert_id) {
                        if new_position < t.inserts.len() {
                            let insert = t.inserts.remove(old_pos);
                            t.inserts.insert(new_position, insert);
                            
                            // Build insert list for response
                            let inserts: Vec<InsertEffectInfo> = t.inserts.iter().map(|slot| {
                                InsertEffectInfo {
                                    id: slot.id,
                                    effect_type: slot.effect_type.clone(),
                                    enabled: slot.enabled,
                                }
                            }).collect();
                            
                            drop(router);
                            
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
                                    file_name: None,
                                    file_artist: None,
                                    file_title: None,
                                    is_stereo: None,
                                    fft_data: None,
                                    inserts: Some(inserts),
                                }]),
                                subgroups: None,
                                auxes: None,
                                master: None,
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Invalid position: {}", new_position),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertEnabled { track, insert_id, enabled } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        slot.enabled = enabled;
                        
                        // Also set the enabled state inside the effect itself
                        match &mut slot.effect {
                            InsertEffectData::Gate(gate) => gate.set_enabled(enabled),
                            InsertEffectData::Compressor(comp) => comp.set_enabled(enabled),
                            InsertEffectData::Reverb(rev) => rev.set_enabled(enabled),
                            InsertEffectData::Delay(delay) => delay.set_enabled(enabled),
                            InsertEffectData::Exciter(exciter) => exciter.set_enabled(enabled),
                            InsertEffectData::DeEsser(deesser) => deesser.set_enabled(enabled),
                            InsertEffectData::Chorus(chorus) => chorus.set_enabled(enabled),
                        }
                        
                        let inserts: Vec<InsertEffectInfo> = t.inserts.iter().map(|slot| {
                            InsertEffectInfo {
                                id: slot.id,
                                effect_type: slot.effect_type.clone(),
                                enabled: slot.enabled,
                            }
                        }).collect();
                        
                        drop(router);
                        
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
                                file_name: None,
                                file_artist: None,
                                file_title: None,
                                is_stereo: None,
                                fft_data: None,
                                inserts: Some(inserts),
                            }]),
                            subgroups: None,
                            auxes: None,
                            master: None,
                        })
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertCompressor { track, insert_id, threshold, ratio, attack, release } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        if let InsertEffectData::Compressor(comp) = &mut slot.effect {
                            comp.set_threshold(threshold);
                            comp.set_ratio(ratio);
                            comp.set_attack(attack);
                            comp.set_release(release);
                            
                            drop(router);
                            Some(Response::Ok {
                                message: format!("Compressor {} updated on track {}", insert_id, track),
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Insert {} is not a compressor", insert_id),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertGate { track, insert_id, threshold, range, attack, release } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        if let InsertEffectData::Gate(gate) = &mut slot.effect {
                            gate.set_threshold(threshold);
                            gate.set_range(range);
                            gate.set_attack(attack);
                            gate.set_release(release);
                            
                            drop(router);
                            Some(Response::Ok {
                                message: format!("Gate {} updated on track {}", insert_id, track),
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Insert {} is not a gate", insert_id),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertReverb { track, insert_id, room_size, damping, wet, width, pre_delay } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        if let InsertEffectData::Reverb(rev) = &mut slot.effect {
                            rev.set_room_size(room_size);
                            rev.set_damping(damping);
                            rev.set_wet(wet);
                            rev.set_width(width);
                            rev.set_pre_delay(pre_delay);
                            
                            drop(router);
                            Some(Response::Ok {
                                message: format!("Reverb {} updated on track {}", insert_id, track),
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Insert {} is not a reverb", insert_id),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertDelay { track, insert_id, time_l, time_r, feedback, mix } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        if let InsertEffectData::Delay(delay) = &mut slot.effect {
                            delay.set_delay_time_left(time_l);
                            delay.set_delay_time_right(time_r);
                            delay.set_feedback(feedback);
                            delay.set_mix(mix);
                            
                            drop(router);
                            Some(Response::Ok {
                                message: format!("Delay {} updated on track {}", insert_id, track),
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Insert {} is not a delay", insert_id),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertExciter { track, insert_id, amount, frequency, mix } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        if let InsertEffectData::Exciter(exciter) = &mut slot.effect {
                            exciter.set_amount(amount);
                            exciter.set_frequency(frequency);
                            exciter.set_mix(mix);
                            
                            drop(router);
                            Some(Response::Ok {
                                message: format!("Exciter {} updated on track {}", insert_id, track),
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Insert {} is not an exciter", insert_id),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertDeEsser { track, insert_id, threshold, frequency, range } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        if let InsertEffectData::DeEsser(deesser) = &mut slot.effect {
                            deesser.set_threshold(threshold);
                            deesser.set_frequency(frequency);
                            deesser.set_range(range);
                            
                            drop(router);
                            Some(Response::Ok {
                                message: format!("DeEsser {} updated on track {}", insert_id, track),
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Insert {} is not a de-esser", insert_id),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            
            Command::SetTrackInsertChorus { track, insert_id, rate, depth, mix } => {
                let mut router = self.router.lock().unwrap();
                if let Some(t) = router.tracks.get_mut(track) {
                    if let Some(slot) = t.inserts.iter_mut().find(|s| s.id == insert_id) {
                        if let InsertEffectData::Chorus(chorus) = &mut slot.effect {
                            chorus.set_rate(rate);
                            chorus.set_depth(depth);
                            chorus.set_mix(mix);
                            
                            drop(router);
                            Some(Response::Ok {
                                message: format!("Chorus {} updated on track {}", insert_id, track),
                            })
                        } else {
                            drop(router);
                            Some(Response::Error {
                                message: format!("Insert {} is not a chorus", insert_id),
                            })
                        }
                    } else {
                        drop(router);
                        Some(Response::Error {
                            message: format!("Insert {} not found on track {}", insert_id, track),
                        })
                    }
                } else {
                    Some(Response::Error {
                        message: format!("Track {} not found", track),
                    })
                }
            },
            _ => None,
        }
    }
}
