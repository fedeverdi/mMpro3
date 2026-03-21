// Audio callback helpers - Real-time safe utilities
// 
// This module contains helper functions for the audio callback to improve
// code organization without impacting real-time performance.
// All functions are designed to be zero-allocation or minimal-allocation.

pub mod frame_output;
pub mod metering;
pub mod fft;
pub mod performance;
pub mod recording;
