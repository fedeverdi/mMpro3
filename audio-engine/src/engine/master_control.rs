//! Master control helper functions
//!
//! This module contains helper functions for controlling Master bus parameters:
//! - Gain controls (unified, left, right, linked)
//! - Mute control
//! - Parametric EQ controls
//! - Output channel routing

use std::sync::{Arc, Mutex};
use crate::processing::routing::Router;
use crate::io::ChannelSelection;
use crate::ipc::ParametricFilter;
use crate::effects::tone::equalizer::FilterType;

/// Set master unified gain (updates left, right, and unified gain)
#[inline]
pub fn set_master_gain_impl(router: &Arc<Mutex<Router>>, gain: f32) {
    let mut router = router.lock().unwrap();
    router.master.gain = gain.max(0.0); // No upper limit
    // When setting unified gain, also update left/right
    router.master.gain_left = gain.max(0.0);
    router.master.gain_right = gain.max(0.0);
    let _gain_db = if gain > 0.0 { 20.0 * gain.log10() } else { -90.0 };
}

/// Set master left channel gain
#[inline]
pub fn set_master_gain_left_impl(router: &Arc<Mutex<Router>>, gain: f32) {
    let mut router = router.lock().unwrap();
    router.master.gain_left = gain.max(0.0);
}

/// Set master right channel gain
#[inline]
pub fn set_master_gain_right_impl(router: &Arc<Mutex<Router>>, gain: f32) {
    let mut router = router.lock().unwrap();
    router.master.gain_right = gain.max(0.0);
}

/// Set master mute state
#[inline]
pub fn set_master_mute_impl(router: &Arc<Mutex<Router>>, mute: bool) {
    let mut router = router.lock().unwrap();
    router.master.mute = mute;
}

/// Set master linked state (left/right gain linked)
#[inline]
pub fn set_master_linked_impl(router: &Arc<Mutex<Router>>, linked: bool) {
    let mut router = router.lock().unwrap();
    router.master.linked = linked;
}

/// Set master parametric EQ filters
pub fn set_master_parametric_eq_filters_impl(
    router: &Arc<Mutex<Router>>,
    filters: &[ParametricFilter],
) {
    let mut router = router.lock().unwrap();
    // Clear existing filters
    router.master.parametric_eq.clear();
    
    // Add new filters
    for filter in filters {
        let filter_type = match filter.filter_type.as_str() {
            "lowshelf" => FilterType::LowShelf,
            "highshelf" => FilterType::HighShelf,
            "peaking" => FilterType::Peaking,
            "lowpass" => FilterType::LowPass,
            "highpass" => FilterType::HighPass,
            _ => {
                eprintln!("[Master] Unknown filter type: {}", filter.filter_type);
                continue;
            }
        };
        
        router.master.parametric_eq.add_band(filter_type, filter.frequency, filter.gain, filter.q);
    }
}

/// Set master parametric EQ enabled state
#[inline]
pub fn set_master_parametric_eq_enabled_impl(router: &Arc<Mutex<Router>>, enabled: bool) {
    let mut router = router.lock().unwrap();
    router.master.parametric_eq.set_enabled(enabled);
}

/// Clear all master parametric EQ filters
#[inline]
pub fn clear_master_parametric_eq_impl(router: &Arc<Mutex<Router>>) {
    let mut router = router.lock().unwrap();
    router.master.parametric_eq.clear();
}

/// Set master output channel selection
#[inline]
pub fn set_master_output_channels_impl(router: &Arc<Mutex<Router>>, left_ch: u16, right_ch: u16) {
    let mut router = router.lock().unwrap();
    router.master.output_channel_selection = ChannelSelection::new(left_ch, right_ch);
}
