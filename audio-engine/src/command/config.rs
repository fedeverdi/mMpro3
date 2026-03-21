use crate::engine::audio_engine::AudioEngine;
use crate::engine::config::{load_audio_config, save_audio_config, AudioConfigData};
use crate::engine::license::{load_license_from_file, save_license_to_file};
use crate::ipc::messages::{Command, Response};

impl AudioEngine {
    /// Handle configuration commands (SaveAudioConfig, GetAudioConfig, SaveLicense, GetLicense)
    pub fn handle_config_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::SaveLicense { key, license_type, expires_at } => {
                match save_license_to_file(&key, &license_type, expires_at.as_deref()) {
                    Ok(_) => Some(Response::Ok {
                        message: "License saved".to_string(),
                    }),
                    Err(e) => Some(Response::Error {
                        message: format!("Failed to save license: {}", e),
                    }),
                }
            }
            Command::GetLicense => {
                match load_license_from_file() {
                    Ok(license) => Some(Response::License {
                        key: license.key,
                        license_type: license.license_type,
                        expires_at: license.expires_at,
                        is_valid: license.is_valid,
                    }),
                    Err(_) => Some(Response::License {
                        key: "DEMO".to_string(),
                        license_type: "demo".to_string(),
                        expires_at: None,
                        is_valid: true,
                    }),
                }
            }
            Command::SaveAudioConfig { sample_rate, buffer_size } => {
                let config = AudioConfigData { sample_rate, buffer_size };
                match save_audio_config(&config) {
                    Ok(_) => Some(Response::Ok {
                        message: "Audio config saved".to_string(),
                    }),
                    Err(e) => Some(Response::Error {
                        message: format!("Failed to save audio config: {}", e),
                    }),
                }
            }
            Command::GetAudioConfig => {
                let config = load_audio_config();
                Some(Response::AudioConfig {
                    sample_rate: config.sample_rate,
                    buffer_size: config.buffer_size,
                })
            }
            _ => None,
        }
    }
}
