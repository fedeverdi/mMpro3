use crate::engine::AudioEngine;
use crate::ipc::{Command, Response, AuxParameters, AuxReverbParams, AuxDelayParams};

impl AudioEngine {
    pub fn handle_aux_buses_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::SetAuxBusGain { aux, gain } => {
                self.set_aux_bus_gain(aux, gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: Some(gain),
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusMute { aux, mute } => {
                self.set_aux_bus_mute(aux, mute);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: Some(mute),
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusReverb {
                aux,
                enabled,
                room_size,
                damping,
                wet,
                width,
                pre_delay,
            } => {
                self.set_aux_bus_reverb(aux, enabled, room_size, damping, wet, width, pre_delay);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: Some(AuxReverbParams {
                            enabled,
                            room_size,
                            damping,
                            wet,
                            width,
                        }),
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusDelay {
                aux,
                enabled,
                time,
                feedback,
                mix,
            } => {
                self.set_aux_bus_delay(aux, enabled, time, feedback, mix);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: Some(AuxDelayParams {
                            enabled,
                            delay_time_l_ms: time,
                            delay_time_r_ms: time,
                            feedback,
                            mix,
                        }),
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusRouteToMaster { aux, route } => {
                self.set_aux_bus_route_to_master(aux, route);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: Some(route),
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusOutputEnabled { aux, enabled } => {
                self.set_aux_bus_output_enabled(aux, enabled);
                None
            }
            Command::SetAuxBusOutputChannels {
                aux,
                left_channel,
                right_channel,
            } => {
                self.set_aux_bus_output_channels(aux, left_channel, right_channel);
                None
            }
            Command::SetAuxBusRouteToSubgroup { aux, subgroup, route } => {
                self.set_aux_bus_route_to_subgroup(aux, subgroup, route);
                
                // Get updated routing list
                let route_to_subgroups = if let Ok(router) = self.router.try_lock() {
                    if let Some(aux_bus) = router.aux_buses.iter().find(|a| a.id == aux) {
                        aux_bus.route_to_subgroups.clone()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: Some(route_to_subgroups),
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusSelectedOutput { aux, device_id } => {
                if let Ok(mut router) = self.router.try_lock() {
                    if let Some(aux_bus) = router.aux_buses.iter_mut().find(|a| a.id == aux) {
                        aux_bus.selected_output = device_id.clone();
                    }
                }
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: Some(device_id),
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            _ => None,
        }
    }
}
