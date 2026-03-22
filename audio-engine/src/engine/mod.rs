// Engine module - Contains AudioEngine, config, license, helpers

pub mod config;
pub mod license;
pub mod helpers;
pub mod audio_engine;
pub mod snapshot; // Scene snapshot structures
pub(crate) mod callback;  // Audio callback helpers (real-time safe)
pub(crate) mod recording; // Recording and WAV file writing
pub(crate) mod track_control; // Track control trait and methods
pub(crate) mod master_control; // Master bus control methods
pub(crate) mod master_fx_control; // Master FX control methods
pub(crate) mod aux_control; // Aux bus control methods
pub(crate) mod snapshot_control; // Scene snapshot save/load methods

pub use config::*;
pub use license::*;
pub use helpers::*;
pub use audio_engine::AudioEngine;
