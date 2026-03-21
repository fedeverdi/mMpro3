use crate::engine::audio_engine::AudioEngine;
use crate::io::ndi_stream::NdiSource;
use crate::ipc::messages::{Command, Response};

impl AudioEngine {
    /// Handle NDI streaming commands (StartNdi, StopNdi, SetNdiSource, SetNdiName, SetNdiVideoText)
    pub fn handle_ndi_command(&self, command: Command) -> Option<Response> {
        match command {
            Command::StartNdi { stream_name, source } => {
                match NdiSource::from_str(&source) {
                    Some(ndi_source) => {
                        match self.ndi_stream.start(stream_name, ndi_source) {
                            Ok(_) => Some(Response::Ok {
                                message: "NDI stream started".to_string(),
                            }),
                            Err(e) => Some(Response::Error {
                                message: format!("Failed to start NDI: {}", e),
                            }),
                        }
                    }
                    None => Some(Response::Error {
                        message: format!("Invalid NDI source: {}", source),
                    }),
                }
            }
            Command::StopNdi => {
                match self.ndi_stream.stop() {
                    Ok(_) => Some(Response::Ok {
                        message: "NDI stream stopped".to_string(),
                    }),
                    Err(e) => Some(Response::Error {
                        message: format!("Failed to stop NDI: {}", e),
                    }),
                }
            }
            Command::SetNdiSource { source } => {
                match NdiSource::from_str(&source) {
                    Some(ndi_source) => {
                        match self.ndi_stream.set_source(ndi_source) {
                            Ok(_) => None, // Silent success
                            Err(e) => Some(Response::Error {
                                message: format!("Failed to set NDI source: {}", e),
                            }),
                        }
                    }
                    None => Some(Response::Error {
                        message: format!("Invalid NDI source: {}", source),
                    }),
                }
            }
            Command::SetNdiName { name } => {
                match self.ndi_stream.set_stream_name(name) {
                    Ok(_) => None, // Silent success
                    Err(e) => Some(Response::Error {
                        message: format!("Failed to set NDI name: {}", e),
                    }),
                }
            }
            Command::SetNdiVideoText { text } => {
                self.ndi_stream.set_video_text(text);
                None // Silent success
            }
            _ => None,
        }
    }
}
