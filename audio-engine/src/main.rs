use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Stream;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};

// Import our modules
mod audio_io;
mod equalizer;
mod file_player;
mod routing;
mod signal_gen;

use audio_io::{AudioIO, ChannelSelection, DeviceInfo};
use file_player::AudioFilePlayer;
use routing::Router;
use signal_gen::WaveformType;

/// Comando ricevuto da Electron via stdin
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Command {
    #[serde(rename = "start")]
    Start {
        input_device: Option<String>,
        output_device: Option<String>,
        sample_rate: Option<u32>,
    },
    #[serde(rename = "stop")]
    Stop,

    // Track source selection
    #[serde(rename = "set_track_source_input")]
    SetTrackSourceInput {
        track: usize,
        left_channel: u16,
        right_channel: u16,
    },
    #[serde(rename = "set_track_source_signal")]
    SetTrackSourceSignal {
        track: usize,
        waveform: String, // "sine", "square", "sawtooth", "triangle", "white", "pink"
        frequency: f32,
    },
    #[serde(rename = "set_track_source_file")]
    SetTrackSourceFile {
        track: usize,
        file_path: String,
    },
    #[serde(rename = "play_file")]
    PlayFile { track: usize },
    #[serde(rename = "pause_file")]
    PauseFile { track: usize },
    #[serde(rename = "stop_file")]
    StopFile { track: usize },
    #[serde(rename = "stop_all_files")]
    StopAllFiles,

    // Track controls
    #[serde(rename = "set_gain")]
    SetGain { track: usize, gain: f32 },
    #[serde(rename = "set_volume")]
    SetVolume { track: usize, volume: f32 },
    #[serde(rename = "set_mute")]
    SetMute { track: usize, mute: bool },
    #[serde(rename = "set_pan")]
    SetPan { track: usize, pan: f32 },
    
    // Track EQ controls
    #[serde(rename = "set_eq")]
    SetEQ {
        track: usize,
        low: f32,
        low_mid: f32,
        high_mid: f32,
        high: f32,
    },
    #[serde(rename = "set_eq_enabled")]
    SetEQEnabled {
        track: usize,
        enabled: bool,
    },

    // Master controls
    #[serde(rename = "set_master_gain")]
    SetMasterGain { gain: f32 },
    #[serde(rename = "set_master_mute")]
    SetMasterMute { mute: bool },
    #[serde(rename = "set_master_output_channels")]
    SetMasterOutputChannels {
        left_channel: u16,
        right_channel: u16,
    },

    // Device management
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
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "levels")]
    Levels {
        tracks: Vec<TrackLevels>,
        master_l: f32,
        master_r: f32,
    },
}

#[derive(Debug, Serialize)]
struct TrackLevels {
    track: usize,
    level_l: f32,
    level_r: f32,
}

/// Engine audio principale
struct AudioEngine {
    audio_io: AudioIO,
    router: Arc<Mutex<Router>>,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    sample_rate: u32,
}

impl AudioEngine {
    fn new() -> Self {
        let audio_io = AudioIO::new();
        let router = Arc::new(Mutex::new(Router::new(1))); // Start with 1 track

        eprintln!("[Engine] Audio engine initialized");

        Self {
            audio_io,
            router,
            input_stream: None,
            output_stream: None,
            sample_rate: 48000,
        }
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        self.audio_io.list_devices()
    }

    fn start(
        &mut self,
        input_device_name: Option<String>,
        output_device_name: Option<String>,
        sample_rate: Option<u32>,
    ) -> Result<()> {
        if self.input_stream.is_some() || self.output_stream.is_some() {
            return Ok(()); // Already running
        }

        // Get devices
        let input_device = if let Some(name) = input_device_name {
            self.audio_io.find_device_by_name(&name, true)?
        } else {
            self.audio_io.default_input_device()?
        };

        let output_device = if let Some(name) = output_device_name {
            self.audio_io.find_device_by_name(&name, false)?
        } else {
            self.audio_io.default_output_device()?
        };

        eprintln!("[Engine] Input device: {:?}", input_device.name());
        eprintln!("[Engine] Output device: {:?}", output_device.name());

        // Get configs
        let input_config = self.audio_io.get_supported_config(&input_device, true, sample_rate)?;
        let output_config = self.audio_io.get_supported_config(&output_device, false, sample_rate)?;

        self.sample_rate = output_config.sample_rate.0;
        
        eprintln!("[Engine] New sample rate: {}", self.sample_rate);
        
        // Update sample rate for all active file players and equalizers
        {
            let mut router = self.router.lock().unwrap();
            for track in router.tracks.iter_mut() {
                // Update file player sample rate
                if let Some(ref mut player) = track.file_player {
                    let old_rate = player.output_sample_rate;
                    player.set_output_sample_rate(self.sample_rate);
                    eprintln!("[Engine] Track {} file player: updated output_sample_rate {} → {} (file_rate={})", 
                        track.id, old_rate, player.output_sample_rate, player.sample_rate);
                }
                // Update equalizer sample rate
                track.equalizer.set_sample_rate(self.sample_rate as f32);
            }
        }

        eprintln!("[Engine] Input config: {:?}", input_config);
        eprintln!("[Engine] Output config: {:?}", output_config);

        let input_channels = input_config.channels as usize;
        let output_channels = output_config.channels as usize;

        eprintln!(
            "[Engine] Channels: Input={}, Output={}",
            input_channels, output_channels
        );

        let err_fn = |err| eprintln!("[Engine] Stream error: {}", err);

        // Clone router for callbacks
        let router_output = Arc::clone(&self.router);

        // Meter update counter and interval (send levels every ~50ms at 48kHz = 2400 frames)
        let meter_update_frames = Arc::new(Mutex::new(0_usize));
        let meter_interval = 2400_usize;

        // === INPUT STREAM: capture audio ===
        // For now, we capture but don't use it directly (tracks may use it later)
        let input_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let input_buffer_clone = Arc::clone(&input_buffer);

        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = input_buffer_clone.lock().unwrap();
                buffer.clear();
                buffer.extend_from_slice(data);
            },
            err_fn,
            None,
        )?;

        // === OUTPUT STREAM: process and output audio ===
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let input_buf = input_buffer.lock().unwrap();
                let frames = data.len() / output_channels;

                // Acquire lock, process audio, release lock quickly
                let levels_to_send = {
                    let mut router = router_output.lock().unwrap();
                    
                    // Process all frames
                    for frame_idx in 0..frames {
                        // Prepare input frame if available
                        let input_frame: Option<Vec<f32>> = if !input_buf.is_empty() && frame_idx < input_buf.len() / input_channels {
                            let start = frame_idx * input_channels;
                            let end = start + input_channels;
                            Some(input_buf[start..end].to_vec())
                        } else {
                            None
                        };

                        // Process one frame through router
                        let (left, right) = router.process_frame(input_frame.as_deref());

                        // Write to output
                        let out_frame_start = frame_idx * output_channels;
                        if output_channels >= 2 {
                            data[out_frame_start] = left.clamp(-1.0, 1.0);
                            data[out_frame_start + 1] = right.clamp(-1.0, 1.0);
                            // Extra channels: silence
                            for ch in 2..output_channels {
                                data[out_frame_start + ch] = 0.0;
                            }
                        } else if output_channels == 1 {
                            // Mono: mix L+R
                            data[out_frame_start] = ((left + right) * 0.5).clamp(-1.0, 1.0);
                        }
                    }

                    // Check if we need to send meter updates
                    let mut counter = meter_update_frames.lock().unwrap();
                    *counter += frames;
                    
                    if *counter >= meter_interval {
                        *counter = 0;
                        
                        // Copy levels data while we have the lock
                        let track_levels: Vec<TrackLevels> = router.tracks.iter()
                            .map(|t| TrackLevels {
                                track: t.id,
                                level_l: t.level_l,
                                level_r: t.level_r,
                            })
                            .collect();
                        
                        let master_l = router.master.level_l;
                        let master_r = router.master.level_r;
                        
                        // Reset peak levels after reading
                        router.master.reset_levels();
                        for track in router.tracks.iter_mut() {
                            track.reset_levels();
                        }
                        
                        // Return data to serialize outside the lock
                        Some((track_levels, master_l, master_r))
                    } else {
                        None
                    }
                }; // Lock is released here
                
                drop(input_buf); // Release input buffer lock
                
                // Send meter updates outside the lock
                if let Some((track_levels, master_l, master_r)) = levels_to_send {
                    let response = Response::Levels {
                        tracks: track_levels,
                        master_l,
                        master_r,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&response) {
                        println!("{}", json);
                    }
                }
            },
            err_fn,
            None,
        )?;

        // Start streams
        input_stream.play()?;
        output_stream.play()?;

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);

        eprintln!("[Engine] Streams started!");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.input_stream.take() {
            drop(stream);
            eprintln!("[Engine] Input stream stopped");
        }
        if let Some(stream) = self.output_stream.take() {
            drop(stream);
            eprintln!("[Engine] Output stream stopped");
        }
        
        Ok(())
    }

    // Track source commands
    fn set_track_source_input(&self, track: usize, left_ch: u16, right_ch: u16) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            let channel_sel = ChannelSelection::new(left_ch, right_ch);
            t.set_audio_input(channel_sel);
            eprintln!("[Track {}] Source: Audio Input (L={}, R={})", track, left_ch, right_ch);
            Ok(())
        } else {
            Err(anyhow!("Track {} not found", track))
        }
    }

    fn set_track_source_signal(&self, track: usize, waveform: &str, frequency: f32) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            let wave = match waveform.to_lowercase().as_str() {
                "sine" => WaveformType::Sine,
                "square" => WaveformType::Square,
                "sawtooth" => WaveformType::Sawtooth,
                "triangle" => WaveformType::Triangle,
                "white" => WaveformType::WhiteNoise,
                "pink" => WaveformType::PinkNoise,
                _ => return Err(anyhow!("Unknown waveform: {}", waveform)),
            };
            t.set_signal_generator(wave, frequency, self.sample_rate as f32);
            eprintln!("[Track {}] Source: Signal ({:?} @ {}Hz)", track, wave, frequency);
            Ok(())
        } else {
            Err(anyhow!("Track {} not found", track))
        }
    }

    fn set_track_source_file(&self, track: usize, file_path: &str) -> Result<()> {
        eprintln!("[Track {}] Attempting to load file: {}", track, file_path);
        let mut router = self.router.lock().unwrap();
        eprintln!("[Track {}] Router has {} tracks", track, router.tracks.len());
        
        if let Some(t) = router.get_track_mut(track) {
            eprintln!("[Track {}] Track found in router", track);
            let mut player = AudioFilePlayer::new();
            player.set_output_sample_rate(self.sample_rate);
            eprintln!("[Track {}] Loading file with output sample rate: {}", track, self.sample_rate);
            
            match player.load_file(file_path) {
                Ok(_) => {
                    // Debug: log loaded samples info
                    let sample_count = player.samples.len();
                    let duration_frames = sample_count / player.channels as usize;
                    let duration_sec = duration_frames as f32 / player.sample_rate as f32;
                    eprintln!("[Track {}] File loaded: {} samples, {} channels, {} Hz, {:.2}s duration", 
                        track, sample_count, player.channels, player.sample_rate, duration_sec);
                    
                    t.set_file_player(player);
                    eprintln!("[Track {}] Source: File ({})", track, file_path);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[Track {}] ERROR loading file: {}", track, e);
                    Err(e)
                }
            }
        } else {
            eprintln!("[Track {}] ERROR: track not found (router has {} tracks)", track, router.tracks.len());
            Err(anyhow!("Track {} not found", track))
        }
    }

    fn play_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.play_file(self.sample_rate)
        } else {
            Err(anyhow!("Track {} not found", track))
        }
    }

    fn pause_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.pause_file()
        } else {
            Err(anyhow!("Track {} not found", track))
        }
    }

    fn stop_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.stop_file()
        } else {
            Err(anyhow!("Track {} not found", track))
        }
    }

    fn stop_all_files(&self) {
        let mut router = self.router.lock().unwrap();
        router.stop_all_files();
    }

    // Track controls
    fn set_gain(&self, track: usize, gain: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.gain = gain.max(0.0); // No upper limit, but can't be negative
            let gain_db = if gain > 0.0 { 20.0 * gain.log10() } else { -90.0 };
            eprintln!("[Track {}] Gain: {:.3} ({:.1} dB)", track, t.gain, gain_db);
        }
    }

    fn set_volume(&self, track: usize, volume: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.volume = volume.max(0.0); // No upper limit, but can't be negative
            let volume_db = if volume > 0.0 { 20.0 * volume.log10() } else { -90.0 };
            eprintln!("[Track {}] Volume: {:.3} ({:.1} dB)", track, t.volume, volume_db);
        }
    }

    fn set_mute(&self, track: usize, mute: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.mute = mute;
            eprintln!("[Track {}] Mute: {}", track, mute);
        }
    }

    fn set_pan(&self, track: usize, pan: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.pan = pan.clamp(-1.0, 1.0);
            eprintln!("[Track {}] Pan: {}", track, t.pan);
        }
    }

    // Track EQ controls
    fn set_eq(&self, track: usize, low: f32, low_mid: f32, high_mid: f32, high: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.set_eq(low, low_mid, high_mid, high);
        }
    }

    fn set_eq_enabled(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.set_eq_enabled(enabled);
        }
    }

    // Master controls
    fn set_master_gain(&self, gain: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.gain = gain.max(0.0); // No upper limit
        let gain_db = if gain > 0.0 { 20.0 * gain.log10() } else { -90.0 };
        eprintln!("[Master] Gain: {:.3} ({:.1} dB)", router.master.gain, gain_db);
    }

    fn set_master_mute(&self, mute: bool) {
        let mut router = self.router.lock().unwrap();
        router.master.mute = mute;
        eprintln!("[Master] Mute: {}", mute);
    }

    fn set_master_output_channels(&self, left_ch: u16, right_ch: u16) {
        let mut router = self.router.lock().unwrap();
        router.master.output_channel_selection = ChannelSelection::new(left_ch, right_ch);
        eprintln!("[Master] Output channels: L={}, R={}", left_ch, right_ch);
    }

    /// Handle a command and return the appropriate response
    fn handle_command(&mut self, command: Command) -> Response {
        match command {
            Command::Start {
                input_device,
                output_device,
                sample_rate,
            } => match self.start(input_device, output_device, sample_rate) {
                Ok(_) => Response::Started,
                Err(e) => Response::Error {
                    message: format!("Start failed: {}", e),
                },
            },
            Command::Stop => match self.stop() {
                Ok(_) => Response::Stopped,
                Err(e) => Response::Error {
                    message: format!("Stop failed: {}", e),
                },
            },
            Command::SetTrackSourceInput {
                track,
                left_channel,
                right_channel,
            } => match self.set_track_source_input(track, left_channel, right_channel) {
                Ok(_) => Response::Ok {
                    message: format!("Track {} source set to input", track),
                },
                Err(e) => Response::Error {
                    message: e.to_string(),
                },
            },
            Command::SetTrackSourceSignal {
                track,
                waveform,
                frequency,
            } => match self.set_track_source_signal(track, &waveform, frequency) {
                Ok(_) => Response::Ok {
                    message: format!("Track {} source set to signal", track),
                },
                Err(e) => Response::Error {
                    message: e.to_string(),
                },
            },
            Command::SetTrackSourceFile { track, file_path } => {
                eprintln!("[Engine] Executing SetTrackSourceFile for track {}", track);
                match self.set_track_source_file(track, &file_path) {
                    Ok(_) => {
                        eprintln!("[Engine] SetTrackSourceFile succeeded for track {}", track);
                        Response::Ok {
                            message: format!("Track {} source set to file", track),
                        }
                    }
                    Err(e) => {
                        eprintln!("[Engine] SetTrackSourceFile FAILED for track {}: {}", track, e);
                        Response::Error {
                            message: e.to_string(),
                        }
                    }
                }
            }
            Command::PlayFile { track } => {
                eprintln!("[Engine] Executing PlayFile for track {}", track);
                match self.play_file(track) {
                    Ok(_) => {
                        eprintln!("[Engine] PlayFile succeeded for track {}", track);
                        Response::Ok {
                            message: format!("Track {} playing", track),
                        }
                    }
                    Err(e) => {
                        eprintln!("[Engine] PlayFile FAILED for track {}: {}", track, e);
                        Response::Error {
                            message: e.to_string(),
                        }
                    }
                }
            }
            Command::PauseFile { track } => match self.pause_file(track) {
                Ok(_) => Response::Ok {
                    message: format!("Track {} paused", track),
                },
                Err(e) => Response::Error {
                    message: e.to_string(),
                },
            },
            Command::StopFile { track } => match self.stop_file(track) {
                Ok(_) => Response::Ok {
                    message: format!("Track {} stopped", track),
                },
                Err(e) => Response::Error {
                    message: e.to_string(),
                },
            },
            Command::StopAllFiles => {
                self.stop_all_files();
                Response::Ok {
                    message: "All file players stopped".to_string(),
                }
            }
            Command::SetGain { track, gain } => {
                self.set_gain(track, gain);
                Response::Ok {
                    message: format!("Track {} gain: {}", track, gain),
                }
            }
            Command::SetVolume { track, volume } => {
                self.set_volume(track, volume);
                Response::Ok {
                    message: format!("Track {} volume: {}", track, volume),
                }
            }
            Command::SetMute { track, mute } => {
                self.set_mute(track, mute);
                Response::Ok {
                    message: format!("Track {} mute: {}", track, mute),
                }
            }
            Command::SetPan { track, pan } => {
                self.set_pan(track, pan);
                Response::Ok {
                    message: format!("Track {} pan: {}", track, pan),
                }
            }
            Command::SetEQ {
                track,
                low,
                low_mid,
                high_mid,
                high,
            } => {
                self.set_eq(track, low, low_mid, high_mid, high);
                Response::Ok {
                    message: format!("Track {} EQ set", track),
                }
            }
            Command::SetEQEnabled { track, enabled } => {
                self.set_eq_enabled(track, enabled);
                Response::Ok {
                    message: format!("Track {} EQ enabled: {}", track, enabled),
                }
            }
            Command::SetMasterGain { gain } => {
                self.set_master_gain(gain);
                Response::Ok {
                    message: format!("Master gain: {}", gain),
                }
            }
            Command::SetMasterMute { mute } => {
                self.set_master_mute(mute);
                Response::Ok {
                    message: format!("Master mute: {}", mute),
                }
            }
            Command::SetMasterOutputChannels {
                left_channel,
                right_channel,
            } => {
                self.set_master_output_channels(left_channel, right_channel);
                Response::Ok {
                    message: format!("Master output: L={}, R={}", left_channel, right_channel),
                }
            }
            Command::ListDevices => match self.list_devices() {
                Ok(devices) => Response::Devices { devices },
                Err(e) => Response::Error {
                    message: format!("List devices failed: {}", e),
                },
            },
        }
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

    // Loop: read commands from stdin
    while let Some(Ok(line)) = lines.next() {
        match serde_json::from_str::<Command>(&line) {
            Ok(command) => {
                eprintln!("[Engine] Command: {:?}", command);
                let response = engine.handle_command(command);
                send_response(&response);
            }
            Err(e) => {
                eprintln!("[Engine] Failed to parse command: {}", e);
                eprintln!("[Engine] Raw input: {}", line);
                send_response(&Response::Error {
                    message: format!("Invalid command: {}", e),
                });
            }
        }
    }

    eprintln!("[Engine] Shutting down...");
    Ok(())
}

