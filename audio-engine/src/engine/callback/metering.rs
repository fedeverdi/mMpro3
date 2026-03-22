use crate::ipc::*;
use crate::processing::{Router, InsertEffectData};

/// Meter data tuple returned from collection
pub type MeterData = (
    Vec<TrackMeters>,
    Vec<SubgroupMeters>,
    Vec<AuxMeters>,
    f32, // master_l
    f32, // master_r
    f32, // master_gain
    f32, // master_gain_left
    f32, // master_gain_right
    bool, // master_linked
    bool, // master_mute
    crate::meters::headroom::HeadroomMeasurement,
    LoudnessDataStruct,
    DynamicRangeDataStruct,
    PhaseCorrelationDataStruct,
    StereoWidthDataStruct,
);

/// Collect all meter data from router
/// Returns structured meter data ready for serialization
pub fn collect_meter_data(router: &mut Router) -> MeterData {
    // Build meter structs directly (optimized - no legacy intermediate structs)
    let track_meters: Vec<TrackMeters> = router.tracks.iter()
        .map(|t| {
            // Find first compressor and gate in insert chain for visualization
            let (comp_input_db, comp_reduction_db) = t.inserts.iter()
                .find_map(|slot| {
                    if let InsertEffectData::Compressor(comp) = &slot.effect {
                        Some((comp.input_level_db, comp.gain_reduction_db))
                    } else {
                        None
                    }
                })
                .unwrap_or((-90.0, 0.0));
            
            let (gate_input_db, gate_attenuation_db) = t.inserts.iter()
                .find_map(|slot| {
                    if let InsertEffectData::Gate(gate) = &slot.effect {
                        Some((gate.input_level_db, gate.attenuation_db))
                    } else {
                        None
                    }
                })
                .unwrap_or((-90.0, 0.0));
            
            TrackMeters {
                track: t.id,
                level_l: t.level_l,
                level_r: t.level_r,
                level_pre_fader_l: t.level_pre_fader_l,
                level_pre_fader_r: t.level_pre_fader_r,
                waveform: t.get_waveform_buffer(128),
                phase_correlation: t.phase_correlation,
                compressor_input_db: comp_input_db,
                compressor_reduction_db: comp_reduction_db,
                gate_input_db: gate_input_db,
                gate_attenuation_db: gate_attenuation_db,
                file_ended: t.file_player.as_ref().map_or(false, |p| p.file_ended),
                is_playing: t.file_player.as_ref().map_or(false, |p| p.playing),
                bpm: t.bpm_detector.get_bpm(),
                // Include essential parameters for remote sync
                gain: t.gain,
                volume: t.volume,
                mute: t.mute,
                pan: t.pan,
                is_stereo: t.file_player.as_ref().map_or(false, |p| p.channels >= 2),
                file_name: t.file_player.as_ref().map_or(String::new(), |p| p.file_name.clone()),
                file_artist: t.file_player.as_ref().and_then(|p| p.file_artist.clone()),
                file_title: t.file_player.as_ref().and_then(|p| p.file_title.clone()),
                // Playlist state
                playlist_id: t.playlist_id.clone(),
                playlist_name: t.playlist_name.clone(),
                playlist_current_index: t.playlist_current_index,
            }
        })
        .collect();
    
    let subgroup_meters: Vec<SubgroupMeters> = router.subgroups.iter()
        .map(|sg| SubgroupMeters {
            subgroup: sg.id,
            level_l: sg.level_l,
            level_r: sg.level_r,
        })
        .collect();
    
    let aux_meters: Vec<AuxMeters> = router.aux_buses.iter()
        .map(|aux| AuxMeters {
            aux: aux.id,
            level_l: aux.level_l,
            level_r: aux.level_r,
        })
        .collect();
    
    let master_l = router.master.level_l;
    let master_r = router.master.level_r;
    let master_gain = router.master.gain;
    let master_gain_left = router.master.gain_left;
    let master_gain_right = router.master.gain_right;
    let master_linked = router.master.linked;
    let master_mute = router.master.mute;
    
    // Collect headroom data
    let headroom_data = router.headroom_meter.get_measurement();
    
    // Collect all other meter data
    let loudness_meas = router.loudness_meter.get_measurements();
    let dynamic_range_meas = router.dynamic_range_meter.get_measurements();
    let phase_correlation_meas = router.phase_correlation_meter.get_measurement();
    let stereo_width_meas = router.stereo_width_meter.get_measurement();
    
    // Convert to serializable structs
    let loudness_data = LoudnessDataStruct {
        momentary_lufs: loudness_meas.momentary,
        short_term_lufs: loudness_meas.short_term,
        integrated_lufs: loudness_meas.integrated,
        loudness_range_lu: loudness_meas.loudness_range,
        true_peak_dbtp: loudness_meas.true_peak_dbtp,
    };
    
    let dynamic_range_data = DynamicRangeDataStruct {
        peak_db_l: dynamic_range_meas.peak_db_l,
        peak_db_r: dynamic_range_meas.peak_db_r,
        rms_db_l: dynamic_range_meas.rms_db_l,
        rms_db_r: dynamic_range_meas.rms_db_r,
        dynamic_range_l: dynamic_range_meas.dynamic_range_l,
        dynamic_range_r: dynamic_range_meas.dynamic_range_r,
        dynamic_range_stereo: dynamic_range_meas.dynamic_range_stereo,
    };
    
    let phase_correlation_data = PhaseCorrelationDataStruct {
        correlation: phase_correlation_meas.correlation,
        mono_compatible: phase_correlation_meas.mono_compatible,
    };
    
    let stereo_width_data = StereoWidthDataStruct {
        width_percent: stereo_width_meas.width_percent,
        mid_rms: stereo_width_meas.mid_rms,
        side_rms: stereo_width_meas.side_rms,
        balance: stereo_width_meas.balance,
    };
    
    // Reset peak levels after reading
    router.master.reset_levels();
    for track in router.tracks.iter_mut() {
        track.reset_levels();
    }
    for subgroup in router.subgroups.iter_mut() {
        subgroup.reset_levels();
    }
    
    (
        track_meters,
        subgroup_meters,
        aux_meters,
        master_l,
        master_r,
        master_gain,
        master_gain_left,
        master_gain_right,
        master_linked,
        master_mute,
        headroom_data,
        loudness_data,
        dynamic_range_data,
        phase_correlation_data,
        stereo_width_data,
    )
}
