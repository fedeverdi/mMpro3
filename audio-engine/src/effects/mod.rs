// Audio effects processing
pub mod dynamics;
pub mod spatial;
pub mod tone;

// Re-export commonly used types 
pub use dynamics::{Compressor, Limiter, NoiseGate, DeEsser};
pub use spatial::{Reverb, Delay, Chorus};
pub use tone::{Equalizer, ParametricEqualizer, FilterData, FilterType, EQPreset, Exciter};
