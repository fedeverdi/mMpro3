//! Master FX control helper functions
//!
//! This module contains helper functions for controlling Master FX effects:
//! - Compressor, Limiter, Delay, Reverb
//! - Effect list management (add/remove)
//! - State synchronization (get all effects)

use std::sync::{Arc, Mutex};
use crate::processing::routing::Router;
use crate::ipc::MasterFxEffect;

/// Set master compressor parameters
#[inline]
pub fn set_master_compressor_impl(
    router: &Arc<Mutex<Router>>,
    enabled: bool,
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
) {
    let mut router = router.lock().unwrap();
    router.master.compressor.set_enabled(enabled);
    router.master.compressor.set_threshold(threshold);
    router.master.compressor.set_ratio(ratio);
    router.master.compressor.set_attack(attack);
    router.master.compressor.set_release(release);
}

/// Set master limiter parameters
#[inline]
pub fn set_master_limiter_impl(
    router: &Arc<Mutex<Router>>,
    enabled: bool,
    ceiling: f32,
    release: f32,
) {
    let mut router = router.lock().unwrap();
    router.master.limiter.set_enabled(enabled);
    router.master.limiter.set_ceiling(ceiling);
    router.master.limiter.set_release(release);
}

/// Set master delay parameters
#[inline]
pub fn set_master_delay_impl(
    router: &Arc<Mutex<Router>>,
    enabled: bool,
    time_l: f32,
    time_r: f32,
    feedback: f32,
    mix: f32,
) {
    let mut router = router.lock().unwrap();
    router.master.delay.set_enabled(enabled);
    router.master.delay.set_delay_time_left(time_l);
    router.master.delay.set_delay_time_right(time_r);
    router.master.delay.set_feedback(feedback);
    router.master.delay.set_mix(mix);
}

/// Set master reverb parameters
#[inline]
pub fn set_master_reverb_impl(
    router: &Arc<Mutex<Router>>,
    enabled: bool,
    room_size: f32,
    damping: f32,
    wet: f32,
    width: f32,
    pre_delay: f32,
) {
    let mut router = router.lock().unwrap();
    router.master.reverb.set_enabled(enabled);
    router.master.reverb.set_room_size(room_size);
    router.master.reverb.set_damping(damping);
    router.master.reverb.set_wet(wet);
    router.master.reverb.set_width(width);
    router.master.reverb.set_pre_delay(pre_delay);
}

/// Get current state of all Master FX effects for synchronization
pub fn get_master_fx_effects_impl(router: &Arc<Mutex<Router>>) -> Vec<MasterFxEffect> {
    let router = router.lock().unwrap();
    let master = &router.master;
    
    let mut effects = Vec::new();
    
    // Only include effects that are "present" in the FX list
    if master.compressor_present {
        effects.push(MasterFxEffect::Compressor {
            enabled: master.compressor.is_enabled(),
            threshold: master.compressor.get_threshold(),
            ratio: master.compressor.get_ratio(),
            attack: master.compressor.get_attack(),
            release: master.compressor.get_release(),
        });
    }
    
    if master.limiter_present {
        effects.push(MasterFxEffect::Limiter {
            enabled: master.limiter.is_enabled(),
            threshold: master.limiter.get_ceiling(),
            release: master.limiter.get_release(),
        });
    }
    
    if master.delay_present {
        effects.push(MasterFxEffect::Delay {
            enabled: master.delay.is_enabled(),
            time_l: master.delay.get_delay_time_l_ms(),
            time_r: master.delay.get_delay_time_r_ms(),
            feedback: master.delay.get_feedback(),
            mix: master.delay.get_mix(),
        });
    }
    
    if master.reverb_present {
        effects.push(MasterFxEffect::Reverb {
            enabled: master.reverb.is_enabled(),
            room_size: master.reverb.get_room_size(),
            damping: master.reverb.get_damping(),
            wet: master.reverb.get_wet(),
            width: master.reverb.get_width(),
        });
    }
    
    println!("[Engine] get_master_fx_effects returning {} effects", effects.len());
    effects
}

/// Add a Master FX effect to the FX list
pub fn add_master_fx_effect_impl(router: &Arc<Mutex<Router>>, effect_type: &str) {
    let mut router = router.lock().unwrap();
    let master = &mut router.master;
    
    match effect_type {
        "compressor" => {
            master.compressor_present = true;
            println!("[Engine] Added Master Compressor to FX list");
        }
        "limiter" => {
            master.limiter_present = true;
            println!("[Engine] Added Master Limiter to FX list");
        }
        "delay" => {
            master.delay_present = true;
            println!("[Engine] Added Master Delay to FX list");
        }
        "reverb" => {
            master.reverb_present = true;
            println!("[Engine] Added Master Reverb to FX list");
        }
        _ => {
            eprintln!("[Engine] Unknown effect type: {}", effect_type);
        }
    }
}

/// Remove a Master FX effect from the FX list
pub fn remove_master_fx_effect_impl(router: &Arc<Mutex<Router>>, effect_type: &str) {
    let mut router = router.lock().unwrap();
    let master = &mut router.master;
    
    match effect_type {
        "compressor" => {
            master.compressor_present = false;
            master.compressor.set_enabled(false); // Also disable it
            println!("[Engine] Removed Master Compressor from FX list");
        }
        "limiter" => {
            master.limiter_present = false;
            master.limiter.set_enabled(false); // Also disable it
            println!("[Engine] Removed Master Limiter from FX list");
        }
        "delay" => {
            master.delay_present = false;
            master.delay.set_enabled(false); // Also disable it
            println!("[Engine] Removed Master Delay from FX list");
        }
        "reverb" => {
            master.reverb_present = false;
            master.reverb.set_enabled(false); // Also disable it
            println!("[Engine] Removed Master Reverb from FX list");
        }
        _ => {
            eprintln!("[Engine] Unknown effect type: {}", effect_type);
        }
    }
}
