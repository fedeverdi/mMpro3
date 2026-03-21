//! Aux bus control helper functions
//!
//! This module contains helper functions for controlling Aux buses:
//! - Gain and mute controls
//! - Reverb and delay effects
//! - Routing (master, subgroups)
//! - Output channel selection

use std::sync::{Arc, Mutex};
use crate::processing::routing::Router;
use crate::io::ChannelSelection;

/// Set aux bus gain
#[inline]
pub fn set_aux_bus_gain_impl(router: &Arc<Mutex<Router>>, aux: usize, gain: f32) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        router.aux_buses[aux].gain = gain.max(0.0);
    }
}

/// Set aux bus mute state
#[inline]
pub fn set_aux_bus_mute_impl(router: &Arc<Mutex<Router>>, aux: usize, mute: bool) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        router.aux_buses[aux].mute = mute;
    }
}

/// Set aux bus reverb parameters
pub fn set_aux_bus_reverb_impl(
    router: &Arc<Mutex<Router>>,
    aux: usize,
    enabled: bool,
    room_size: f32,
    damping: f32,
    wet: f32,
    width: f32,
    pre_delay: f32,
) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        router.aux_buses[aux].reverb.set_enabled(enabled);
        router.aux_buses[aux].reverb.set_room_size(room_size);
        router.aux_buses[aux].reverb.set_damping(damping);
        router.aux_buses[aux].reverb.set_wet(wet);
        router.aux_buses[aux].reverb.set_width(width);
        router.aux_buses[aux].reverb.set_pre_delay(pre_delay);
    }
}

/// Set aux bus delay parameters
pub fn set_aux_bus_delay_impl(
    router: &Arc<Mutex<Router>>,
    aux: usize,
    enabled: bool,
    time: f32,
    feedback: f32,
    mix: f32,
) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        router.aux_buses[aux].delay.set_enabled(enabled);
        router.aux_buses[aux].delay.set_delay_time_left(time);
        router.aux_buses[aux].delay.set_delay_time_right(time);
        router.aux_buses[aux].delay.set_feedback(feedback);
        router.aux_buses[aux].delay.set_mix(mix);
    }
}

/// Set aux bus routing to master
#[inline]
pub fn set_aux_bus_route_to_master_impl(router: &Arc<Mutex<Router>>, aux: usize, route: bool) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        router.aux_buses[aux].route_to_master = route;
    }
}

/// Set aux bus routing to subgroup
pub fn set_aux_bus_route_to_subgroup_impl(
    router: &Arc<Mutex<Router>>,
    aux: usize,
    subgroup: usize,
    route: bool,
) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        if route {
            // Add subgroup to routing if not already present
            if !router.aux_buses[aux].route_to_subgroups.contains(&subgroup) {
                router.aux_buses[aux].route_to_subgroups.push(subgroup);
            }
        } else {
            // Remove subgroup from routing
            router.aux_buses[aux].route_to_subgroups.retain(|&sg| sg != subgroup);
        }
    }
}

/// Set aux bus output enabled state
#[inline]
pub fn set_aux_bus_output_enabled_impl(router: &Arc<Mutex<Router>>, aux: usize, enabled: bool) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        router.aux_buses[aux].output_enabled = enabled;
    }
}

/// Set aux bus output channel selection
#[inline]
pub fn set_aux_bus_output_channels_impl(
    router: &Arc<Mutex<Router>>,
    aux: usize,
    left_ch: u16,
    right_ch: u16,
) {
    let mut router = router.lock().unwrap();
    if aux < router.aux_buses.len() {
        router.aux_buses[aux].output_channel_selection = ChannelSelection::new(left_ch, right_ch);
    }
}
