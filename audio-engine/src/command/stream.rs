use crate::engine::audio_engine::AudioEngine;
use crate::ipc::messages::{Command, Response};

impl AudioEngine {
    /// Handle stream and device commands (Start, Stop, EnableMasterTap, DisableMasterTap)
    pub fn handle_stream_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::Start { input_device, output_device, sample_rate, buffer_size } => {
                match self.start(input_device, output_device, sample_rate, buffer_size) {
                    Ok(_) => Some(Response::Started),
                    Err(e) => Some(Response::Error {
                        message: format!("Start failed: {}", e),
                    }),
                }
            }
            Command::Stop => {
                match self.stop() {
                    Ok(_) => Some(Response::Stopped),
                    Err(e) => Some(Response::Error {
                        message: format!("Stop failed: {}", e),
                    }),
                }
            }
            Command::EnableMasterTap { sample_rate, bit_depth, format } => {
                // Generate recording path automatically (Rust manages the file location)
                match crate::engine::recording::generate_recording_path() {
                    Ok(file_path) => {
                        self.enable_master_tap(
                            file_path.to_string_lossy().to_string(),
                            sample_rate.unwrap_or(48000),
                            bit_depth.unwrap_or(16),
                            format.as_deref().unwrap_or("wav")
                        );
                        Some(Response::Ok {
                            message: format!("Recording started: {}", file_path.display()),
                        })
                    },
                    Err(e) => {
                        Some(Response::Error {
                            message: format!("Failed to create recording path: {}", e),
                        })
                    }
                }
            }
            Command::DisableMasterTap => {
                self.disable_master_tap();
                Some(Response::Ok {
                    message: "Recording stopped and saved".to_string(),
                })
            }
            _ => None,
        }
    }
}
