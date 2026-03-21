use crate::engine::audio_engine::AudioEngine;
use crate::ipc::messages::{Command, Response};
use crate::io::DeviceInfo;

impl AudioEngine {
    /// Handle audio input commands (OpenAudioInput, CloseAudioInput, ListAudioInputs)
    pub fn handle_audio_input_command(&self, command: Command) -> Option<Response> {
        match command {
            Command::OpenAudioInput { .. } => {
                // Note: This command is deprecated. Input is now managed automatically
                // via SetTrackSourceInput. Kept for backwards compatibility.
                eprintln!("[Engine] Warning: OpenAudioInput command is deprecated, use SetTrackSourceInput instead");
                None
            }
            Command::CloseAudioInput => {
                // Note: This command is deprecated. Input is now managed automatically
                // via track source changes. Kept for backwards compatibility.
                eprintln!("[Engine] Warning: CloseAudioInput command is deprecated, input is managed automatically");
                None
            }
            Command::ListAudioInputs => {
                match self.audio_io.list_devices() {
                    Ok(devices) => {
                        // Filter only input devices and expand multi-channel devices
                        let input_devices: Vec<DeviceInfo> = devices.into_iter()
                            .filter(|d| d.input_channels > 0)
                            .flat_map(|device| {
                                if device.input_channels > 2 {
                                    // Expand multi-channel device into individual channel entries
                                    let channel_count = device.input_channels as usize;
                                    eprintln!("[Engine] Expanding {} into {} channels", device.name, channel_count);
                                    (0..channel_count).map(move |ch| {
                                        DeviceInfo {
                                            id: format!("{}:{}", device.id, ch),
                                            name: format!("{} - Channel {}", device.name, ch + 1),
                                            input_channels: device.input_channels,
                                            output_channels: device.output_channels,
                                            default_sample_rate: device.default_sample_rate,
                                            is_default: false, // Individual channels are not marked as default
                                        }
                                    }).collect::<Vec<_>>()
                                } else {
                                    // Single or stereo channel device
                                    vec![device]
                                }
                            })
                            .collect();
                        
                        Some(Response::AudioInputs { inputs: input_devices })
                    },
                    Err(e) => {
                        eprintln!("[Engine] Error listing audio inputs: {}", e);
                        Some(Response::Error {
                            message: format!("Failed to list audio inputs: {}", e),
                        })
                    },
                }
            }
            _ => None,
        }
    }
}
