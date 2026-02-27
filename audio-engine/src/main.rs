use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, Stream, StreamConfig};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};

/// Comando ricevuto da Electron via stdin
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Command {
    #[serde(rename = "start")]
    Start {
        device_id: Option<String>,
        sample_rate: Option<u32>,
        buffer_size: Option<u32>,
    },
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "set_gain")]
    SetGain { track: usize, gain: f32 },
    #[serde(rename = "set_mute")]
    SetMute { track: usize, mute: bool },
    #[serde(rename = "list_devices")]
    ListDevices,
}

/// Risposta inviata a Electron via stdout
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum Response {
    #[serde(rename = "ok")]
    Ok { message: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "devices")]
    Devices { devices: Vec<DeviceInfo> },
    #[serde(rename = "level")]
    Level { track: usize, level: f32 },
}

#[derive(Debug, Serialize)]
struct DeviceInfo {
    id: String,
    name: String,
    input_channels: u16,
    output_channels: u16,
}

/// Stato del mixer audio
struct MixerState {
    tracks: Vec<TrackState>,
    master_gain: f32,
}

struct TrackState {
    gain: f32,
    mute: bool,
}

impl MixerState {
    fn new(num_tracks: usize) -> Self {
        Self {
            tracks: (0..num_tracks)
                .map(|_| TrackState {
                    gain: 1.0,
                    mute: false,
                })
                .collect(),
            master_gain: 1.0,
        }
    }

    fn set_track_gain(&mut self, track: usize, gain: f32) {
        if let Some(t) = self.tracks.get_mut(track) {
            t.gain = gain.clamp(0.0, 2.0);
        }
    }

    fn set_track_mute(&mut self, track: usize, mute: bool) {
        if let Some(t) = self.tracks.get_mut(track) {
            t.mute = mute;
        }
    }
}

/// Engine audio principale
struct AudioEngine {
    host: Host,
    stream: Option<Stream>,
    state: Arc<Mutex<MixerState>>,
}

impl AudioEngine {
    fn new() -> Self {
        // Usa host di default (CoreAudio su macOS, WASAPI su Windows)
        let host = cpal::default_host();
        
        eprintln!("[Engine] Using host: {:?}", host.id());

        Self {
            host,
            stream: None,
            state: Arc::new(Mutex::new(MixerState::new(32))), // 32 tracce
        }
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let mut devices = Vec::new();

        // Input devices
        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let config = device.default_input_config().ok();
                    devices.push(DeviceInfo {
                        id: name.clone(),
                        name,
                        input_channels: config.map(|c| c.channels()).unwrap_or(0),
                        output_channels: 0,
                    });
                }
            }
        }

        // Output devices
        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    let config = device.default_output_config().ok();
                    devices.push(DeviceInfo {
                        id: name.clone(),
                        name,
                        input_channels: 0,
                        output_channels: config.map(|c| c.channels()).unwrap_or(0),
                    });
                }
            }
        }

        Ok(devices)
    }

    fn start(&mut self, _device_id: Option<String>, sample_rate: Option<u32>, buffer_size: Option<u32>) -> Result<()> {
        if self.stream.is_some() {
            return Ok(()); // Already running
        }

        // Trova device input/output (per ora usa default)
        let input_device = self.host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;
        let output_device = self.host.default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

        eprintln!("[Engine] Input device: {:?}", input_device.name());
        eprintln!("[Engine] Output device: {:?}", output_device.name());

        // Config
        let mut config: StreamConfig = input_device.default_input_config()?.into();
        if let Some(sr) = sample_rate {
            config.sample_rate = cpal::SampleRate(sr);
        }
        if let Some(bs) = buffer_size {
            config.buffer_size = cpal::BufferSize::Fixed(bs);
        }

        eprintln!("[Engine] Config: {:?}", config);

        // Crea stream duplex (input + output)
        let state = Arc::clone(&self.state);
        let err_fn = |err| eprintln!("[Engine] Stream error: {}", err);

        // Per ora: passthrough semplice (input -> output)
        let stream = output_device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // TODO: Mix tracks, apply gain, routing, ecc.
                // Per ora: silenzio o semplice passthrough
                let _state = state.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = 0.0; // Silence for now
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);

        eprintln!("[Engine] Stream started!");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
            eprintln!("[Engine] Stream stopped!");
        }
        Ok(())
    }

    fn set_gain(&self, track: usize, gain: f32) {
        self.state.lock().unwrap().set_track_gain(track, gain);
    }

    fn set_mute(&self, track: usize, mute: bool) {
        self.state.lock().unwrap().set_track_mute(track, mute);
    }
}

fn send_response(response: &Response) {
    if let Ok(json) = serde_json::to_string(response) {
        println!("{}", json);
    }
}

fn main() -> Result<()> {
    eprintln!("[Engine] mMpro3 Audio Engine starting...");
    
    let mut engine = AudioEngine::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    eprintln!("[Engine] Ready. Waiting for commands on stdin...");

    // Loop: leggi comandi da stdin
    while let Some(Ok(line)) = lines.next() {
        let command: Result<Command, _> = serde_json::from_str(&line);

        match command {
            Ok(Command::Start { device_id, sample_rate, buffer_size }) => {
                match engine.start(device_id, sample_rate, buffer_size) {
                    Ok(_) => send_response(&Response::Ok {
                        message: "Engine started".to_string(),
                    }),
                    Err(e) => send_response(&Response::Error {
                        message: format!("Failed to start: {}", e),
                    }),
                }
            }
            Ok(Command::Stop) => {
                match engine.stop() {
                    Ok(_) => send_response(&Response::Ok {
                        message: "Engine stopped".to_string(),
                    }),
                    Err(e) => send_response(&Response::Error {
                        message: format!("Failed to stop: {}", e),
                    }),
                }
            }
            Ok(Command::SetGain { track, gain }) => {
                engine.set_gain(track, gain);
                send_response(&Response::Ok {
                    message: format!("Track {} gain set to {}", track, gain),
                });
            }
            Ok(Command::SetMute { track, mute }) => {
                engine.set_mute(track, mute);
                send_response(&Response::Ok {
                    message: format!("Track {} mute: {}", track, mute),
                });
            }
            Ok(Command::ListDevices) => {
                match engine.list_devices() {
                    Ok(devices) => send_response(&Response::Devices { devices }),
                    Err(e) => send_response(&Response::Error {
                        message: format!("Failed to list devices: {}", e),
                    }),
                }
            }
            Err(e) => {
                send_response(&Response::Error {
                    message: format!("Invalid command: {}", e),
                });
            }
        }
    }

    eprintln!("[Engine] Shutting down...");
    Ok(())
}

