use crate::engine::AudioEngine;
use crate::ipc::{Command, Response, EQPresetData, EQPresetFilter, ParametricFilter, MasterParameters};

impl AudioEngine {
    pub fn handle_eq_presets_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::GetEQPresets => {
                use crate::effects::tone::equalizer::EQPreset;
                
                let presets = EQPreset::get_presets();
                let preset_data: Vec<EQPresetData> = presets.into_iter().map(|p| {
                    EQPresetData {
                        name: p.name,
                        filters: p.filters.into_iter().map(|(filter_type, frequency, gain, q)| {
                            EQPresetFilter {
                                filter_type: filter_type.to_string(),
                                frequency,
                                gain,
                                q,
                            }
                        }).collect(),
                    }
                }).collect();
                
                Some(Response::EQPresets {
                    presets: preset_data,
                })
            },
            Command::ApplyEQPreset { preset_name } => {
                use crate::effects::tone::equalizer::EQPreset;
                
                let presets = EQPreset::get_presets();
                if let Some(preset) = presets.into_iter().find(|p| p.name == preset_name) {
                    let mut router = self.router.lock().unwrap();
                    preset.apply_to(&mut router.master.parametric_eq);
                    
                    // Get updated filters from the equalizer
                    let filters_data = router.master.parametric_eq.export_filters();
                    let filters: Vec<ParametricFilter> = filters_data.iter().map(|f| {
                        ParametricFilter {
                            filter_type: f.filter_type.clone(),
                            frequency: f.frequency,
                            gain: f.gain,
                            q: f.q,
                        }
                    }).collect();
                    
                    drop(router);
                    
                    // Send ParametersChanged instead of Ok
                    Some(Response::ParametersChanged {
                        tracks: None,
                        subgroups: None,
                        auxes: None,
                        master: Some(MasterParameters {
                            gain: None,
                            gain_left: None,
                            gain_right: None,
                            mute: None,
                            linked: None,
                            eq_filters: Some(filters),
                            current_eq_preset: Some(Some(preset_name.clone())),
                            fx_effects: None,
                            selected_output: None,
                            available_output_devices: None,
                        }),
                    })
                } else {
                    Some(Response::Error {
                        message: format!("EQ preset not found: {}", preset_name),
                    })
                }
            },
            _ => None,
        }
    }
}
