use crate::engine::AudioEngine;
use crate::ipc::{Command, Response, SubgroupParameters};

impl AudioEngine {
    pub fn handle_subgroups_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::AddSubgroup => {
                let id = self.add_subgroup();
                Some(Response::SubgroupCreated { id })
            }
            Command::RemoveSubgroup { subgroup } => {
                self.remove_subgroup(subgroup);
                None
            }
            Command::SetSubgroupGain { subgroup, gain } => {
                self.set_subgroup_gain(subgroup, gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: Some(gain),
                        mute: None,
                        route_to_master: None,
                        selected_output: None,
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSubgroupMute { subgroup, mute } => {
                self.set_subgroup_mute(subgroup, mute);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: None,
                        mute: Some(mute),
                        route_to_master: None,
                        selected_output: None,
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSubgroupOutputEnabled { subgroup, enabled } => {
                self.set_subgroup_output_enabled(subgroup, enabled);
                None
            }
            Command::SetSubgroupRouteToMaster { subgroup, route } => {
                self.set_subgroup_route_to_master(subgroup, route);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: None,
                        mute: None,
                        route_to_master: Some(route),
                        selected_output: None,
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSubgroupOutputChannels {
                subgroup,
                left_channel,
                right_channel,
            } => {
                self.set_subgroup_output_channels(subgroup, left_channel, right_channel);
                None
            }
            Command::SetSelectedSubgroupOutput { subgroup, device_id } => {
                if let Ok(mut router) = self.router.try_lock() {
                    if let Some(sg) = router.subgroups.iter_mut().find(|s| s.id == subgroup) {
                        sg.selected_output = device_id.clone();
                    }
                }
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        selected_output: Some(device_id),
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            _ => None,
        }
    }
}
