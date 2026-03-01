use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
        buffer_size: Option<u32>,
    },
    #[serde(rename = "stop")]
    Stop,

    // Track source selection
    #[serde(rename = "set_track_source_input")]
    SetTrackSourceInput {
        track: usize,
        left_channel: u16,
        right_channel: u16,
        device_name: Option<String>,
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

    // Master tap (for recording) - Rust saves the file directly
    #[serde(rename = "enable_master_tap")]
    EnableMasterTap {
        file_path: String, // Where to save the WAV file
    },
    #[serde(rename = "disable_master_tap")]
    DisableMasterTap,

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

    // Performance management
    #[serde(rename = "set_updates_suspended")]
    SetUpdatesSuspended { suspended: bool },

    // Input stream management (on-demand for privacy)
    #[serde(rename = "open_audio_input")]
    OpenAudioInput { device_name: Option<String> },
    #[serde(rename = "close_audio_input")]
    CloseAudioInput,

    // Device management
    #[serde(rename = "list_devices")]
    ListDevices,
    #[serde(rename = "list_audio_inputs")]
    ListAudioInputs,
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
    #[serde(rename = "audio_inputs")]
    AudioInputs { inputs: Vec<DeviceInfo> },
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
    input_sample_rate: u32, // Track input sample rate (may differ from output)
    output_buffer_size: Option<u32>, // Track output buffer size for input matching
    updates_suspended: Arc<AtomicBool>,
    input_buffer: Arc<Mutex<Vec<f32>>>, // Shared buffer - always contains latest input frame
    input_channels: Arc<AtomicUsize>,
    input_users: HashSet<usize>, // Track IDs that are using audio input
    current_input_device: Option<String>, // Currently open input device name
    master_tap_buffer: Arc<Mutex<Vec<f32>>>, // Master output tap for recording (stereo interleaved)
    master_tap_enabled: Arc<AtomicBool>, // Enable/disable master tap
    recording_path: Arc<Mutex<Option<PathBuf>>>, // Path where to save the recording
}

impl AudioEngine {
    fn new() -> Self {
        let audio_io = AudioIO::new();
        let router = Arc::new(Mutex::new(Router::new(24))); // Support up to 24 tracks
        let updates_suspended = Arc::new(AtomicBool::new(false));
        let input_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let input_channels = Arc::new(AtomicUsize::new(2)); // Default stereo
        let audio_io = AudioIO::new();
        let router = Arc::new(Mutex::new(Router::new(24))); // Support up to 24 tracks
        let updates_suspended = Arc::new(AtomicBool::new(false));
        let input_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let input_channels = Arc::new(AtomicUsize::new(2)); // Default stereo
        let master_tap_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(4800000))); // ~100 sec @ 48kHz stereo
        let master_tap_enabled = Arc::new(AtomicBool::new(false));
        let recording_path = Arc::new(Mutex::new(None));

        Self {
            audio_io,
            router,
            input_stream: None,
            output_stream: None,
            sample_rate: 48000, // Default, will be overwritten by device native rate
            input_sample_rate: 48000, // Default, will be overwritten when input opens
            output_buffer_size: None,
            updates_suspended,
            input_buffer,
            input_channels,
            input_users: HashSet::new(),
            current_input_device: None,
            master_tap_buffer,
            master_tap_enabled,
            recording_path,
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
        buffer_size: Option<u32>,
    ) -> Result<()> {
        eprintln!("[Engine] start() called with: sample_rate={:?}, buffer_size={:?}", sample_rate, buffer_size);
        if self.input_stream.is_some() || self.output_stream.is_some() {
            return Ok(()); // Already running
        }

        // Get output device (REQUIRED)
        let output_device = if let Some(name) = output_device_name {
            self.audio_io.find_device_by_name(&name, false)?
        } else {
            self.audio_io.default_output_device()?
        };

        // Input device is OPTIONAL - only get it if explicitly requested
        // This prevents macOS from showing "microphone in use" indicator
        // TODO: Implement on-demand input opening when track selects audio input
        let _input_device_available = input_device_name.is_some();
        
        // Skip input device for now (privacy)
        /*
        let input_device = if let Some(name) = input_device_name {
            Some(self.audio_io.find_device_by_name(&name, true)?)
        } else {
            None // Don't use default input unless explicitly requested
        };
        */

        // Get device name for logging
        let output_device_name = output_device.name().unwrap_or_else(|_| String::from("Unknown"));
        let output_device_name_lower = output_device_name.to_lowercase();
        
        let is_bluetooth = output_device_name_lower.contains("airpods") 
            || output_device_name_lower.contains("bluetooth")
            || output_device_name_lower.contains("wireless")
            || output_device_name_lower.contains("bt");
        
        // Skip input config for now (see TODO above about privacy)
        // let input_config = self.audio_io.get_supported_config(&input_device, true, sample_rate, buffer_size)?;
        let output_config = self.audio_io.get_supported_config(&output_device, false, sample_rate, buffer_size)?;

        self.sample_rate = output_config.sample_rate.0;
        
        // Save buffer size for input stream matching
        self.output_buffer_size = match output_config.buffer_size {
            cpal::BufferSize::Fixed(size) => Some(size),
            cpal::BufferSize::Default => None,
        };
        
        // Get channel count for logging
        let output_channels = output_config.channels as usize;
        
        // Log configuration
        let actual_buffer_size = output_config.buffer_size;
        let device_type = if is_bluetooth { "🎧 Bluetooth" } else { "🔌 Wired" };
        
        match actual_buffer_size {
            cpal::BufferSize::Fixed(size) => {
                let latency_ms = (size as f32 / self.sample_rate as f32) * 1000.0;
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
                eprintln!("[Engine] Audio Configuration");
                eprintln!("[Engine] ───────────────────────────────────────────────────");
                eprintln!("[Engine] Device: {}", output_device_name);
                eprintln!("[Engine] Sample Rate: {} Hz", self.sample_rate);
                eprintln!("[Engine] Output Channels: {}", output_channels);
                eprintln!("[Engine] Buffer Size: {} frames ({:.2}ms latency)", size, latency_ms);
                eprintln!("[Engine] Type: {}", if is_bluetooth { "Bluetooth" } else { "Wired" });
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
            },
            cpal::BufferSize::Default => {
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
                eprintln!("[Engine] Audio Configuration");
                eprintln!("[Engine] ───────────────────────────────────────────────────");
                eprintln!("[Engine] Device: {}", output_device_name);
                eprintln!("[Engine] Sample Rate: {} Hz", self.sample_rate);
                eprintln!("[Engine] Output Channels: {}", output_channels);
                eprintln!("[Engine] Buffer Size: DEFAULT (system auto)");
                eprintln!("[Engine] Type: {}", if is_bluetooth { "Bluetooth" } else { "Wired" });
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

        let input_channels = 2; // Default stereo (not used since input is disabled)

        let err_fn = |err| eprintln!("[Engine] Stream error: {}", err);

        // Clone router for callbacks
        let router_output = Arc::clone(&self.router);
        let updates_suspended_flag = Arc::clone(&self.updates_suspended);

        // Meter update counter and interval (send levels every ~50ms at 48kHz = 2400 frames)
        let meter_update_frames = Arc::new(Mutex::new(0_usize));
        let meter_interval = 2400_usize;

        // Performance tracking
        let perf_stats = Arc::new(Mutex::new(PerformanceStats::new()));
        let perf_stats_clone = Arc::clone(&perf_stats);
        let sample_rate_for_perf = self.sample_rate; // Save sample rate for performance calculation

        // === INPUT STREAM: capture audio ===
        // Input stream is now managed on-demand via open_audio_input()/close_audio_input()
        // when tracks select audio input sources. This preserves privacy by not opening
        // the microphone unless actually needed.
        
        // Share input buffer with output callback
        let input_buffer = Arc::clone(&self.input_buffer);
        let input_channels = Arc::clone(&self.input_channels);
        let master_tap_buffer = Arc::clone(&self.master_tap_buffer);
        let master_tap_enabled = Arc::clone(&self.master_tap_enabled);

        // === OUTPUT STREAM: process and output audio ===
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Start performance measurement
                let start_time = Instant::now();
                
                let frames = data.len() / output_channels;
                let input_ch_count = input_channels.load(Ordering::Relaxed);
                
                // Lock input buffer and copy the data (keep lock time minimal)
                let input_buf = input_buffer.lock().unwrap();

                // Acquire lock, process audio, release lock quickly
                let (levels_to_send, fft_data) = {
                    let mut router = router_output.lock().unwrap();
                    
                    // Process all frames
                    for frame_idx in 0..frames {
                        // Extract input frame from buffer (take latest available data)
                        let input_frame: Option<Vec<f32>> = if input_ch_count > 0 && !input_buf.is_empty() {
                            let start = frame_idx * input_ch_count;
                            let end = start + input_ch_count;
                            if end <= input_buf.len() {
                                Some(input_buf[start..end].to_vec())
                            } else {
                                None // Not enough data for this frame
                            }
                        } else {
                            None
                        };

                        // Process one frame through router
                        let (master_left, master_right) = router.process_frame(input_frame.as_deref());

                        // Push MASTER BUS samples to FFT analyzer (parallel tap, doesn't affect audio)
                        let (master_l, master_r) = router.last_master_output;
                        router.fft_analyzer.push_samples(master_l, master_r);

                        // Initialize output frame to silence
                        let out_frame_start = frame_idx * output_channels;
                        for ch in 0..output_channels {
                            data[out_frame_start + ch] = 0.0;
                        }

                        // Write master output to its channels
                        let master_left_ch = router.master.output_channel_selection.left as usize;
                        let master_right_ch = router.master.output_channel_selection.right as usize;
                        if master_left_ch < output_channels {
                            data[out_frame_start + master_left_ch] += master_left.clamp(-1.0, 1.0);
                        }
                        if master_right_ch < output_channels {
                            data[out_frame_start + master_right_ch] += master_right.clamp(-1.0, 1.0);
                        }

                        // Write subgroup outputs to their channels (if enabled)
                        for (i, subgroup) in router.subgroups.iter().enumerate() {
                            if subgroup.output_enabled && i < router.last_subgroup_outputs.len() {
                                let (sg_l, sg_r) = router.last_subgroup_outputs[i];
                                let sg_left_ch = subgroup.output_channel_selection.left as usize;
                                let sg_right_ch = subgroup.output_channel_selection.right as usize;
                                
                                if sg_left_ch < output_channels {
                                    data[out_frame_start + sg_left_ch] += sg_l.clamp(-1.0, 1.0);
                                }
                                if sg_right_ch < output_channels {
                                    data[out_frame_start + sg_right_ch] += sg_r.clamp(-1.0, 1.0);
                                }
                            }
                        }

                        // Write aux outputs to their channels (if enabled)
                        for (i, aux_bus) in router.aux_buses.iter().enumerate() {
                            if aux_bus.output_enabled && i < router.last_aux_outputs.len() {
                                let (aux_l, aux_r) = router.last_aux_outputs[i];
                                let aux_left_ch = aux_bus.output_channel_selection.left as usize;
                                let aux_right_ch = aux_bus.output_channel_selection.right as usize;
                                if aux_left_ch < output_channels {
                                    data[out_frame_start + aux_left_ch] += aux_l.clamp(-1.0, 1.0);
                                }
                                if aux_right_ch < output_channels {
                                    data[out_frame_start + aux_right_ch] += aux_r.clamp(-1.0, 1.0);
                                }
                            }
                        }
                    }

                    // Copy master output to tap buffer if enabled (for recording)
                    // Record ONLY master bus (not direct subgroups/aux)
                    // Do this AFTER writing to output to minimize audio callback latency
                    if master_tap_enabled.load(Ordering::Relaxed) {
                        if let Ok(mut tap_buffer) = master_tap_buffer.try_lock() {
                            // Copy stereo interleaved samples (L, R, L, R, ...)
                            // Limit to 10 minutes max (prevents memory issues for very long recordings)
                            let max_samples = sample_rate_for_perf as usize * 2 * 600; // 10 min stereo
                            if tap_buffer.len() < max_samples {
                                for frame_idx in 0..frames {
                                    // Record master output from router (before adding direct outputs)
                                    let (master_l, master_r) = router.last_master_output;
                                    tap_buffer.push(master_l.clamp(-1.0, 1.0));
                                    tap_buffer.push(master_r.clamp(-1.0, 1.0));
                                }
                            }
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
                
                // CRITICAL: Check if updates are suspended (during window resize)
                // This prevents blocking I/O on stdout which would freeze audio
                let suspended = updates_suspended_flag.load(Ordering::Relaxed);
                
                if !suspended {
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
                
                // Send performance stats every 2-3 seconds (only if not suspended)
                if !suspended && stats.should_log() && stats.buffer_count > 0 {
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

        // Start streams (input stream is disabled for privacy - see TODO above)
        // input_stream.play()?;
        output_stream.play()?;

        self.input_stream = None; // Keep None until we implement on-demand opening
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

    fn open_audio_input(&mut self, track_id: usize, device_name: Option<String>) -> Result<()> {
        // Add track to users set
        let was_empty = self.input_users.is_empty();
        self.input_users.insert(track_id);
        
        // Determine the device name to use (None means default device)
        let requested_device = device_name.clone();
        
        // Check if we need to change device
        let device_changed = self.current_input_device != requested_device;
        
        // If device changed, close existing stream
        if device_changed && self.input_stream.is_some() {
            eprintln!("[Engine] Input device changed from {:?} to {:?}, reopening stream", 
                self.current_input_device, requested_device);
            if let Some(stream) = self.input_stream.take() {
                drop(stream);
            }
            // Clear buffer when changing device
            if let Ok(mut buffer) = self.input_buffer.lock() {
                buffer.clear();
            }
        }
        
        // If already open with the same device and had other users, just return
        if !was_empty && !device_changed {
            eprintln!("[Engine] Track {} using audio input (total users: {})", track_id, self.input_users.len());
            return Ok(());
        }

        // Get input device
        let input_device = if let Some(name) = &requested_device {
            eprintln!("[Engine] Opening input device: {}", name);
            self.audio_io.find_device_by_name(name, true)?
        } else {
            eprintln!("[Engine] Opening default input device");
            self.audio_io.default_input_device()?
        };

        // Use device native sample rate and buffer size for best compatibility
        // BUT: Force input to match output sample rate to avoid resampling issues
        // AND: Use same buffer size as output for synchronized callbacks
        let buffer_size = self.output_buffer_size;
        
        let input_config = self.audio_io.get_supported_config(&input_device, true, Some(self.sample_rate), buffer_size)?;
        
        eprintln!("[Engine] Input device sample rate: {} Hz (output: {} Hz)", 
            input_config.sample_rate.0, self.sample_rate);
        
        if let Some(size) = buffer_size {
            eprintln!("[Engine] Input buffer size: {} frames (matched to output)", size);
        } else {
            eprintln!("[Engine] Input buffer size: DEFAULT (matched to output)");
        }
        
        // Store input sample rate
        self.input_sample_rate = input_config.sample_rate.0;
        self.input_channels.store(input_config.channels as usize, Ordering::Relaxed);
        
        let input_buffer_clone = Arc::clone(&self.input_buffer);
        
        eprintln!("[Engine] Using shared buffer for input audio");
        
        let err_fn = |err| eprintln!("[Engine] Input stream error: {}", err);

        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Replace buffer with latest input data (CPAL handles sample rate conversion)
                if let Ok(mut buffer) = input_buffer_clone.lock() {
                    buffer.clear();
                    buffer.extend_from_slice(data);
                }
            },
            err_fn,
            None,
        )?;

        input_stream.play()?;
        
        self.input_stream = Some(input_stream);
        self.current_input_device = requested_device.clone();
        
        eprintln!("[Engine] ✓ Audio input opened for track {} (device: {:?})", track_id, requested_device);
        
        Ok(())
    }

    fn close_audio_input(&mut self, track_id: usize) -> Result<()> {
        // Remove track from users set
        self.input_users.remove(&track_id);
        
        eprintln!("[Engine] Track {} stopped using audio input (remaining users: {})", track_id, self.input_users.len());

        // Only actually close the stream when no users remain
        if self.input_users.is_empty() {
            if let Some(stream) = self.input_stream.take() {
                drop(stream);
                eprintln!("[Engine] ✓ Audio input closed (no more users)");
            }
            
            // Clear input buffer
            if let Ok(mut buffer) = self.input_buffer.lock() {
                buffer.clear();
            }
            
            // Clear current device
            self.current_input_device = None;
        }
        
        Ok(())
    }

    // Master tap controls
    fn enable_master_tap(&self, file_path: String) {
        // Clear previous buffer
        if let Ok(mut buffer) = self.master_tap_buffer.lock() {
            buffer.clear();
        }
        // Set recording path
        if let Ok(mut path) = self.recording_path.lock() {
            *path = Some(PathBuf::from(file_path));
        }
        self.master_tap_enabled.store(true, Ordering::Relaxed);
        eprintln!("[Engine] ✓ Master tap enabled - recording started");
    }

    fn disable_master_tap(&self) {
        self.master_tap_enabled.store(false, Ordering::Relaxed);
        
        // Get samples and path
        let samples = if let Ok(mut buffer) = self.master_tap_buffer.lock() {
            let s = buffer.clone();
            buffer.clear();
            s
        } else {
            Vec::new()
        };

        let path = if let Ok(mut p) = self.recording_path.lock() {
            p.take()
        } else {
            None
        };

        // Save WAV file if we have samples and path
        if !samples.is_empty() && path.is_some() {
            let file_path = path.unwrap();
            match self.write_wav_file(&file_path, &samples) {
                Ok(_) => eprintln!("[Engine] ✓ Recording saved: {:?} ({} samples)", file_path, samples.len()),
                Err(e) => eprintln!("[Engine] ✗ Failed to save recording: {}", e),
            }
        } else {
            eprintln!("[Engine] ✓ Master tap disabled (no recording to save)");
        }
    }

    fn write_wav_file(&self, path: &PathBuf, samples: &[f32]) -> Result<()> {
        let mut file = File::create(path)?;
        
        // Convert float32 to int16
        let num_samples = samples.len();
        let num_channels = 2u16; // Stereo
        let sample_rate = self.sample_rate;
        let bits_per_sample = 16u16;
        let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample / 8) as u32;
        let block_align = num_channels * (bits_per_sample / 8);
        let data_size = num_samples as u32 * (bits_per_sample / 8) as u32;
        
        // Write WAV header
        file.write_all(b"RIFF")?;
        file.write_all(&(36 + data_size).to_le_bytes())?;
        file.write_all(b"WAVE")?;
        
        // fmt chunk
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // chunk size
        file.write_all(&1u16.to_le_bytes())?; // PCM format
        file.write_all(&num_channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&block_align.to_le_bytes())?;
        file.write_all(&bits_per_sample.to_le_bytes())?;
        
        // data chunk
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;
        
        // Write samples (convert f32 to i16)
        for sample in samples {
            let s = sample.max(-1.0).min(1.0);
            let i16_sample = if s < 0.0 {
                (s * 32768.0) as i16
            } else {
                (s * 32767.0) as i16
            };
            file.write_all(&i16_sample.to_le_bytes())?;
        }
        
        Ok(())
    }

    // Track source commands
    fn set_track_source_input(&mut self, track: usize, left_ch: u16, right_ch: u16, device_name: Option<String>) -> Result<()> {
        // If device_name is None, clear the track source and close input
        if device_name.is_none() {
            // Close audio input for this track
            if let Err(e) = self.close_audio_input(track) {
                eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
            }
            
            // Clear track source
            let mut router = self.router.lock().unwrap();
            return track::clear_source(&mut router, track);
        }
        
        // Open input stream when a track selects audio input
        if let Err(e) = self.open_audio_input(track, device_name) {
            eprintln!("[Engine] Failed to open audio input for track {}: {}", track, e);
            return Err(e);
        }
        
        let mut router = self.router.lock().unwrap();
        track::set_source_input(&mut router, track, left_ch, right_ch)
    }

    fn set_track_source_signal(&mut self, track: usize, waveform: &str, frequency: f32) -> Result<()> {
        // Close input stream when track switches away from audio input
        if let Err(e) = self.close_audio_input(track) {
            eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
        }
        
        let mut router = self.router.lock().unwrap();
        track::set_source_signal(&mut router, track, waveform, frequency, self.sample_rate)
    }

    fn set_track_source_file(&mut self, track: usize, file_path: &str) -> Result<()> {
        // Close input stream when track switches away from audio input
        if let Err(e) = self.close_audio_input(track) {
            eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
        }
        
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
                buffer_size,
            } => match self.start(input_device, output_device, sample_rate, buffer_size) {
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
            Command::EnableMasterTap { file_path } => {
                self.enable_master_tap(file_path);
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
            Command::SetTrackSourceInput {
                track,
                left_channel,
                right_channel,
                device_name,
            } => {
                // Fire-and-forget command, no response needed
                let _ = self.set_track_source_input(track, left_channel, right_channel, device_name);
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
            Command::SetUpdatesSuspended { suspended } => {
                self.updates_suspended.store(suspended, Ordering::Relaxed);
                None // No response needed, performance-critical
            }
            Command::OpenAudioInput { device_name } => {
                // Note: This command is deprecated. Input is now managed automatically
                // via SetTrackSourceInput. Kept for backwards compatibility.
                eprintln!("[Engine] Warning: OpenAudioInput command is deprecated, use SetTrackSourceInput instead");
                None
            }
            Command::CloseAudioInput => {
                // Note: This command is deprecated. Input is now managed automatically
                // via track source changes. Kept for backwards compatibility.
                eprintln!("[Engine] Warning: CloseAudioInput command is deprecated, input is managed automatically");
                None
            }
            Command::ListDevices => match self.list_devices() {
                Ok(devices) => Some(Response::Devices { devices }),
                Err(e) => Some(Response::Error {
                    message: format!("List devices failed: {}", e),
                }),
            },
            Command::ListAudioInputs => {
                eprintln!("[Engine] Received ListAudioInputs command");
                match self.audio_io.list_devices() {
                    Ok(devices) => {
                        // Filter only input devices and expand multi-channel devices
                        let input_devices: Vec<DeviceInfo> = devices.into_iter()
                            .filter(|d| d.input_channels > 0)
                            .flat_map(|device| {
                                if device.input_channels > 2 {
                                    // Expand multi-channel device into individual channel entries
                                    let channel_count = device.input_channels as usize;
                                    eprintln!("[Engine] Expanding {} into {} channels", device.name, channel_count);
                                    (0..channel_count).map(move |ch| {
                                        DeviceInfo {
                                            id: format!("{}:{}", device.id, ch),
                                            name: format!("{} - Channel {}", device.name, ch + 1),
                                            input_channels: device.input_channels,
                                            output_channels: device.output_channels,
                                            default_sample_rate: device.default_sample_rate,
                                        }
                                    }).collect::<Vec<_>>()
                                } else {
                                    // Single or stereo channel device
                                    vec![device]
                                }
                            })
                            .collect();
                        
                        eprintln!("[Engine] Found {} input channels", input_devices.len());
                        Some(Response::AudioInputs { inputs: input_devices })
                    },
                    Err(e) => {
                        eprintln!("[Engine] Error listing audio inputs: {}", e);
                        Some(Response::Error {
                            message: format!("Failed to list audio inputs: {}", e),
                        })
                    },
                }
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

