// Tonal/frequency-based effects
pub mod equalizer;
pub mod exciter;

pub use equalizer::{Equalizer, ParametricEqualizer, FilterData, FilterType, EQPreset};
pub use exciter::Exciter;
