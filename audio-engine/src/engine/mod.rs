// Engine module - Contains AudioEngine, config, license, helpers

pub mod config;
pub mod license;
pub mod helpers;
pub mod audio_engine;
pub(crate) mod callback;  // Audio callback helpers (real-time safe)
mod audio_engine_traits;  // Trait definitions for splitting impl blocks

pub use config::*;
pub use license::*;
pub use helpers::*;
pub use audio_engine::AudioEngine;
