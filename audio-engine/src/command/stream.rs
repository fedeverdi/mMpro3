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
            Command::EnableMasterTap { file_path, sample_rate, bit_depth, format } => {
                self.enable_master_tap(
                    file_path,
                    sample_rate.unwrap_or(48000),
                    bit_depth.unwrap_or(16),
                    format.as_deref().unwrap_or("wav")
                );
                Some(Response::Ok {
                    message: "Recording started".to_string(),
                })
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
