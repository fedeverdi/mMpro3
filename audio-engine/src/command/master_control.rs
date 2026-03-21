use crate::engine::AudioEngine;
use crate::ipc::{Command, Response, MasterParameters};

impl AudioEngine {
    pub fn handle_master_control_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::SetMasterGain { gain } => {
                self.set_master_gain(gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: Some(gain),
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterGainLeft { gain } => {
                self.set_master_gain_left(gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: Some(gain),
                        gain_right: None,
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterGainRight { gain } => {
                self.set_master_gain_right(gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: Some(gain),
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterMute { mute } => {
                self.set_master_mute(mute);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: Some(mute),
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterLinked { linked } => {
                self.set_master_linked(linked);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: Some(linked),
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterParametricEQFilters { filters } => {
                self.set_master_parametric_eq_filters(&filters);
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
                        selected_output: None,
                        eq_filters: Some(filters),
                        fx_effects: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterParametricEQEnabled { enabled } => {
                self.set_master_parametric_eq_enabled(enabled);
                None
            }
            Command::ClearMasterParametricEQ => {
                self.clear_master_parametric_eq();
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
                        selected_output: None,
                        eq_filters: Some(vec![]),
                        fx_effects: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterOutputChannels {
                left_channel,
                right_channel,
            } => {
                self.set_master_output_channels(left_channel, right_channel);
                None
            }
            Command::SetSelectedMasterOutput { device_id } => {
                *self.selected_master_output.lock().unwrap() = device_id.clone();
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
                        eq_filters: None,
                        fx_effects: None,
                        selected_output: Some(device_id),
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            _ => None,
        }
    }
}
