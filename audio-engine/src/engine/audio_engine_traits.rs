// Audio Engine Traits - Logical method groupings
// This allows splitting impl blocks across multiple files

use anyhow::Result;
use crate::io::DeviceInfo;

/// Device management operations
pub trait AudioDeviceOps {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>>;
    fn start(
        &mut self,
        input_device: Option<String>,
        output_device: Option<String>,
        sample_rate: Option<u32>,
        buffer_size: Option<u32>,
    ) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

/// Audio input management
pub trait AudioInputOps {
    fn open_audio_input(&mut self, track_id: usize, device_name: Option<String>) -> Result<()>;
    fn close_audio_input(&mut self, track_id: usize) -> Result<()>;
}

/// Recording operations
pub trait RecordingOps {
    fn enable_master_tap(&self, file_path: String, sample_rate: u32, bit_depth: u32, format: &str);
    fn disable_master_tap(&self);
}

/// Track source management
pub trait TrackSourceOps {
    fn set_track_source_input(&mut self, track: usize, left_ch: u16, right_ch: u16, device_name: Option<String>) -> Result<()>;
    fn set_track_source_signal(&mut self, track: usize, waveform: &str, frequency: f32) -> Result<()>;
    fn set_signal_frequency(&mut self, track: usize, frequency: f32) -> Result<()>;
    fn set_signal_waveform(&mut self, track: usize, waveform: &str) -> Result<()>;
    fn clear_track_source(&mut self, track: usize) -> Result<()>;
    fn set_track_source_file(&mut self, track: usize, file_path: &str, artist: Option<&str>, title: Option<&str>, 
                             playlist_id: Option<&str>, playlist_name: Option<&str>, playlist_index: Option<usize>) -> Result<()>;
    fn set_track_source_aux_return(&mut self, track: usize, aux: usize) -> Result<()>;
}

/// File playback control
pub trait FilePlaybackOps {
    fn play_file(&mut self, track: usize, file_path: Option<&str>, artist: Option<&str>, title: Option<&str>) -> Result<()>;
    fn pause_file(&self, track: usize) -> Result<()>;
    fn stop_file(&self, track: usize) -> Result<()>;
    fn seek_file(&self, track: usize, time_seconds: f32) -> Result<()>;
    fn get_waveform_data(&self, track: usize, num_points: usize) -> Result<(Vec<f32>, f32, u32)>;
    fn stop_all_files(&self);
}

/// Track parameter control (gain, volume, mute, etc)
pub trait TrackControlOps {
    fn set_gain(&self, track: usize, gain: f32);
    fn set_volume(&self, track: usize, volume: f32);
    fn set_mute(&self, track: usize, mute: bool);
    fn set_solo(&self, track: usize, solo: bool);
    fn set_route_to_master(&self, track: usize, route: bool);
    fn set_pan(&self, track: usize, pan: f32);
    fn set_pad(&self, track: usize, enabled: bool);
    fn set_hpf(&self, track: usize, enabled: bool);
    fn set_phase_invert(&self, track: usize, enabled: bool);
    fn set_pfl(&self, track: usize, enabled: bool);
}

/// Track effects control (compressor, gate, EQ)
pub trait TrackEffectsOps {
    fn set_compressor(&self, track: usize, enabled: bool, threshold: f32, ratio: f32, attack: f32, release: f32);
    fn set_gate(&self, track: usize, enabled: bool, threshold: f32, range: f32, attack: f32, release: f32);
    fn set_eq(&self, track: usize, low: f32, low_mid: f32, high_mid: f32, high: f32);
    fn set_eq_enabled(&self, track: usize, enabled: bool);
}

/// Master bus control
pub trait MasterControlOps {
    fn set_master_gain(&self, gain: f32);
    fn set_master_gain_left(&self, gain: f32);
    fn set_master_gain_right(&self, gain: f32);
    fn set_master_mute(&self, mute: bool);
    fn set_master_linked(&self, linked: bool);
}

/// Command handling
pub trait CommandOps {
    fn handle_command(&mut self, command: crate::ipc::Command) -> Option<crate::ipc::Response>;
}
