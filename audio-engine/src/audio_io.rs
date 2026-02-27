/// Audio I/O management with device and channel selection
use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, StreamConfig};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub input_channels: u16,
    pub output_channels: u16,
    pub default_sample_rate: u32,
}

pub struct AudioIO {
    host: Host,
}

impl AudioIO {
    pub fn new() -> Self {
        let host = cpal::default_host();
        Self { host }
    }

    /// List all available audio devices (input and output)
    pub fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let mut devices = Vec::new();

        // Input devices
        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(info) = self.get_device_info(&device, true) {
                    devices.push(info);
                }
            }
        }

        // Output devices
        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(info) = self.get_device_info(&device, false) {
                    // Check if device already in list (some devices are both input/output)
                    if !devices.iter().any(|d| d.name == info.name) {
                        devices.push(info);
                    } else {
                        // Update existing device with output channels
                        if let Some(existing) = devices.iter_mut().find(|d| d.name == info.name) {
                            existing.output_channels = info.output_channels;
                        }
                    }
                }
            }
        }

        Ok(devices)
    }

    /// Get device info
    fn get_device_info(&self, device: &Device, is_input: bool) -> Result<DeviceInfo> {
        let name = device.name()?;
        
        // Try to get supported configs
        let channels = if is_input {
            if let Ok(configs) = device.supported_input_configs() {
                configs
                    .filter_map(|c| Some(c.channels()))
                    .max()
                    .unwrap_or(2)
            } else {
                2
            }
        } else {
            if let Ok(configs) = device.supported_output_configs() {
                configs
                    .filter_map(|c| Some(c.channels()))
                    .max()
                    .unwrap_or(2)
            } else {
                2
            }
        };

        let sample_rate = if is_input {
            device
                .default_input_config()
                .map(|c| c.sample_rate().0)
                .unwrap_or(48000)
        } else {
            device
                .default_output_config()
                .map(|c| c.sample_rate().0)
                .unwrap_or(48000)
        };

        Ok(DeviceInfo {
            id: name.clone(),
            name,
            input_channels: if is_input { channels } else { 0 },
            output_channels: if !is_input { channels } else { 0 },
            default_sample_rate: sample_rate,
        })
    }

    /// Get default input device
    pub fn default_input_device(&self) -> Result<Device> {
        self.host
            .default_input_device()
            .ok_or_else(|| anyhow!("No default input device available"))
    }

    /// Get default output device
    pub fn default_output_device(&self) -> Result<Device> {
        self.host
            .default_output_device()
            .ok_or_else(|| anyhow!("No default output device available"))
    }

    /// Find device by name
    pub fn find_device_by_name(&self, name: &str, is_input: bool) -> Result<Device> {
        let devices = if is_input {
            self.host.input_devices()?
        } else {
            self.host.output_devices()?
        };

        for device in devices {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Ok(device);
                }
            }
        }

        Err(anyhow!("Device '{}' not found", name))
    }

    /// Get supported config for device
    pub fn get_supported_config(
        &self,
        device: &Device,
        is_input: bool,
        sample_rate: Option<u32>,
        buffer_size: Option<u32>,
    ) -> Result<StreamConfig> {
        let config = if is_input {
            device.default_input_config()?
        } else {
            device.default_output_config()?
        };

        let mut stream_config = config.config();

        // Override sample rate if specified
        if let Some(rate) = sample_rate {
            stream_config.sample_rate = cpal::SampleRate(rate);
        }

        // Override buffer size if specified
        if let Some(size) = buffer_size {
            stream_config.buffer_size = cpal::BufferSize::Fixed(size);
        }

        Ok(stream_config)
    }
}

impl Default for AudioIO {
    fn default() -> Self {
        Self::new()
    }
}

/// Channel selection for routing
#[derive(Debug, Clone, Copy)]
pub struct ChannelSelection {
    pub left: u16,  // Channel index for left (0-based)
    pub right: u16, // Channel index for right (0-based)
}

impl ChannelSelection {
    pub fn new(left: u16, right: u16) -> Self {
        Self { left, right }
    }

    /// Default stereo selection (channels 0,1)
    pub fn stereo() -> Self {
        Self { left: 0, right: 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_io_creation() {
        let io = AudioIO::new();
        assert!(io.list_devices().is_ok());
    }

    #[test]
    fn test_channel_selection() {
        let stereo = ChannelSelection::stereo();
        assert_eq!(stereo.left, 0);
        assert_eq!(stereo.right, 1);

        let mono = ChannelSelection::mono(0);
        assert_eq!(mono.left, 0);
        assert_eq!(mono.right, 0);
    }
}
