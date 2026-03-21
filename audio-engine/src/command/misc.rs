use crate::engine::audio_engine::AudioEngine;
use crate::ipc::messages::{Command, Response};
use std::sync::atomic::Ordering;

impl AudioEngine {
    /// Handle miscellaneous commands (ListDevices, SetUpdatesSuspended, etc.)
    pub fn handle_misc_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::ListDevices => {
                match self.list_devices() {
                    Ok(devices) => Some(Response::Devices { devices }),
                    Err(e) => Some(Response::Error {
                        message: format!("List devices failed: {}", e),
                    }),
                }
            }
            Command::SetUpdatesSuspended { suspended } => {
                self.updates_suspended.store(suspended, Ordering::Relaxed);
                None // No response needed, performance-critical
            }
            _ => None,
        }
    }
}
