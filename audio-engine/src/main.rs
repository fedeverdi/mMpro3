use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Stream;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Import our modules
mod audio_io;
mod delay;
mod limiter;
mod compressor;
mod equalizer;
mod file_player;
mod gate;
mod reverb;
mod routing;
mod signal_gen;
mod track;

use audio_io::{AudioIO, ChannelSelection, DeviceInfo};
use routing::Router;

/// Parametric filter specification from frontend
#[derive(Debug, Deserialize, Clone)]
pub struct ParametricFilter {
    #[serde(rename = "type")]
    pub filter_type: String, // "lowshelf", "highshelf", "peaking", "lowpass", "highpass"
    pub frequency: f32,
    pub gain: f32,
    pub q: f32,
}

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
    #[serde(rename = "set_route_to_master")]
    SetRouteToMaster { track: usize, route: bool },
    #[serde(rename = "set_pan")]
    SetPan { track: usize, pan: f32 },
    #[serde(rename = "set_track_pad")]
    SetTrackPad { track: usize, enabled: bool },
    #[serde(rename = "set_track_hpf")]
    SetTrackHPF { track: usize, enabled: bool },
    
    // Track dynamics controls
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
        range: f32,       // Attenuation range in dB (not ratio like compressor)
        attack: f32,
        release: f32,
    },
    
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
    
    // Parametric EQ controls
    #[serde(rename = "set_parametric_eq_filters")]
    SetParametricEQFilters {
        track: usize,
        filters: Vec<ParametricFilter>,
    },
    #[serde(rename = "set_parametric_eq_enabled")]
    SetParametricEQEnabled {
        track: usize,
        enabled: bool,
    },
    #[serde(rename = "clear_parametric_eq")]
    ClearParametricEQ {
        track: usize,
    },

    // Master controls
    #[serde(rename = "set_master_gain")]
    SetMasterGain { gain: f32 },
    #[serde(rename = "set_master_mute")]
    SetMasterMute { mute: bool },
    #[serde(rename = "set_master_parametric_eq_filters")]
    SetMasterParametricEQFilters {
        filters: Vec<ParametricFilter>,
    },
    #[serde(rename = "set_master_parametric_eq_enabled")]
    SetMasterParametricEQEnabled {
        enabled: bool,
    },
    #[serde(rename = "clear_master_parametric_eq")]
    ClearMasterParametricEQ,
    #[serde(rename = "set_master_output_channels")]
    SetMasterOutputChannels {
        left_channel: u16,
        right_channel: u16,
    },

    // Master FX controls
    #[serde(rename = "set_master_compressor")]
    SetMasterCompressor {
        enabled: bool,
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
    },
    #[serde(rename = "set_master_limiter")]
    SetMasterLimiter {
        enabled: bool,
        ceiling: f32,
        release: f32,
    },
    #[serde(rename = "set_master_delay")]
    SetMasterDelay {
        enabled: bool,
        time_l: f32,
        time_r: f32,
        feedback: f32,
        mix: f32,
    },
    #[serde(rename = "set_master_reverb")]
    SetMasterReverb {
        enabled: bool,
        room_size: f32,
        damping: f32,
        wet: f32,
        width: f32,
    },

    // Subgroup controls
    #[serde(rename = "add_subgroup")]
    AddSubgroup,
    #[serde(rename = "remove_subgroup")]
    RemoveSubgroup { subgroup: usize },
    #[serde(rename = "set_subgroup_gain")]
    SetSubgroupGain { subgroup: usize, gain: f32 },
    #[serde(rename = "set_subgroup_mute")]
    SetSubgroupMute { subgroup: usize, mute: bool },
    #[serde(rename = "set_subgroup_output_enabled")]
    SetSubgroupOutputEnabled { subgroup: usize, enabled: bool },
    #[serde(rename = "set_subgroup_route_to_master")]
    SetSubgroupRouteToMaster { subgroup: usize, route: bool },
    #[serde(rename = "set_subgroup_output_channels")]
    SetSubgroupOutputChannels {
        subgroup: usize,
        left_channel: u16,
        right_channel: u16,
    },
    #[serde(rename = "set_track_route_to_subgroup")]
    SetTrackRouteToSubgroup {
        track: usize,
        subgroup: usize,
        route: bool, // true = add to routing, false = remove from routing
    },

    // Aux bus controls
    #[serde(rename = "set_track_aux_send")]
    SetTrackAuxSend {
        track: usize,
        aux: usize,
        level: f32,
        pre_fader: bool,
        muted: bool,
    },
    #[serde(rename = "set_aux_bus_gain")]
    SetAuxBusGain {
        aux: usize,
        gain: f32,
    },
    #[serde(rename = "set_aux_bus_mute")]
    SetAuxBusMute {
        aux: usize,
        mute: bool,
    },
    #[serde(rename = "set_aux_bus_reverb")]
    SetAuxBusReverb {
        aux: usize,
        enabled: bool,
        room_size: f32,
        damping: f32,
        wet: f32,
        width: f32,
    },
    #[serde(rename = "set_aux_bus_delay")]
    SetAuxBusDelay {
        aux: usize,
        enabled: bool,
        time: f32,
        feedback: f32,
        mix: f32,
    },
    #[serde(rename = "set_aux_bus_route_to_master")]
    SetAuxBusRouteToMaster {
        aux: usize,
        route: bool,
    },
    #[serde(rename = "set_aux_bus_output_enabled")]
    SetAuxBusOutputEnabled {
        aux: usize,
        enabled: bool,
    },
    #[serde(rename = "set_aux_bus_output_channels")]
    SetAuxBusOutputChannels {
        aux: usize,
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
    #[serde(rename = "subgroup_created")]
    SubgroupCreated { id: usize },
    #[serde(rename = "levels")]
    Levels {
        tracks: Vec<TrackLevels>,
        subgroups: Vec<SubgroupLevels>,
        master_l: f32,
        master_r: f32,
    },
    #[serde(rename = "fft")]
    FFTData {
        bins_left: Vec<f32>,
        bins_right: Vec<f32>,
        sample_rate: u32,
    },
    #[serde(rename = "performance")]
    PerformanceStats {
        buffer_size: usize,
        latency_ms: f32,
        avg_process_ms: f32,
        cpu_percent: f32,
        min_process_ms: f32,
        max_process_ms: f32,
    },
}

#[derive(Debug, Serialize)]
struct TrackLevels {
    track: usize,
    level_l: f32,
    level_r: f32,
    waveform: Vec<f32>, // Waveform samples (downsampled to ~128 samples)
    compressor_input_db: f32,
    compressor_reduction_db: f32,
    gate_input_db: f32,
    gate_attenuation_db: f32,
}

#[derive(Debug, Serialize)]
struct SubgroupLevels {
    subgroup: usize,
    level_l: f32,
    level_r: f32,
}

/// Performance statistics for audio processing
#[derive(Debug, Clone)]
struct PerformanceStats {
    buffer_count: usize,
    total_process_time_us: u128,
    min_process_time_us: u128,
    max_process_time_us: u128,
    last_log_time: Option<Instant>,
}

impl PerformanceStats {
    fn new() -> Self {
        Self {
            buffer_count: 0,
            total_process_time_us: 0,
            min_process_time_us: u128::MAX,
            max_process_time_us: 0,
            last_log_time: None,
        }
    }

    fn record(&mut self, duration_us: u128) {
        self.buffer_count += 1;
        self.total_process_time_us += duration_us;
        self.min_process_time_us = self.min_process_time_us.min(duration_us);
        self.max_process_time_us = self.max_process_time_us.max(duration_us);
    }

    fn should_log(&mut self) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_log_time {
            if now.duration_since(last).as_secs() >= 2 {
                self.last_log_time = Some(now);
                true
            } else {
                false
            }
        } else {
            self.last_log_time = Some(now);
            false // Don't log on first call
        }
    }

    fn reset(&mut self) {
        self.buffer_count = 0;
        self.total_process_time_us = 0;
        self.min_process_time_us = u128::MAX;
        self.max_process_time_us = 0;
    }
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
        let router = Arc::new(Mutex::new(Router::new(24))); // Support up to 24 tracks

        Self {
            audio_io,
            router,
            input_stream: None,
            output_stream: None,
            sample_rate: 44100,
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

        // Get configs with 256 frame buffer for low latency
        // If sample_rate is None, will automatically use the highest supported rate for each device
        let input_config = self.audio_io.get_supported_config(&input_device, true, sample_rate, Some(256))?;
        let output_config = self.audio_io.get_supported_config(&output_device, false, sample_rate, Some(256))?;

        self.sample_rate = output_config.sample_rate.0;
        
        // Log buffer configuration
        let buffer_size = output_config.buffer_size;
        match buffer_size {
            cpal::BufferSize::Fixed(size) => {
                let latency_ms = (size as f32 / self.sample_rate as f32) * 1000.0;
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
                eprintln!("[Engine] CURRENT: {} frames → {:.2}ms latency @ {}Hz", 
                    size, latency_ms, self.sample_rate);
                eprintln!("[Engine] ───────────────────────────────────────────────────");
                eprintln!("[Engine] Buffer size options (@ {}Hz):", self.sample_rate);
                
                let options = [(128, "ultra-low, può causare glitch"), 
                               (256, "bassa, raccomandato per live"), 
                               (512, "default, bilanciato"), 
                               (1024, "alta, più stabile")];
                
                for (frames, desc) in options.iter() {
                    let ms = (*frames as f32 / self.sample_rate as f32) * 1000.0;
                    let marker = if *frames == size { " ← YOU ARE HERE" } else { "" };
                    eprintln!("[Engine]   {} frames → {:5.2}ms ({}){}", 
                        frames, ms, desc, marker);
                }
                
                eprintln!("[Engine] ───────────────────────────────────────────────────");
                eprintln!("[Engine] Buffer impostato dall'applicazione a 256 frames");
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
            },
            cpal::BufferSize::Default => {
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
                eprintln!("[Engine] Buffer size: DEFAULT (system controlled)");
                eprintln!("[Engine] To optimize latency: macOS → Audio MIDI Setup");
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
            },
        }
                
        // Update sample rate for all active file players and equalizers
        {
            let mut router = self.router.lock().unwrap();
            for track in router.tracks.iter_mut() {
                track.set_sample_rate(self.sample_rate as f32);
            }
            
            // Update aux buses
            for aux_bus in router.aux_buses.iter_mut() {
                aux_bus.set_sample_rate(self.sample_rate as f32);
            }
            
            // Update master bus (EQ + FX chain)
            router.master.parametric_eq.set_sample_rate(self.sample_rate as f32);
            router.master.set_sample_rate(self.sample_rate as f32);
        }

        let input_channels = input_config.channels as usize;
        let output_channels = output_config.channels as usize;

        let err_fn = |err| eprintln!("[Engine] Stream error: {}", err);

        // Clone router for callbacks
        let router_output = Arc::clone(&self.router);

        // Meter update counter and interval (send levels every ~50ms at 48kHz = 2400 frames)
        let meter_update_frames = Arc::new(Mutex::new(0_usize));
        let meter_interval = 2400_usize;

        // Performance tracking
        let perf_stats = Arc::new(Mutex::new(PerformanceStats::new()));
        let perf_stats_clone = Arc::clone(&perf_stats);
        let sample_rate_for_perf = self.sample_rate; // Save sample rate for performance calculation

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
                // Start performance measurement
                let start_time = Instant::now();
                
                let input_buf = input_buffer.lock().unwrap();
                let frames = data.len() / output_channels;

                // Acquire lock, process audio, release lock quickly
                let (levels_to_send, fft_data) = {
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

                        // Push MASTER BUS samples to FFT analyzer (parallel tap, doesn't affect audio)
                        let (master_l, master_r) = router.last_master_output;
                        router.fft_analyzer.push_samples(master_l, master_r);

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
                    
                    let levels_to_send = if *counter >= meter_interval {
                        *counter = 0;
                        
                        // Copy levels data while we have the lock
                        let track_levels: Vec<TrackLevels> = router.tracks.iter()
                            .map(|t| TrackLevels {
                                track: t.id,
                                level_l: t.level_l,
                                level_r: t.level_r,
                                waveform: t.get_waveform_buffer(128), // 128 samples for efficient streaming
                                compressor_input_db: t.compressor.input_level_db,
                                compressor_reduction_db: t.compressor.gain_reduction_db,
                                gate_input_db: t.gate.input_level_db,
                                gate_attenuation_db: t.gate.attenuation_db,
                            })
                            .collect();
                        
                        let master_l = router.master.level_l;
                        let master_r = router.master.level_r;
                        
                        // Collect subgroup levels
                        let subgroup_levels: Vec<SubgroupLevels> = router
                            .subgroups
                            .iter()
                            .map(|sg| SubgroupLevels {
                                subgroup: sg.id,
                                level_l: sg.level_l,
                                level_r: sg.level_r,
                            })
                            .collect();
                        
                        // Reset peak levels after reading
                        router.master.reset_levels();
                        for track in router.tracks.iter_mut() {
                            track.reset_levels();
                        }
                        for subgroup in router.subgroups.iter_mut() {
                            subgroup.reset_levels();
                        }
                        
                        // Return data to serialize outside the lock
                        Some((track_levels, subgroup_levels, master_l, master_r))
                    } else {
                        None
                    };

                    // Check for FFT data
                    let fft_data = router.fft_analyzer.analyze();
                    
                    (levels_to_send, fft_data)
                }; // Lock is released here
                
                drop(input_buf); // Release input buffer lock
                
                // Send meter updates outside the lock
                if let Some((track_levels, subgroup_levels, master_l, master_r)) = levels_to_send {
                    let response = Response::Levels {
                        tracks: track_levels,
                        subgroups: subgroup_levels,
                        master_l,
                        master_r,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&response) {
                        println!("{}", json);
                    }
                }

                // Send FFT data if available
                if let Some((bins_left, bins_right)) = fft_data {
                    let response = Response::FFTData {
                        bins_left,
                        bins_right,
                        sample_rate: sample_rate_for_perf,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&response) {
                        println!("{}", json);
                    }
                }

                // End performance measurement and log statistics
                let process_time = start_time.elapsed();
                let process_time_us = process_time.as_micros();
                
                // Calculate buffer duration (theoretical time available for processing)
                let buffer_duration_us = (frames as u128 * 1_000_000) / (sample_rate_for_perf as u128);
                let buffer_latency_ms = (frames as f32 / sample_rate_for_perf as f32) * 1000.0;
                let cpu_usage = (process_time_us as f32 / buffer_duration_us as f32) * 100.0;
                
                // Record performance stats
                let mut stats = perf_stats_clone.lock().unwrap();
                stats.record(process_time_us);
                
                // Send performance stats every 2-3 seconds
                if stats.should_log() && stats.buffer_count > 0 {
                    let avg_us = stats.total_process_time_us / stats.buffer_count as u128;
                    let avg_ms = avg_us as f32 / 1000.0;
                    let avg_cpu = (avg_us as f32 / buffer_duration_us as f32) * 100.0;
                    let min_ms = stats.min_process_time_us as f32 / 1000.0;
                    let max_ms = stats.max_process_time_us as f32 / 1000.0;
                    
                    let response = Response::PerformanceStats {
                        buffer_size: frames,
                        latency_ms: buffer_latency_ms,
                        avg_process_ms: avg_ms,
                        cpu_percent: avg_cpu,
                        min_process_ms: min_ms,
                        max_process_ms: max_ms,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&response) {
                        println!("{}", json);
                    }
                    
                    stats.reset();
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

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.input_stream.take() {
            drop(stream);
        }
        if let Some(stream) = self.output_stream.take() {
            drop(stream);
        }
        
        Ok(())
    }

    // Track source commands
    fn set_track_source_input(&self, track: usize, left_ch: u16, right_ch: u16) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::set_source_input(&mut router, track, left_ch, right_ch)
    }

    fn set_track_source_signal(&self, track: usize, waveform: &str, frequency: f32) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::set_source_signal(&mut router, track, waveform, frequency, self.sample_rate)
    }

    fn set_track_source_file(&self, track: usize, file_path: &str) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::set_source_file(&mut router, track, file_path, self.sample_rate)
    }

    fn play_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::play_file(&mut router, track, self.sample_rate)
    }

    fn pause_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::pause_file(&mut router, track)
    }

    fn stop_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::stop_file(&mut router, track)
    }

    fn stop_all_files(&self) {
        let mut router = self.router.lock().unwrap();
        router.stop_all_files();
    }

    // Track controls
    fn set_gain(&self, track: usize, gain: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_gain(&mut router, track, gain);
    }

    fn set_volume(&self, track: usize, volume: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_volume(&mut router, track, volume);
    }

    fn set_mute(&self, track: usize, mute: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_mute(&mut router, track, mute);
    }

    fn set_route_to_master(&self, track: usize, route: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_route_to_master(&mut router, track, route);
    }

    fn set_pan(&self, track: usize, pan: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_pan(&mut router, track, pan);
    }

    fn set_pad(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_pad(&mut router, track, enabled);
    }

    fn set_hpf(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_hpf(&mut router, track, enabled);
    }

    // Track dynamics controls
    fn set_compressor(&self, track: usize, enabled: bool, threshold: f32, ratio: f32, attack: f32, release: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.tracks.get_mut(track) {
            t.compressor.set_enabled(enabled);
            t.compressor.set_threshold(threshold);
            t.compressor.set_ratio(ratio);
            t.compressor.set_attack(attack);
            t.compressor.set_release(release);
        }
    }

    fn set_gate(&self, track: usize, enabled: bool, threshold: f32, range: f32, attack: f32, release: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.tracks.get_mut(track) {
            t.gate.set_enabled(enabled);
            t.gate.set_threshold(threshold);
            t.gate.set_range(range);
            t.gate.set_attack(attack);
            t.gate.set_release(release);
        }
    }

    // Track EQ controls
    fn set_eq(&self, track: usize, low: f32, low_mid: f32, high_mid: f32, high: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_eq(&mut router, track, low, low_mid, high_mid, high);
    }

    fn set_eq_enabled(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_eq_enabled(&mut router, track, enabled);
    }

    // Parametric EQ controls
    fn set_parametric_eq_filters(&self, track: usize, filters: &[ParametricFilter]) {
        use equalizer::FilterType;
        
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.tracks.get_mut(track) {
            // Clear existing filters
            t.parametric_eq.clear();
            
            // Add new filters
            for filter in filters {
                let filter_type = match filter.filter_type.as_str() {
                    "lowshelf" => FilterType::LowShelf,
                    "highshelf" => FilterType::HighShelf,
                    "peaking" => FilterType::Peaking,
                    "lowpass" => FilterType::LowPass,
                    "highpass" => FilterType::HighPass,
                    _ => {
                        eprintln!("[Track {}] Unknown filter type: {}", track, filter.filter_type);
                        continue;
                    }
                };
                
                t.parametric_eq.add_band(filter_type, filter.frequency, filter.gain, filter.q);
            }
        } else {
            eprintln!("[Engine] Invalid track number: {}", track);
        }
    }

    fn set_parametric_eq_enabled(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.tracks.get_mut(track) {
            t.parametric_eq.set_enabled(enabled);
        } else {
            eprintln!("[Engine] Invalid track number: {}", track);
        }
    }

    fn clear_parametric_eq(&self, track: usize) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.tracks.get_mut(track) {
            t.parametric_eq.clear();
        } else {
            eprintln!("[Engine] Invalid track number: {}", track);
        }
    }

    // Master controls
    fn set_master_gain(&self, gain: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.gain = gain.max(0.0); // No upper limit
        let gain_db = if gain > 0.0 { 20.0 * gain.log10() } else { -90.0 };
    }

    fn set_master_mute(&self, mute: bool) {
        let mut router = self.router.lock().unwrap();
        router.master.mute = mute;
    }

    fn set_master_parametric_eq_filters(&self, filters: &[ParametricFilter]) {
        use equalizer::FilterType;
        
        let mut router = self.router.lock().unwrap();
        // Clear existing filters
        router.master.parametric_eq.clear();
        
        // Add new filters
        for filter in filters {
            let filter_type = match filter.filter_type.as_str() {
                "lowshelf" => FilterType::LowShelf,
                "highshelf" => FilterType::HighShelf,
                "peaking" => FilterType::Peaking,
                "lowpass" => FilterType::LowPass,
                "highpass" => FilterType::HighPass,
                _ => {
                    eprintln!("[Master] Unknown filter type: {}", filter.filter_type);
                    continue;
                }
            };
            
            router.master.parametric_eq.add_band(filter_type, filter.frequency, filter.gain, filter.q);
        }
    }

    fn set_master_parametric_eq_enabled(&self, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        router.master.parametric_eq.set_enabled(enabled);
    }

    fn clear_master_parametric_eq(&self) {
        let mut router = self.router.lock().unwrap();
        router.master.parametric_eq.clear();
    }

    fn set_master_output_channels(&self, left_ch: u16, right_ch: u16) {
        let mut router = self.router.lock().unwrap();
        router.master.output_channel_selection = ChannelSelection::new(left_ch, right_ch);
    }

    // Master FX methods
    fn set_master_compressor(&self, enabled: bool, threshold: f32, ratio: f32, attack: f32, release: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.compressor.set_enabled(enabled);
        router.master.compressor.set_threshold(threshold);
        router.master.compressor.set_ratio(ratio);
        router.master.compressor.set_attack(attack);
        router.master.compressor.set_release(release);
    }

    fn set_master_limiter(&self, enabled: bool, ceiling: f32, release: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.limiter.set_enabled(enabled);
        router.master.limiter.set_ceiling(ceiling);
        router.master.limiter.set_release(release);
    }

    fn set_master_delay(&self, enabled: bool, time_l: f32, time_r: f32, feedback: f32, mix: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.delay.set_enabled(enabled);
        router.master.delay.set_delay_time_left(time_l);
        router.master.delay.set_delay_time_right(time_r);
        router.master.delay.set_feedback(feedback);
        router.master.delay.set_mix(mix);
    }

    fn set_master_reverb(&self, enabled: bool, room_size: f32, damping: f32, wet: f32, width: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.reverb.set_enabled(enabled);
        router.master.reverb.set_room_size(room_size);
        router.master.reverb.set_damping(damping);
        router.master.reverb.set_wet(wet);
        router.master.reverb.set_width(width);
    }

    // Subgroup methods
    fn add_subgroup(&self) -> usize {
        let mut router = self.router.lock().unwrap();
        router.add_subgroup()
    }

    fn remove_subgroup(&self, subgroup: usize) {
        let mut router = self.router.lock().unwrap();
        router.remove_subgroup(subgroup);
    }

    fn set_subgroup_gain(&self, subgroup: usize, gain: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.gain = gain.max(0.0);
        }
    }

    fn set_subgroup_mute(&self, subgroup: usize, mute: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.mute = mute;
        }
    }

    fn set_subgroup_output_enabled(&self, subgroup: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.output_enabled = enabled;
        }
    }

    fn set_subgroup_route_to_master(&self, subgroup: usize, route: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.route_to_master = route;
        }
    }

    fn set_subgroup_output_channels(&self, subgroup: usize, left_ch: u16, right_ch: u16) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.output_channel_selection = ChannelSelection::new(left_ch, right_ch);
        }
    }

    fn set_track_route_to_subgroup(&self, track: usize, subgroup: usize, route: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            if route {
                // Add subgroup to routing if not already present
                if !t.route_to_subgroups.contains(&subgroup) {
                    t.route_to_subgroups.push(subgroup);
                }
            } else {
                // Remove subgroup from routing
                t.route_to_subgroups.retain(|&sg| sg != subgroup);
            }
        }
    }

    // Aux bus methods
    fn set_track_aux_send(&self, track: usize, aux: usize, level: f32, pre_fader: bool, muted: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            if aux < t.aux_sends.len() {
                t.aux_sends[aux].level = level.max(0.0);
                t.aux_sends[aux].pre_fader = pre_fader;
                t.aux_sends[aux].muted = muted;
            }
        }
    }

    fn set_aux_bus_gain(&self, aux: usize, gain: f32) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            router.aux_buses[aux].gain = gain.max(0.0);
        }
    }

    fn set_aux_bus_mute(&self, aux: usize, mute: bool) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            router.aux_buses[aux].mute = mute;
        }
    }

    fn set_aux_bus_reverb(&self, aux: usize, enabled: bool, room_size: f32, damping: f32, wet: f32, width: f32) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            router.aux_buses[aux].reverb.set_enabled(enabled);
            router.aux_buses[aux].reverb.set_room_size(room_size);
            router.aux_buses[aux].reverb.set_damping(damping);
            router.aux_buses[aux].reverb.set_wet(wet);
            router.aux_buses[aux].reverb.set_width(width);
        }
    }

    fn set_aux_bus_delay(&self, aux: usize, enabled: bool, time: f32, feedback: f32, mix: f32) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            router.aux_buses[aux].delay.set_enabled(enabled);
            router.aux_buses[aux].delay.set_delay_time_left(time);
            router.aux_buses[aux].delay.set_delay_time_right(time);
            router.aux_buses[aux].delay.set_feedback(feedback);
            router.aux_buses[aux].delay.set_mix(mix);
        }
    }

    fn set_aux_bus_route_to_master(&self, aux: usize, route: bool) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            router.aux_buses[aux].route_to_master = route;
        }
    }

    fn set_aux_bus_output_enabled(&self, aux: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            router.aux_buses[aux].output_enabled = enabled;
        }
    }

    fn set_aux_bus_output_channels(&self, aux: usize, left_ch: u16, right_ch: u16) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            router.aux_buses[aux].output_channel_selection = ChannelSelection::new(left_ch, right_ch);
        }
    }

    /// Handle a command and return an optional response (only for critical operations)
    fn handle_command(&mut self, command: Command) -> Option<Response> {
        match command {
            Command::Start {
                input_device,
                output_device,
                sample_rate,
            } => match self.start(input_device, output_device, sample_rate) {
                Ok(_) => Some(Response::Started),
                Err(e) => Some(Response::Error {
                    message: format!("Start failed: {}", e),
                }),
            },
            Command::Stop => match self.stop() {
                Ok(_) => Some(Response::Stopped),
                Err(e) => Some(Response::Error {
                    message: format!("Stop failed: {}", e),
                }),
            },
            Command::SetTrackSourceInput {
                track,
                left_channel,
                right_channel,
            } => {
                // Fire-and-forget command, no response needed
                let _ = self.set_track_source_input(track, left_channel, right_channel);
                None
            }
            Command::SetTrackSourceSignal {
                track,
                waveform,
                frequency,
            } => {
                // Fire-and-forget command, no response needed
                let _ = self.set_track_source_signal(track, &waveform, frequency);
                None
            }
            Command::SetTrackSourceFile { track, file_path } => {
                match self.set_track_source_file(track, &file_path) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("[Engine] SetTrackSourceFile FAILED for track {}: {}", track, e);
                    }
                }
                None
            }
            Command::PlayFile { track } => {
                match self.play_file(track) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("[Engine] PlayFile FAILED for track {}: {}", track, e);
                    }
                }
                None
            }
            Command::PauseFile { track } => {
                let _ = self.pause_file(track);
                None
            }
            Command::StopFile { track } => {
                let _ = self.stop_file(track);
                None
            }
            Command::StopAllFiles => {
                self.stop_all_files();
                None
            }
            Command::SetGain { track, gain } => {
                self.set_gain(track, gain);
                None
            }
            Command::SetVolume { track, volume } => {
                self.set_volume(track, volume);
                None
            }
            Command::SetMute { track, mute } => {
                self.set_mute(track, mute);
                None
            }
            Command::SetRouteToMaster { track, route } => {
                self.set_route_to_master(track, route);
                None
            }
            Command::SetPan { track, pan } => {
                self.set_pan(track, pan);
                None
            }
            Command::SetTrackPad { track, enabled } => {
                self.set_pad(track, enabled);
                None
            }
            Command::SetTrackHPF { track, enabled } => {
                self.set_hpf(track, enabled);
                None
            }
            Command::SetCompressor {
                track,
                enabled,
                threshold,
                ratio,
                attack,
                release,
            } => {
                self.set_compressor(track, enabled, threshold, ratio, attack, release);
                None
            }
            Command::SetGate {
                track,
                enabled,
                threshold,
                range,
                attack,
                release,
            } => {
                self.set_gate(track, enabled, threshold, range, attack, release);
                None
            }
            Command::SetEQ {
                track,
                low,
                low_mid,
                high_mid,
                high,
            } => {
                self.set_eq(track, low, low_mid, high_mid, high);
                None
            }
            Command::SetEQEnabled { track, enabled } => {
                self.set_eq_enabled(track, enabled);
                None
            }
            Command::SetParametricEQFilters { track, filters } => {
                self.set_parametric_eq_filters(track, &filters);
                None
            }
            Command::SetParametricEQEnabled { track, enabled } => {
                self.set_parametric_eq_enabled(track, enabled);
                None
            }
            Command::ClearParametricEQ { track } => {
                self.clear_parametric_eq(track);
                None
            }
            Command::SetMasterGain { gain } => {
                self.set_master_gain(gain);
                None
            }
            Command::SetMasterMute { mute } => {
                self.set_master_mute(mute);
                None
            }
            Command::SetMasterParametricEQFilters { filters } => {
                self.set_master_parametric_eq_filters(&filters);
                None
            }
            Command::SetMasterParametricEQEnabled { enabled } => {
                self.set_master_parametric_eq_enabled(enabled);
                None
            }
            Command::ClearMasterParametricEQ => {
                self.clear_master_parametric_eq();
                None
            }
            Command::SetMasterOutputChannels {
                left_channel,
                right_channel,
            } => {
                self.set_master_output_channels(left_channel, right_channel);
                None
            }
            Command::SetMasterCompressor {
                enabled,
                threshold,
                ratio,
                attack,
                release,
            } => {
                self.set_master_compressor(enabled, threshold, ratio, attack, release);
                None
            }
            Command::SetMasterLimiter {
                enabled,
                ceiling,
                release,
            } => {
                self.set_master_limiter(enabled, ceiling, release);
                None
            }
            Command::SetMasterDelay {
                enabled,
                time_l,
                time_r,
                feedback,
                mix,
            } => {
                self.set_master_delay(enabled, time_l, time_r, feedback, mix);
                None
            }
            Command::SetMasterReverb {
                enabled,
                room_size,
                damping,
                wet,
                width,
            } => {
                self.set_master_reverb(enabled, room_size, damping, wet, width);
                None
            }
            Command::AddSubgroup => {
                let id = self.add_subgroup();
                Some(Response::SubgroupCreated { id })
            }
            Command::RemoveSubgroup { subgroup } => {
                self.remove_subgroup(subgroup);
                None
            }
            Command::SetSubgroupGain { subgroup, gain } => {
                self.set_subgroup_gain(subgroup, gain);
                None
            }
            Command::SetSubgroupMute { subgroup, mute } => {
                self.set_subgroup_mute(subgroup, mute);
                None
            }
            Command::SetSubgroupOutputEnabled { subgroup, enabled } => {
                self.set_subgroup_output_enabled(subgroup, enabled);
                None
            }
            Command::SetSubgroupRouteToMaster { subgroup, route } => {
                self.set_subgroup_route_to_master(subgroup, route);
                None
            }
            Command::SetSubgroupOutputChannels {
                subgroup,
                left_channel,
                right_channel,
            } => {
                self.set_subgroup_output_channels(subgroup, left_channel, right_channel);
                None
            }
            Command::SetTrackRouteToSubgroup {
                track,
                subgroup,
                route,
            } => {
                self.set_track_route_to_subgroup(track, subgroup, route);
                None
            }
            Command::SetTrackAuxSend {
                track,
                aux,
                level,
                pre_fader,
                muted,
            } => {
                self.set_track_aux_send(track, aux, level, pre_fader, muted);
                None
            }
            Command::SetAuxBusGain { aux, gain } => {
                self.set_aux_bus_gain(aux, gain);
                None
            }
            Command::SetAuxBusMute { aux, mute } => {
                self.set_aux_bus_mute(aux, mute);
                None
            }
            Command::SetAuxBusReverb {
                aux,
                enabled,
                room_size,
                damping,
                wet,
                width,
            } => {
                self.set_aux_bus_reverb(aux, enabled, room_size, damping, wet, width);
                None
            }
            Command::SetAuxBusDelay {
                aux,
                enabled,
                time,
                feedback,
                mix,
            } => {
                self.set_aux_bus_delay(aux, enabled, time, feedback, mix);
                None
            }
            Command::SetAuxBusRouteToMaster { aux, route } => {
                self.set_aux_bus_route_to_master(aux, route);
                None
            }
            Command::SetAuxBusOutputEnabled { aux, enabled } => {
                self.set_aux_bus_output_enabled(aux, enabled);
                None
            }
            Command::SetAuxBusOutputChannels {
                aux,
                left_channel,
                right_channel,
            } => {
                self.set_aux_bus_output_channels(aux, left_channel, right_channel);
                None
            }
            Command::ListDevices => match self.list_devices() {
                Ok(devices) => Some(Response::Devices { devices }),
                Err(e) => Some(Response::Error {
                    message: format!("List devices failed: {}", e),
                }),
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

    // Loop: read commands from stdin
    while let Some(Ok(line)) = lines.next() {
        match serde_json::from_str::<Command>(&line) {
            Ok(command) => {
                // Only send response if there is one (critical operations only)
                if let Some(response) = engine.handle_command(command) {
                    send_response(&response);
                }
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

