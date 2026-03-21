use crate::engine::AudioEngine;
use crate::ipc::{Command, Response, MasterParameters};

impl AudioEngine {
    pub fn handle_master_effects_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::SetMasterCompressor {
                enabled,
                threshold,
                ratio,
                attack,
                release,
            } => {
                println!("[Engine] SetMasterCompressor: enabled={}", enabled);
                self.set_master_compressor(enabled, threshold, ratio, attack, release);
                let fx_effects = self.get_master_fx_effects();
                println!("[Engine] Returning fx_effects: {:?}", fx_effects);
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
                        fx_effects: Some(fx_effects),
                        selected_output: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterLimiter {
                enabled,
                ceiling,
                release,
            } => {
                self.set_master_limiter(enabled, ceiling, release);
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
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterDelay {
                enabled,
                time_l,
                time_r,
                feedback,
                mix,
            } => {
                self.set_master_delay(enabled, time_l, time_r, feedback, mix);
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
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::SetMasterReverb {
                enabled,
                room_size,
                damping,
                wet,
                width,
                pre_delay,
            } => {
                self.set_master_reverb(enabled, room_size, damping, wet, width, pre_delay);
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
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::AddMasterFxEffect { effect_type } => {
                self.add_master_fx_effect(&effect_type);
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
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            Command::RemoveMasterFxEffect { effect_type } => {
                self.remove_master_fx_effect(&effect_type);
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
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                        current_eq_preset: None,
                    }),
                })
            }
            _ => None,
        }
    }
}
