use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, Stream, StreamConfig};
use ringbuf::HeapRb;
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
    #[serde(rename = "set_eq")]
    SetEQ {
        track: usize,
        low: f32,
        mid: f32,
        high: f32,
    },
    #[serde(rename = "set_compressor")]
    SetCompressor {
        track: usize,
        enabled: bool,
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
    },
    #[serde(rename = "set_gate")]
    SetGate {
        track: usize,
        enabled: bool,
        threshold: f32,
        range: f32,
        attack: f32,
        release: f32,
    },
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

#[derive(Clone)]
struct EQState {
    low: f32,   // -12 to +12 dB
    mid: f32,   // -12 to +12 dB
    high: f32,  // -12 to +12 dB
}

#[derive(Clone)]
struct CompressorState {
    enabled: bool,
    threshold: f32,  // -60 to 0 dB
    ratio: f32,      // 1 to 20:1
    attack: f32,     // 0 to 1 s
    release: f32,    // 0 to 4 s
}

#[derive(Clone)]
struct GateState {
    enabled: bool,
    threshold: f32,  // -80 to 0 dB
    range: f32,      // -60 to 0 dB (attenuation when closed)
    attack: f32,     // 0.001 to 0.1 s
    release: f32,    // 0.01 to 2 s
}

struct TrackState {
    gain: f32,
    mute: bool,
    eq: EQState,
    compressor: CompressorState,
    gate: GateState,
}

impl MixerState {
    fn new(num_tracks: usize) -> Self {
        Self {
            tracks: (0..num_tracks)
                .map(|_| TrackState {
                    gain: 1.0,
                    mute: false,
                    eq: EQState {
                        low: 0.0,
                        mid: 0.0,
                        high: 0.0,
                    },
                    compressor: CompressorState {
                        enabled: false,
                        threshold: -20.0,
                        ratio: 4.0,
                        attack: 0.1,
                        release: 0.25,
                    },
                    gate: GateState {
                        enabled: false,
                        threshold: -40.0,
                        range: -30.0,
                        attack: 0.005,
                        release: 0.2,
                    },
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
    
    fn set_track_eq(&mut self, track: usize, low: f32, mid: f32, high: f32) {
        if let Some(t) = self.tracks.get_mut(track) {
            t.eq.low = low.clamp(-12.0, 12.0);
            t.eq.mid = mid.clamp(-12.0, 12.0);
            t.eq.high = high.clamp(-12.0, 12.0);
        }
    }
    
    fn set_track_compressor(&mut self, track: usize, enabled: bool, threshold: f32, ratio: f32, attack: f32, release: f32) {
        if let Some(t) = self.tracks.get_mut(track) {
            t.compressor.enabled = enabled;
            t.compressor.threshold = threshold.clamp(-60.0, 0.0);
            t.compressor.ratio = ratio.clamp(1.0, 20.0);
            t.compressor.attack = attack.clamp(0.0, 1.0);
            t.compressor.release = release.clamp(0.0, 4.0);
        }
    }
    
    fn set_track_gate(&mut self, track: usize, enabled: bool, threshold: f32, range: f32, attack: f32, release: f32) {
        if let Some(t) = self.tracks.get_mut(track) {
            t.gate.enabled = enabled;
            t.gate.threshold = threshold.clamp(-80.0, 0.0);
            t.gate.range = range.clamp(-60.0, 0.0);
            t.gate.attack = attack.clamp(0.001, 0.1);
            t.gate.release = release.clamp(0.01, 2.0);
        }
    }
}

/// Engine audio principale
struct AudioEngine {
    host: Host,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    state: Arc<Mutex<MixerState>>,
}

impl AudioEngine {
    fn new() -> Self {
        // Usa host di default (CoreAudio su macOS, WASAPI su Windows)
        let host = cpal::default_host();
        
        eprintln!("[Engine] Using host: {:?}", host.id());

        Self {
            host,
            input_stream: None,
            output_stream: None,
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
        if self.input_stream.is_some() || self.output_stream.is_some() {
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
        let mut input_config: StreamConfig = input_device.default_input_config()?.into();
        let mut output_config: StreamConfig = output_device.default_output_config()?.into();
        
        if let Some(sr) = sample_rate {
            input_config.sample_rate = cpal::SampleRate(sr);
            output_config.sample_rate = cpal::SampleRate(sr);
        }
        if let Some(bs) = buffer_size {
            input_config.buffer_size = cpal::BufferSize::Fixed(bs);
            output_config.buffer_size = cpal::BufferSize::Fixed(bs);
        }

        eprintln!("[Engine] Input config: {:?}", input_config);
        eprintln!("[Engine] Output config: {:?}", output_config);

        let input_channels = input_config.channels as usize;
        let output_channels = output_config.channels as usize;

        eprintln!("[Engine] Input channels: {}, Output channels: {}", input_channels, output_channels);

        // Crea ring buffer: 8192 samples per canale (circa 170ms @ 48kHz)
        // Buffer interleaved: total_size = 8192 * max(input_channels, output_channels)
        let ring_buf_samples = 8192 * input_channels.max(output_channels);
        let ring = HeapRb::<f32>::new(ring_buf_samples);
        let (mut producer, consumer) = ring.split();

        // Pre-fill con silenzio per evitare underrun all'avvio
        for _ in 0..(ring_buf_samples / 2) {
            let _ = producer.push(0.0);
        }

        let producer = Arc::new(Mutex::new(producer));
        let consumer = Arc::new(Mutex::new(consumer));

        let err_fn = |err| eprintln!("[Engine] Stream error: {}", err);

        // === INPUT STREAM: legge dal device, scrive nel ring buffer ===
        let producer_clone = Arc::clone(&producer);
        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut prod = producer_clone.lock().unwrap();
                // Scrivi tutti i sample nel ring buffer
                for &sample in data.iter() {
                    // Se ring buffer è pieno, sovrascrivi (drop vecchi sample)
                    if prod.push(sample).is_err() {
                        // Buffer pieno, skip sample (in produzione: metriche xrun)
                    }
                }
            },
            err_fn,
            None,
        )?;

        // === OUTPUT STREAM: legge dal ring buffer, applica routing, scrive al device ===
        let consumer_clone = Arc::clone(&consumer);
        let state = Arc::clone(&self.state);
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut cons = consumer_clone.lock().unwrap();
                let state = state.lock().unwrap();

                // Buffer temporaneo per input (legge dal ring buffer)
                let frames = data.len() / output_channels;
                let mut input_buf = vec![0.0f32; frames * input_channels];

                // Leggi dal ring buffer
                for sample in input_buf.iter_mut() {
                    *sample = cons.pop().unwrap_or(0.0);
                }

                // Routing e mixing:
                // - Ogni traccia prende il suo canale input corrispondente
                // - Applica gain e mute
                // - Mixa tutto su master stereo (o multi-canale)
                
                // Per semplicità: mix tutte le tracce su L/R stereo
                for frame_idx in 0..frames {
                    let mut left_mix = 0.0f32;
                    let mut right_mix = 0.0f32;

                    // Mix tracks: ogni traccia contribuisce al mix stereo
                    for (track_idx, track) in state.tracks.iter().enumerate() {
                        if track.mute {
                            continue;
                        }

                        // Prendi il sample corrispondente dal canale input
                        // Se abbiamo meno canali input che tracce, fa cycling
                        let input_channel = track_idx % input_channels;
                        let sample_idx = frame_idx * input_channels + input_channel;
                        let sample = if sample_idx < input_buf.len() {
                            input_buf[sample_idx]
                        } else {
                            0.0
                        };

                        // Applica gain della traccia
                        let processed = sample * track.gain;

                        // Pan: per ora hard-pan left/right alternato (TODO: implementare pan knob)
                        if track_idx % 2 == 0 {
                            left_mix += processed;
                        } else {
                            right_mix += processed;
                        }
                    }

                    // Scrivi su output (stereo o multi-canale)
                    let out_frame_start = frame_idx * output_channels;
                    
                    if output_channels >= 2 {
                        // Stereo o multi-canale: L/R sui primi due canali
                        data[out_frame_start] = left_mix.clamp(-1.0, 1.0);
                        data[out_frame_start + 1] = right_mix.clamp(-1.0, 1.0);
                        
                        // Eventuali canali extra: silenzio
                        for ch in 2..output_channels {
                            data[out_frame_start + ch] = 0.0;
                        }
                    } else if output_channels == 1 {
                        // Mono: mix L+R
                        data[out_frame_start] = ((left_mix + right_mix) * 0.5).clamp(-1.0, 1.0);
                    }
                }
            },
            err_fn,
            None,
        )?;

        // Avvia entrambi gli stream
        input_stream.play()?;
        output_stream.play()?;

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);

        eprintln!("[Engine] Duplex streams started!");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.input_stream.take() {
            drop(stream);
            eprintln!("[Engine] Input stream stopped!");
        }
        if let Some(stream) = self.output_stream.take() {
            drop(stream);
            eprintln!("[Engine] Output stream stopped!");
        }
        Ok(())
    }

    fn set_gain(&self, track: usize, gain: f32) {
        self.state.lock().unwrap().set_track_gain(track, gain);
    }

    fn set_mute(&self, track: usize, mute: bool) {
        self.state.lock().unwrap().set_track_mute(track, mute);
    }
    
    fn set_eq(&self, track: usize, low: f32, mid: f32, high: f32) {
        self.state.lock().unwrap().set_track_eq(track, low, mid, high);
    }
    
    fn set_compressor(&self, track: usize, enabled: bool, threshold: f32, ratio: f32, attack: f32, release: f32) {
        self.state.lock().unwrap().set_track_compressor(track, enabled, threshold, ratio, attack, release);
    }
    
    fn set_gate(&self, track: usize, enabled: bool, threshold: f32, range: f32, attack: f32, release: f32) {
        self.state.lock().unwrap().set_track_gate(track, enabled, threshold, range, attack, release);
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
        
        if let Ok(ref cmd) = command {
            eprintln!("[Engine] Received command: {:?}", cmd);
        }

        match command {
            Ok(Command::Start { device_id, sample_rate, buffer_size }) => {
                match engine.start(device_id, sample_rate, buffer_size) {
                    Ok(_) => {
                        send_response(&Response::Ok {
                            message: "Engine started".to_string(),
                        });
                        // Send started event
                        println!("{{\"type\":\"started\"}}");
                    },
                    Err(e) => send_response(&Response::Error {
                        message: format!("Failed to start: {}", e),
                    }),
                }
            }
            Ok(Command::Stop) => {
                match engine.stop() {
                    Ok(_) => {
                        send_response(&Response::Ok {
                            message: "Engine stopped".to_string(),
                        });
                        // Send stopped event
                        println!("{{\"type\":\"stopped\"}}");
                    },
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
            Ok(Command::SetEQ { track, low, mid, high }) => {
                engine.set_eq(track, low, mid, high);
                send_response(&Response::Ok {
                    message: format!("Track {} EQ set", track),
                });
            }
            Ok(Command::SetCompressor { track, enabled, threshold, ratio, attack, release }) => {
                engine.set_compressor(track, enabled, threshold, ratio, attack, release);
                send_response(&Response::Ok {
                    message: format!("Track {} compressor {}", track, if enabled { "enabled" } else { "disabled" }),
                });
            }
            Ok(Command::SetGate { track, enabled, threshold, range, attack, release }) => {
                engine.set_gate(track, enabled, threshold, range, attack, release);
                send_response(&Response::Ok {
                    message: format!("Track {} gate {}", track, if enabled { "enabled" } else { "disabled" }),
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

