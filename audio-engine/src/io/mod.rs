// Audio I/O and NDI streaming
pub mod audio_io;
pub mod ndi_ffi;
pub mod ndi_stream;

pub use audio_io::{AudioIO, ChannelSelection, DeviceInfo};
pub use ndi_stream::NdiStream;
