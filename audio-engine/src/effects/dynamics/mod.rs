// Dynamics processing effects
pub mod compressor;
pub mod limiter;
pub mod gate;
pub mod deesser;

pub use compressor::Compressor;
pub use limiter::Limiter;
pub use gate::NoiseGate;
pub use deesser::DeEsser;
