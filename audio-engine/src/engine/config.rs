use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfigData {
    pub sample_rate: u32,  // 0 = Auto (use device default)
    pub buffer_size: u32,
}

impl Default for AudioConfigData {
    fn default() -> Self {
        AudioConfigData {
            sample_rate: 0,    // Auto
            buffer_size: 512,
        }
    }
}

pub fn get_config_file_path() -> PathBuf {
    let base_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base_path).join("audio_config.json")
}

pub fn load_audio_config() -> AudioConfigData {
    let path = get_config_file_path();
    
    if !path.exists() {
        eprintln!("[Engine] No config file found, using defaults");
        return AudioConfigData::default();
    }
    
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            match serde_json::from_str::<AudioConfigData>(&json) {
                Ok(config) => {
                    eprintln!("[Engine] ✓ Loaded config: sample_rate={}, buffer_size={}", 
                              config.sample_rate, config.buffer_size);
                    config
                }
                Err(e) => {
                    eprintln!("[Engine] ✗ Failed to parse config: {}, using defaults", e);
                    AudioConfigData::default()
                }
            }
        }
        Err(e) => {
            eprintln!("[Engine] ✗ Failed to read config file: {}, using defaults", e);
            AudioConfigData::default()
        }
    }
}

pub fn save_audio_config(config: &AudioConfigData) -> Result<()> {
    let path = get_config_file_path();
    let json = serde_json::to_string_pretty(config)?;
    
    match std::fs::write(&path, &json) {
        Ok(_) => {
            eprintln!("[Engine] ✓ Saved config: sample_rate={}, buffer_size={}", 
                      config.sample_rate, config.buffer_size);
            Ok(())
        }
        Err(e) => {
            eprintln!("[Engine] ✗ Failed to write config file: {}", e);
            Err(e.into())
        }
    }
}
