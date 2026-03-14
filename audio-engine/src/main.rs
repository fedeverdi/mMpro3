use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
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
mod ndi_ffi;
mod ndi_stream;
mod loudness;
mod dynamic_range;
mod phase_correlation;
mod stereo_width;
mod headroom;

use audio_io::{AudioIO, ChannelSelection, DeviceInfo};
use routing::Router;
use signal_gen::WaveformType;
use ndi_stream::{NdiStream, NdiSource};
use equalizer::FilterData;

/// Parametric filter specification from frontend
#[derive(Debug, Deserialize, Serialize, Clone)]
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
    #[serde(rename = "set_signal_frequency")]
    SetSignalFrequency {
        track: usize,
        frequency: f32,
    },
    #[serde(rename = "set_signal_waveform")]
    SetSignalWaveform {
        track: usize,
        waveform: String, // "sine", "square", "sawtooth", "triangle", "white", "pink"
    },
    #[serde(rename = "clear_track_source")]
    ClearTrackSource {
        track: usize,
    },
    #[serde(rename = "set_track_source_file")]
    SetTrackSourceFile {
        track: usize,
        file_path: String,
        artist: Option<String>,
        title: Option<String>,
    },
    #[serde(rename = "play_file")]
    PlayFile { 
        track: usize,
        file_path: Option<String>,
        artist: Option<String>,
        title: Option<String>,
    },
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
    #[serde(rename = "set_track_phase_invert")]
    SetTrackPhaseInvert { track: usize, enabled: bool },
    
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
    #[serde(rename = "set_master_gain_left")]
    SetMasterGainLeft { gain: f32 },
    #[serde(rename = "set_master_gain_right")]
    SetMasterGainRight { gain: f32 },
    #[serde(rename = "set_master_mute")]
    SetMasterMute { mute: bool },
    #[serde(rename = "set_master_linked")]
    SetMasterLinked { linked: bool },
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
    #[serde(rename = "set_selected_master_output")]
    SetSelectedMasterOutput {
        device_id: Option<String>,
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
    #[serde(rename = "add_master_fx_effect")]
    AddMasterFxEffect {
        effect_type: String, // "compressor", "limiter", "delay", "reverb"
    },
    #[serde(rename = "remove_master_fx_effect")]
    RemoveMasterFxEffect {
        effect_type: String, // "compressor", "limiter", "delay", "reverb"
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
    #[serde(rename = "set_selected_subgroup_output")]
    SetSelectedSubgroupOutput {
        subgroup: usize,
        device_id: Option<String>,
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
        sample_rate: Option<u32>, // Recording sample rate (44100, 48000, 96000, 192000)
        bit_depth: Option<u32>, // Bit depth (16, 24, 32)
        format: Option<String>, // "wav", "mp3", "opus"
    },
    #[serde(rename = "disable_master_tap")]
    DisableMasterTap,

    // License management
    #[serde(rename = "save_license")]
    SaveLicense {
        key: String,
        license_type: String,
        expires_at: Option<String>,
    },
    #[serde(rename = "get_license")]
    GetLicense,

    // Audio configuration management
    #[serde(rename = "save_audio_config")]
    SaveAudioConfig {
        sample_rate: u32, // 0 = Auto, otherwise specific rate
        buffer_size: u32,
    },
    #[serde(rename = "get_audio_config")]
    GetAudioConfig,

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
    #[serde(rename = "set_aux_bus_route_to_subgroup")]
    SetAuxBusRouteToSubgroup {
        aux: usize,
        subgroup: usize,
        route: bool,
    },
    #[serde(rename = "set_aux_bus_selected_output")]
    SetAuxBusSelectedOutput {
        aux: usize,
        device_id: Option<String>,
    },
    #[serde(rename = "set_track_source_aux_return")]
    SetTrackSourceAuxReturn {
        track: usize,
        aux: usize,
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

    // NDI Streaming
    #[serde(rename = "start_ndi")]
    StartNdi { 
        stream_name: String,
        source: String, // "master", "subgroup1", "subgroup2", etc.
    },
    #[serde(rename = "stop_ndi")]
    StopNdi,
    #[serde(rename = "set_ndi_source")]
    SetNdiSource { source: String },
    #[serde(rename = "set_ndi_name")]
    SetNdiName { name: String },
    #[serde(rename = "set_ndi_video_text")]
    SetNdiVideoText { text: String },
    
    // Loudness Metering (EBU R128)
    #[serde(rename = "get_loudness")]
    GetLoudness,
    #[serde(rename = "reset_loudness")]
    ResetLoudness,
    
    // Dynamic Range Metering
    #[serde(rename = "get_dynamic_range")]
    GetDynamicRange,
    #[serde(rename = "reset_dynamic_range")]
    ResetDynamicRange,
    
    // Phase Correlation Metering (Master)
    #[serde(rename = "get_phase_correlation")]
    GetPhaseCorrelation,
    #[serde(rename = "reset_phase_correlation")]
    ResetPhaseCorrelation,
    
    // Stereo Width Metering (Master)
    #[serde(rename = "get_stereo_width")]
    GetStereoWidth,
    #[serde(rename = "reset_stereo_width")]
    ResetStereoWidth,
    
    // Headroom Metering (Master)
    #[serde(rename = "get_headroom")]
    GetHeadroom,
    #[serde(rename = "reset_headroom")]
    ResetHeadroom,
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
    
    // Optimized meters stream (60fps) - Only real-time data that changes continuously
    #[serde(rename = "meters")]
    Meters {
        tracks: Vec<TrackMeters>,
        subgroups: Vec<SubgroupMeters>,
        auxes: Vec<AuxMeters>,
        master_l: f32,
        master_r: f32,
        available_output_devices: Vec<DeviceInfo>,
        headroom: Option<HeadroomDataStruct>,
        loudness: Option<LoudnessDataStruct>,
        dynamic_range: Option<DynamicRangeDataStruct>,
        phase_correlation: Option<PhaseCorrelationDataStruct>,
        stereo_width: Option<StereoWidthDataStruct>,
    },
    
    // NEW: Parameters changed (event-driven) - Only when user modifies something
    #[serde(rename = "parameters")]
    ParametersChanged {
        tracks: Option<Vec<TrackParameters>>,
        subgroups: Option<Vec<SubgroupParameters>>,
        auxes: Option<Vec<AuxParameters>>,
        master: Option<MasterParameters>,
    },
    
    #[serde(rename = "fft")]
    FFTData {
        bins_left: Vec<f32>,
        bins_right: Vec<f32>,
        sample_rate: u32,
    },
    #[serde(rename = "track_fft")]
    TrackFFTData {
        track: usize,
        bins_left: Vec<f32>,
        bins_right: Vec<f32>,
        sample_rate: u32,
    },
    #[serde(rename = "performance")]
    PerformanceStats {
        buffer_size: usize,
        sample_rate: u32,
        latency_ms: f32,
        avg_process_ms: f32,
        cpu_percent: f32,
        min_process_ms: f32,
        max_process_ms: f32,
    },
    #[serde(rename = "recording_stats")]
    RecordingStats {
        elapsed_seconds: u64,
        file_size_bytes: u64,
        available_space_gb: f32,
    },
    #[serde(rename = "loudness")]
    LoudnessData {
        momentary_lufs: f32,
        short_term_lufs: f32,
        integrated_lufs: f32,
        loudness_range_lu: f32,
        true_peak_dbtp: f32,
    },
    #[serde(rename = "dynamic_range")]
    DynamicRangeData {
        peak_db_l: f32,
        peak_db_r: f32,
        rms_db_l: f32,
        rms_db_r: f32,
        dynamic_range_l: f32,
        dynamic_range_r: f32,
        dynamic_range_stereo: f32,
    },
    #[serde(rename = "phase_correlation")]
    PhaseCorrelationData {
        correlation: f32,
        mono_compatible: bool,
    },
    #[serde(rename = "stereo_width")]
    StereoWidthData {
        width_percent: f32,
        mid_rms: f32,
        side_rms: f32,
        balance: f32,
    },
    #[serde(rename = "headroom")]
    HeadroomData {
        peak_l: f32,
        peak_r: f32,
        headroom_l: f32,
        headroom_r: f32,
        headroom_stereo: f32,
    },
    #[serde(rename = "license")]
    License {
        key: String,
        license_type: String,
        expires_at: Option<String>,
        is_valid: bool,
    },
    #[serde(rename = "audio_config")]
    AudioConfig {
        sample_rate: u32,  // 0 = Auto
        buffer_size: u32,
    },
}

// === METERS STRUCTS (Real-time data only) ===

#[derive(Debug, Serialize)]
struct TrackMeters {
    track: usize,
    level_l: f32,
    level_r: f32,
    waveform: Vec<f32>,
    phase_correlation: f32,
    compressor_input_db: f32,
    compressor_reduction_db: f32,
    gate_input_db: f32,
    gate_attenuation_db: f32,
    file_ended: bool,
    is_playing: bool,
    // Include essential parameters for remote sync
    gain: f32,
    volume: f32,
    mute: bool,
    pan: f32,
    is_stereo: bool,
    file_name: String,
    file_artist: Option<String>,
    file_title: Option<String>,
}

#[derive(Debug, Serialize)]
struct SubgroupMeters {
    subgroup: usize,
    level_l: f32,
    level_r: f32,
}

#[derive(Debug, Serialize)]
struct AuxMeters {
    aux: usize,
    level_l: f32,
    level_r: f32,
}

#[derive(Debug, Serialize)]
struct HeadroomDataStruct {
    peak_l: f32,
    peak_r: f32,
    headroom_l: f32,
    headroom_r: f32,
    headroom_stereo: f32,
}

// === PARAMETERS STRUCTS (State data, event-driven) ===

#[derive(Debug, Serialize, Clone)]
struct TrackParameters {
    track: usize,
    gain: Option<f32>,
    volume: Option<f32>,
    mute: Option<bool>,
    pan: Option<f32>,
    route_to_master: Option<bool>,
    route_to_subgroups: Option<Vec<usize>>,
    pad_enabled: Option<bool>,
    hpf_enabled: Option<bool>,
    phase_inverted: Option<bool>,
    // Compressor
    compressor_enabled: Option<bool>,
    compressor_threshold_db: Option<f32>,
    compressor_ratio: Option<f32>,
    compressor_attack_ms: Option<f32>,
    compressor_release_ms: Option<f32>,
    // Gate
    gate_enabled: Option<bool>,
    gate_threshold_db: Option<f32>,
    gate_range_db: Option<f32>,
    gate_attack_ms: Option<f32>,
    gate_release_ms: Option<f32>,
    // EQ
    eq_enabled: Option<bool>,
    eq_low: Option<f32>,
    eq_low_mid: Option<f32>,
    eq_high_mid: Option<f32>,
    eq_high: Option<f32>,
    parametric_eq_enabled: Option<bool>,
    eq_filters: Option<Vec<ParametricFilter>>,
    // Aux sends
    aux_sends: Option<Vec<AuxSendData>>,
    // File player
    file_name: Option<String>,
    file_artist: Option<String>,
    file_title: Option<String>,
    is_stereo: Option<bool>,
    // FFT data for parametric EQ visualization
    fft_data: Option<FFTDataSimple>,
}

#[derive(Debug, Serialize, Clone)]
struct FFTDataSimple {
    bins_left: Vec<f32>,
    bins_right: Vec<f32>,
    sample_rate: u32,
}

#[derive(Debug, Serialize, Clone)]
struct SubgroupParameters {
    subgroup: usize,
    gain: Option<f32>,
    mute: Option<bool>,
    route_to_master: Option<bool>,
    selected_output: Option<Option<String>>,
}

#[derive(Debug, Serialize, Clone)]
struct AuxParameters {
    aux: usize,
    gain: Option<f32>,
    mute: Option<bool>,
    route_to_master: Option<bool>,
    route_to_subgroups: Option<Vec<usize>>,
    output_enabled: Option<bool>,
    output_channel_selection_left: Option<u16>,
    output_channel_selection_right: Option<u16>,
    selected_output: Option<Option<String>>,
    reverb: Option<AuxReverbParams>,
    delay: Option<AuxDelayParams>,
}

/// Master FX effect data for frontend synchronization
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
enum MasterFxEffect {
    #[serde(rename = "compressor")]
    Compressor {
        enabled: bool,
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
    },
    #[serde(rename = "limiter")]
    Limiter {
        enabled: bool,
        threshold: f32,
        release: f32,
    },
    #[serde(rename = "delay")]
    Delay {
        enabled: bool,
        time_l: f32,
        time_r: f32,
        feedback: f32,
        mix: f32,
    },
    #[serde(rename = "reverb")]
    Reverb {
        enabled: bool,
        room_size: f32,
        damping: f32,
        wet: f32,
        width: f32,
    },
}

#[derive(Debug, Serialize)]
struct MasterParameters {
    gain: Option<f32>,
    gain_left: Option<f32>,
    gain_right: Option<f32>,
    mute: Option<bool>,
    linked: Option<bool>,
    eq_filters: Option<Vec<ParametricFilter>>,
    fx_effects: Option<Vec<MasterFxEffect>>,
    selected_output: Option<Option<String>>,
    available_output_devices: Option<Vec<DeviceInfo>>,
}

#[derive(Debug, Serialize, Clone)]
struct AuxSendData {
    level: f32,
    pre_fader: bool,
    muted: bool,
}

#[derive(Debug, Serialize, Clone)]
struct AuxReverbParams {
    enabled: bool,
    room_size: f32,
    damping: f32,
    wet: f32,
    width: f32,
}

#[derive(Debug, Serialize, Clone)]
struct AuxDelayParams {
    enabled: bool,
    delay_time_l_ms: f32,
    delay_time_r_ms: f32,
    feedback: f32,
    mix: f32,
}

#[derive(Debug, Serialize, Clone)]
struct LoudnessDataStruct {
    momentary_lufs: f32,
    short_term_lufs: f32,
    integrated_lufs: f32,
    loudness_range_lu: f32,
    true_peak_dbtp: f32,
}

#[derive(Debug, Serialize, Clone)]
struct DynamicRangeDataStruct {
    peak_db_l: f32,
    peak_db_r: f32,
    rms_db_l: f32,
    rms_db_r: f32,
    dynamic_range_l: f32,
    dynamic_range_r: f32,
    dynamic_range_stereo: f32,
}

#[derive(Debug, Serialize, Clone)]
struct PhaseCorrelationDataStruct {
    correlation: f32,
    mono_compatible: bool,
}

#[derive(Debug, Serialize, Clone)]
struct StereoWidthDataStruct {
    width_percent: f32,
    mid_rms: f32,
    side_rms: f32,
    balance: f32,
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

/// Get available disk space in GB for the given path
fn get_available_disk_space_gb(path: &PathBuf) -> f32 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Get the parent directory (recordings folder)
        let dir = path.parent().unwrap_or(path.as_path());
        
        // Use df -k to get available space in KB
        match Command::new("df")
            .arg("-k")
            .arg(dir)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse output: df -k returns KB in 4th column of last line
                // Example: /dev/disk1s1  488555536 123456789 364098747    26%    1234567  9876543210   0%   /System/Volumes/Data
                if let Some(line) = stdout.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Ok(available_kb) = parts[3].parse::<f64>() {
                            return (available_kb / (1024.0 * 1024.0)) as f32;
                        }
                    }
                }
                0.0
            }
            Err(_) => 0.0,
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        let dir = path.parent().unwrap_or(path.as_path());
        
        match Command::new("df")
            .arg("-k")
            .arg(dir)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Ok(available_kb) = parts[3].parse::<f64>() {
                            return (available_kb / (1024.0 * 1024.0)) as f32;
                        }
                    }
                }
                0.0
            }
            Err(_) => 0.0,
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Get drive letter
        let path_str = path.to_string_lossy();
        if path_str.len() >= 2 {
            let drive = &path_str[0..2];
            
            // Use wmic to get free space
            match Command::new("wmic")
                .args(&["logicaldisk", "where", &format!("DeviceID='{}'", drive), "get", "FreeSpace"])
                .output()
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if let Ok(free_bytes) = line.trim().parse::<f64>() {
                            return (free_bytes / (1024.0 * 1024.0 * 1024.0)) as f32;
                        }
                    }
                    0.0
                }
                Err(_) => 0.0,
            }
        } else {
            0.0
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        0.0
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
    active_stream_id: Arc<AtomicUsize>, // ID of the currently active output stream
    input_buffer: Arc<Mutex<Vec<f32>>>, // Shared buffer - always contains latest input frame
    input_channels: Arc<AtomicUsize>,
    input_users: HashSet<usize>, // Track IDs that are using audio input
    current_input_device: Option<String>, // Currently open input device name
    master_tap_buffer: Arc<Mutex<Vec<f32>>>, // Master output tap for recording (stereo interleaved)
    master_tap_enabled: Arc<AtomicBool>, // Enable/disable master tap
    recording_path: Arc<Mutex<Option<PathBuf>>>, // Path where to save the recording
    recording_start_time: Arc<Mutex<Option<Instant>>>, // Recording start time for elapsed calculation
    recording_last_stats_time: Arc<Mutex<Option<Instant>>>, // Last time stats were sent (for 1-second interval)
    recording_sample_rate: Arc<Mutex<u32>>, // Recording sample rate (configurable)
    recording_bit_depth: Arc<Mutex<u32>>, // Recording bit depth (16, 24, or 32)
    recording_format: Arc<Mutex<String>>, // Recording format ("wav", "mp3", "opus")
    output_sender: mpsc::SyncSender<String>, // Non-blocking channel for sending updates to frontend
    ndi_stream: Arc<NdiStream>, // NDI audio streaming
    selected_master_output: Arc<Mutex<Option<String>>>, // Selected master output device ID
    available_output_devices: Vec<DeviceInfo>, // List of available output devices
}

impl AudioEngine {
    fn new() -> Self {
        let audio_io = AudioIO::new();
        
        // Load license to determine number of aux buses
        let num_aux_buses = match load_license_from_file() {
            Ok(license) => {
                if license.license_type == "full" {
                    6 // Full license gets 6 aux buses
                } else {
                    1 // Demo and other licenses get 1 aux bus
                }
            }
            Err(_) => 1 // Default to 1 aux bus if license can't be loaded
        };
        
        let router = Arc::new(Mutex::new(Router::new(24, num_aux_buses))); // Support up to 24 tracks + dynamic aux count
        let updates_suspended = Arc::new(AtomicBool::new(false));
        let input_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let input_channels = Arc::new(AtomicUsize::new(2)); // Default stereo
        let master_tap_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(4800000))); // ~100 sec @ 48kHz stereo
        let master_tap_enabled = Arc::new(AtomicBool::new(false));
        let recording_path = Arc::new(Mutex::new(None));
        let recording_start_time = Arc::new(Mutex::new(None));
        let recording_last_stats_time = Arc::new(Mutex::new(None));
        let recording_sample_rate = Arc::new(Mutex::new(48000)); // Default 48kHz
        let recording_bit_depth = Arc::new(Mutex::new(16)); // Default 16-bit
        let recording_format = Arc::new(Mutex::new("wav".to_string())); // Default WAV
        let active_stream_id = Arc::new(AtomicUsize::new(0)); // Start with stream ID 0

        // Create non-blocking channel for audio updates
        // Capacity of 100 messages allows some buffering without accumulating too much latency
        let (output_sender, output_receiver) = mpsc::sync_channel::<String>(100);

        // Spawn dedicated I/O thread to handle stdout writes without blocking audio thread
        thread::spawn(move || {
            while let Ok(message) = output_receiver.recv() {
                println!("{}", message);
            }
            // Channel closed, thread exits
            eprintln!("[AudioEngine] Output thread terminated");
        });

        let ndi_stream = Arc::new(NdiStream::new());

        Self {
            audio_io,
            router,
            input_stream: None,
            output_stream: None,
            sample_rate: 48000, // Default, will be overwritten by device native rate
            input_sample_rate: 48000, // Default, will be overwritten when input opens
            output_buffer_size: None,
            updates_suspended,
            active_stream_id,
            input_buffer,
            input_channels,
            input_users: HashSet::new(),
            current_input_device: None,
            master_tap_buffer,
            master_tap_enabled,
            recording_path,
            recording_start_time,
            recording_last_stats_time,
            recording_sample_rate,
            recording_bit_depth,
            recording_format,
            output_sender,
            ndi_stream,
            selected_master_output: Arc::new(Mutex::new(None)),
            available_output_devices: Vec::new(),
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
        
        // Load saved audio configuration if not explicitly specified
        let (final_sample_rate, final_buffer_size) = if sample_rate.is_none() && buffer_size.is_none() {
            // Both not specified - load from file
            let config = load_audio_config_from_file();
            let sr = if config.sample_rate == 0 { None } else { Some(config.sample_rate) };
            let bs = if config.buffer_size == 0 { None } else { Some(config.buffer_size) };
            (sr, bs)
        } else {
            // At least one was specified - use what was provided
            (sample_rate, buffer_size)
        };
        
        // Force stop if streams are still active (restart scenario)
        if self.input_stream.is_some() || self.output_stream.is_some() {
            eprintln!("[Engine] Streams still active, forcing stop before restart...");
            
            // CRITICAL: Suspend updates to prevent race conditions during device/sample rate change
            // This stops the audio callback from processing while we're changing sample rates
            self.updates_suspended.store(true, Ordering::Relaxed);
            eprintln!("[Engine] Audio processing suspended");
            
            // Wait for any in-flight callbacks to complete
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // CRITICAL: Call pause() BEFORE dropping to actually stop the stream
            if let Some(stream) = &self.input_stream {
                let _ = stream.pause();
                eprintln!("[Engine] Input stream paused");
            }
            if let Some(stream) = &self.output_stream {
                let _ = stream.pause();
                eprintln!("[Engine] Output stream paused");
            }
            
            // Wait for pause to take effect
            std::thread::sleep(std::time::Duration::from_millis(50));
            
            // Now drop the streams
            {
                let _input = self.input_stream.take();
                let _output = self.output_stream.take();
                // Drops happen here when variables go out of scope
            }
            eprintln!("[Engine] Streams dropped");
            
            // Clear input buffer when stopping
            if let Ok(mut buffer) = self.input_buffer.lock() {
                buffer.clear();
            }
            
            // Clear master tap buffer to avoid old audio data
            if let Ok(mut buffer) = self.master_tap_buffer.lock() {
                buffer.clear();
            }
            
            // Wait for OS to release audio hardware
            std::thread::sleep(std::time::Duration::from_millis(100));
            eprintln!("[Engine] System cleanup complete");
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
        // let input_config = self.audio_io.get_supported_config(&input_device, true, final_sample_rate, final_buffer_size)?;
        let output_config = self.audio_io.get_supported_config(&output_device, false, final_sample_rate, final_buffer_size)?;

        // Check if sample rate changed
        let old_sample_rate = self.sample_rate;
        let new_sample_rate = output_config.sample_rate.0;
        let sample_rate_changed = old_sample_rate != new_sample_rate;
        
        if sample_rate_changed {
            eprintln!("[Engine] Sample rate changing from {} Hz to {} Hz", old_sample_rate, new_sample_rate);
        }

        self.sample_rate = new_sample_rate;
        
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
                // Note: set_sample_rate already calls player.set_output_sample_rate()
                // which resets resample_position to avoid pitch/speed issues
            }
            
            // Update aux buses
            for aux_bus in router.aux_buses.iter_mut() {
                aux_bus.set_sample_rate(self.sample_rate as f32);
            }
            
            // Update master bus (EQ + FX chain)
            router.master.parametric_eq.set_sample_rate(self.sample_rate as f32);
            router.master.set_sample_rate(self.sample_rate as f32);
            
            // Log sample rate change completion
            if sample_rate_changed {
                eprintln!("[Engine] All components updated to {} Hz", self.sample_rate);
            }
        }

        // Update NDI stream sample rate
        let _ = self.ndi_stream.set_sample_rate(self.sample_rate);

        let input_channels = 2; // Default stereo (not used since input is disabled)

        let err_fn = |err| eprintln!("[Engine] Stream error: {}", err);

        // Clone router for callbacks
        let router_output = Arc::clone(&self.router);
        let updates_suspended_flag = Arc::clone(&self.updates_suspended);

        // Meter update counter and interval (send levels every 50ms at 48kHz = 2400 frames for 20 FPS)
        let meter_update_frames = Arc::new(Mutex::new(0_usize));
        let meter_interval = 2400_usize;

        // Performance tracking
        let perf_stats = Arc::new(Mutex::new(PerformanceStats::new()));
        let perf_stats_clone = Arc::clone(&perf_stats);
        let sample_rate_for_perf = self.sample_rate; // Save sample rate for performance calculation
        let buffer_size_for_perf = actual_buffer_size; // Save configured buffer size for stats
        
        // Stream ID for debugging (increment for each new stream)
        use std::sync::atomic::AtomicUsize;
        static STREAM_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let stream_id = STREAM_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Mark this stream as the active one
        self.active_stream_id.store(stream_id, Ordering::SeqCst);
        
        // Collect available output devices BEFORE cloning for callback
        if let Ok(devices) = self.list_devices() {
            // Filter to only output-capable devices
            self.available_output_devices = devices.into_iter()
                .filter(|d| d.output_channels > 0)
                .collect();
        }
        
        // Clone active_stream_id for callback to check if it's still the active stream
        let active_stream_id_check = Arc::clone(&self.active_stream_id);

        // === INPUT STREAM: capture audio ===
        // Input stream is now managed on-demand via open_audio_input()/close_audio_input()
        // when tracks select audio input sources. This preserves privacy by not opening
        // the microphone unless actually needed.
        
        // Share input buffer with output callback
        let input_buffer = Arc::clone(&self.input_buffer);
        let input_channels = Arc::clone(&self.input_channels);
        let master_tap_buffer = Arc::clone(&self.master_tap_buffer);
        let master_tap_enabled = Arc::clone(&self.master_tap_enabled);
        let recording_start_time = Arc::clone(&self.recording_start_time);
        let recording_last_stats_time = Arc::clone(&self.recording_last_stats_time);
        let recording_path = Arc::clone(&self.recording_path);
        let recording_bit_depth = Arc::clone(&self.recording_bit_depth);
        let ndi_stream = Arc::clone(&self.ndi_stream);
        
        // Clone Arc for selected output (thread-safe shared access)
        let selected_master_output = Arc::clone(&self.selected_master_output);
        
        // Clone available output devices for thread-safe access in closure
        let available_output_devices = self.available_output_devices.clone();
        
        // Clone output sender for non-blocking updates
        let output_sender = self.output_sender.clone();

        // === OUTPUT STREAM: process and output audio ===
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // CRITICAL: Check if this is still the active stream
                // If another stream was created, this one should die silently
                let current_active = active_stream_id_check.load(Ordering::SeqCst);
                if stream_id != current_active {
                    // This is an old stream that should be dead - output silence and return
                    data.fill(0.0);
                    return;
                }
                
                // CRITICAL: Check if updates are suspended (during sample rate change)
                // If suspended, output silence to prevent race conditions
                if updates_suspended_flag.load(Ordering::Relaxed) {
                    data.fill(0.0);
                    return;
                }
                
                // Start performance measurement
                let start_time = Instant::now();
                
                let frames = data.len() / output_channels;
                let input_ch_count = input_channels.load(Ordering::Relaxed);
                
                // Lock input buffer and copy the data (keep lock time minimal)
                let input_buf = input_buffer.lock().unwrap();

                // Acquire lock, process audio, release lock quickly
                let levels_to_send = {
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
                        
                        // Process master audio through loudness meter (EBU R128)
                        router.loudness_meter.process(master_l, master_r);
                        
                        // Process master audio through dynamic range meter
                        router.dynamic_range_meter.process(master_l, master_r);
                        
                        // Process master audio through phase correlation meter
                        router.phase_correlation_meter.process(master_l, master_r);
                        
                        // Process master audio through stereo width meter
                        router.stereo_width_meter.process(master_l, master_r);
                        
                        // Process master audio through headroom meter
                        router.headroom_meter.process(master_l, master_r);

                        // Record master output for this frame (if recording enabled)
                        if master_tap_enabled.load(Ordering::Relaxed) {
                            if let Ok(mut tap_buffer) = master_tap_buffer.try_lock() {
                                let max_samples = sample_rate_for_perf as usize * 2 * 600; // 10 min stereo
                                if tap_buffer.len() < max_samples {
                                    tap_buffer.push(master_left.clamp(-1.0, 1.0));
                                    tap_buffer.push(master_right.clamp(-1.0, 1.0));
                                }
                            }
                        }

                        // Send audio to NDI stream (if active)
                        // Get the selected source audio
                        if ndi_stream.is_active() {
                            use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
                            static NDI_FIRST_SAMPLE: AtomicBool = AtomicBool::new(true);
                            static NDI_SAMPLE_COUNT: AtomicU64 = AtomicU64::new(0);
                            let ndi_source = ndi_stream.get_source();
                            let (ndi_l, ndi_r) = match ndi_source {
                                ndi_stream::NdiSource::Master => {
                                    // Use last_master_output which has the final processed audio
                                    router.last_master_output
                                },
                                ndi_stream::NdiSource::Subgroup(id) => {
                                    // Find subgroup by ID (IDs are 1-based, array is 0-based)
                                    let idx = id.saturating_sub(1);
                                    if idx < router.last_subgroup_outputs.len() {
                                        router.last_subgroup_outputs[idx]
                                    } else {
                                        (0.0, 0.0)
                                    }
                                },
                            };
                            
                            // Send stereo pair to NDI
                            let ndi_samples = [ndi_l.clamp(-1.0, 1.0), ndi_r.clamp(-1.0, 1.0)];
                            let _ = ndi_stream.send_audio(&ndi_samples);
                        }

                        // Initialize output frame to silence
                        let out_frame_start = frame_idx * output_channels;
                        for ch in 0..output_channels {
                            data[out_frame_start + ch] = 0.0;
                        }

                        // Write master output to its channels
                        let master_left_ch = router.master.output_channel_selection.left as usize;
                        let master_right_ch = router.master.output_channel_selection.right as usize;
                        if master_left_ch < output_channels {
                            data[out_frame_start + master_left_ch] += master_right.clamp(-1.0, 1.0); // SWAPPED: was master_left
                        }
                        if master_right_ch < output_channels {
                            data[out_frame_start + master_right_ch] += master_left.clamp(-1.0, 1.0); // SWAPPED: was master_right
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
                                
                                // If both channels are the same, send mono mix to single channel
                                if aux_left_ch == aux_right_ch {
                                    let mono = (aux_l + aux_r) * 0.5;
                                    if aux_left_ch < output_channels {
                                        data[out_frame_start + aux_left_ch] += mono.clamp(-1.0, 1.0);
                                    }
                                } else {
                                    // Send stereo to different channels
                                    if aux_left_ch < output_channels {
                                        data[out_frame_start + aux_left_ch] += aux_l.clamp(-1.0, 1.0);
                                    }
                                    if aux_right_ch < output_channels {
                                        data[out_frame_start + aux_right_ch] += aux_r.clamp(-1.0, 1.0);
                                    }
                                }
                            }
                        }
                    }

                    // Check if we need to send meter updates
                    let mut counter = meter_update_frames.lock().unwrap();
                    *counter += frames;
                    
                    let levels_to_send = if *counter >= meter_interval {
                        *counter = 0;
                        
                        // Build meter structs directly (optimized - no legacy intermediate structs)
                        let track_meters: Vec<TrackMeters> = router.tracks.iter()
                            .map(|t| TrackMeters {
                                track: t.id,
                                level_l: t.level_l,
                                level_r: t.level_r,
                                waveform: t.get_waveform_buffer(128),
                                phase_correlation: t.phase_correlation,
                                compressor_input_db: t.compressor.input_level_db,
                                compressor_reduction_db: t.compressor.gain_reduction_db,
                                gate_input_db: t.gate.input_level_db,
                                gate_attenuation_db: t.gate.attenuation_db,
                                file_ended: t.file_player.as_ref().map_or(false, |p| p.file_ended),
                                is_playing: t.file_player.as_ref().map_or(false, |p| p.playing),
                                // Include essential parameters for remote sync
                                gain: t.gain,
                                volume: t.volume,
                                mute: t.mute,
                                pan: t.pan,
                                is_stereo: t.file_player.as_ref().map_or(false, |p| p.channels >= 2),
                                file_name: t.file_player.as_ref().map_or(String::new(), |p| p.file_name.clone()),
                                file_artist: t.file_player.as_ref().and_then(|p| p.file_artist.clone()),
                                file_title: t.file_player.as_ref().and_then(|p| p.file_title.clone()),
                            })
                            .collect();
                        
                        let subgroup_meters: Vec<SubgroupMeters> = router.subgroups.iter()
                            .map(|sg| SubgroupMeters {
                                subgroup: sg.id,
                                level_l: sg.level_l,
                                level_r: sg.level_r,
                            })
                            .collect();
                        
                        let aux_meters: Vec<AuxMeters> = router.aux_buses.iter()
                            .map(|aux| AuxMeters {
                                aux: aux.id,
                                level_l: aux.level_l,
                                level_r: aux.level_r,
                            })
                            .collect();
                        
                        let master_l = router.master.level_l;
                        let master_r = router.master.level_r;
                        
                        // Collect headroom data
                        let headroom_data = router.headroom_meter.get_measurement();
                        
                        // Collect all other meter data
                        let loudness_meas = router.loudness_meter.get_measurements();
                        let dynamic_range_meas = router.dynamic_range_meter.get_measurements();
                        let phase_correlation_meas = router.phase_correlation_meter.get_measurement();
                        let stereo_width_meas = router.stereo_width_meter.get_measurement();
                        
                        // Convert to serializable structs
                        let loudness_data = LoudnessDataStruct {
                            momentary_lufs: loudness_meas.momentary,
                            short_term_lufs: loudness_meas.short_term,
                            integrated_lufs: loudness_meas.integrated,
                            loudness_range_lu: loudness_meas.loudness_range,
                            true_peak_dbtp: loudness_meas.true_peak_dbtp,
                        };
                        
                        let dynamic_range_data = DynamicRangeDataStruct {
                            peak_db_l: dynamic_range_meas.peak_db_l,
                            peak_db_r: dynamic_range_meas.peak_db_r,
                            rms_db_l: dynamic_range_meas.rms_db_l,
                            rms_db_r: dynamic_range_meas.rms_db_r,
                            dynamic_range_l: dynamic_range_meas.dynamic_range_l,
                            dynamic_range_r: dynamic_range_meas.dynamic_range_r,
                            dynamic_range_stereo: dynamic_range_meas.dynamic_range_stereo,
                        };
                        
                        let phase_correlation_data = PhaseCorrelationDataStruct {
                            correlation: phase_correlation_meas.correlation,
                            mono_compatible: phase_correlation_meas.mono_compatible,
                        };
                        
                        let stereo_width_data = StereoWidthDataStruct {
                            width_percent: stereo_width_meas.width_percent,
                            mid_rms: stereo_width_meas.mid_rms,
                            side_rms: stereo_width_meas.side_rms,
                            balance: stereo_width_meas.balance,
                        };
                        
                        // Reset peak levels after reading
                        router.master.reset_levels();
                        for track in router.tracks.iter_mut() {
                            track.reset_levels();
                        }
                        for subgroup in router.subgroups.iter_mut() {
                            subgroup.reset_levels();
                        }
                        
                        // Return meter data to serialize outside the lock
                        Some((
                            track_meters,
                            subgroup_meters,
                            aux_meters,
                            master_l,
                            master_r,
                            headroom_data,
                            loudness_data,
                            dynamic_range_data,
                            phase_correlation_data,
                            stereo_width_data
                        ))
                    } else {
                        None
                    };

                    levels_to_send
                }; // Lock is released here
                
                // Check for FFT data outside the lock
                let fft_data = {
                    let mut router = router_output.lock().unwrap();
                    router.fft_analyzer.analyze()
                };
                
                // Check for track FFT data outside the lock
                // Generate FFT for all tracks with audio loaded (ignore mute status)
                // This allows EQ editing with FFT visualization even on muted tracks
                let track_fft_data: Vec<(usize, Vec<f32>, Vec<f32>)> = {
                    let mut router = router_output.lock().unwrap();
                    router.tracks.iter_mut()
                        .filter(|t| t.source != crate::routing::TrackSource::None)
                        .filter_map(|t| {
                            t.fft_analyzer.analyze().map(|(left, right)| (t.id, left, right))
                        })
                        .collect()
                };
                
                // CRITICAL: Check if updates are suspended (during window resize)
                // This prevents blocking I/O on stdout which would freeze audio
                let suspended = updates_suspended_flag.load(Ordering::Relaxed);
                
                if !suspended {
                    // Send meter updates outside the lock
                    if let Some((
                        track_meters,
                        subgroup_meters,
                        aux_meters,
                        master_l,
                        master_r,
                        headroom_data,
                        loudness_data,
                        dynamic_range_data,
                        phase_correlation_data,
                        stereo_width_data
                    )) = levels_to_send {
                        let headroom_struct = HeadroomDataStruct {
                            peak_l: headroom_data.peak_l,
                            peak_r: headroom_data.peak_r,
                            headroom_l: headroom_data.headroom_l,
                            headroom_r: headroom_data.headroom_r,
                            headroom_stereo: headroom_data.headroom_stereo,
                        };
                        
                        let response_optimized = Response::Meters {
                            tracks: track_meters,
                            subgroups: subgroup_meters,
                            auxes: aux_meters,
                            master_l,
                            master_r,
                            available_output_devices: available_output_devices.clone(),
                            headroom: Some(headroom_struct),
                            loudness: Some(loudness_data),
                            dynamic_range: Some(dynamic_range_data),
                            phase_correlation: Some(phase_correlation_data),
                            stereo_width: Some(stereo_width_data),
                        };
                        
                        if let Ok(json) = serde_json::to_string(&response_optimized) {
                            // Use try_send to avoid blocking audio thread if channel is full
                            let _ = output_sender.try_send(json);
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
                            // Use try_send to avoid blocking audio thread if channel is full
                            let _ = output_sender.try_send(json);
                        }
                    }

                    // Send track FFT data for tracks in play
                    for (track_id, bins_left, bins_right) in track_fft_data {
                        let response = Response::TrackFFTData {
                            track: track_id,
                            bins_left,
                            bins_right,
                            sample_rate: sample_rate_for_perf,
                        };
                        
                        if let Ok(json) = serde_json::to_string(&response) {
                            let _ = output_sender.try_send(json);
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
                    
                    // Use configured buffer size for stats display instead of variable callback frames
                    let (stats_buffer_size, stats_latency_ms) = match buffer_size_for_perf {
                        cpal::BufferSize::Fixed(size) => {
                            let latency = (size as f32 / sample_rate_for_perf as f32) * 1000.0;
                            (size as usize, latency)
                        }
                        cpal::BufferSize::Default => (frames, buffer_latency_ms),
                    };
                    
                    let response = Response::PerformanceStats {
                        buffer_size: stats_buffer_size,
                        sample_rate: sample_rate_for_perf,
                        latency_ms: stats_latency_ms,
                        avg_process_ms: avg_ms,
                        cpu_percent: avg_cpu,
                        min_process_ms: min_ms,
                        max_process_ms: max_ms,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&response) {
                        // Use try_send to avoid blocking audio thread if channel is full
                        let _ = output_sender.try_send(json);
                    }
                    
                    stats.reset();
                }
                
                // Send recording stats every 1 second (only if recording is enabled)
                if master_tap_enabled.load(Ordering::Relaxed) {
                    if let (Ok(start_time), Ok(mut last_stats_time)) = (
                        recording_start_time.lock(),
                        recording_last_stats_time.lock()
                    ) {
                        if let (Some(start), Some(last)) = (*start_time, *last_stats_time) {
                            let now = Instant::now();
                            let elapsed_since_last = now.duration_since(last);
                            
                            // Send stats every 1 second
                            if elapsed_since_last.as_secs() >= 1 {
                                let elapsed_seconds = now.duration_since(start).as_secs();
                                
                                // Calculate file size (stereo interleaved samples, saved as 16-bit WAV)
                                let num_samples = if let Ok(buffer) = master_tap_buffer.try_lock() {
                                    buffer.len() as u64
                                } else {
                                    0
                                };
                                
                                // Get configured bit depth to calculate accurate file size
                                let bytes_per_sample = if let Ok(bd) = recording_bit_depth.lock() {
                                    match *bd {
                                        16 => 2,
                                        24 => 3,
                                        32 => 4,
                                        _ => 2, // fallback to 16-bit
                                    }
                                } else {
                                    2 // fallback to 16-bit
                                };
                                let file_size_bytes = num_samples * bytes_per_sample;
                                
                                // Get available disk space for the recordings directory
                                let available_space_gb = if let Ok(path) = recording_path.lock() {
                                    if let Some(ref p) = *path {
                                        get_available_disk_space_gb(p)
                                    } else {
                                        0.0
                                    }
                                } else {
                                    0.0
                                };
                                
                                let response = Response::RecordingStats {
                                    elapsed_seconds,
                                    file_size_bytes,
                                    available_space_gb,
                                };
                                
                                if let Ok(json) = serde_json::to_string(&response) {
                                    // Use try_send to avoid blocking audio thread if channel is full
                                    let _ = output_sender.try_send(json);
                                }
                                
                                // Update last stats time
                                *last_stats_time = Some(now);
                            }
                        }
                    }
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
        
        // Wait before resuming to ensure stream stability
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Resume audio processing now that new streams are active
        self.updates_suspended.store(false, Ordering::Relaxed);

        // Send immediate performance stats with actual configuration
        // This updates the UI footer immediately instead of waiting 2-3 seconds
        let buffer_size_val = match actual_buffer_size {
            cpal::BufferSize::Fixed(size) => size as usize,
            cpal::BufferSize::Default => 256, // Fallback estimate
        };
        let latency_ms_val = (buffer_size_val as f32 / self.sample_rate as f32) * 1000.0;
        
        let initial_stats = Response::PerformanceStats {
            buffer_size: buffer_size_val,
            sample_rate: self.sample_rate,
            latency_ms: latency_ms_val,
            avg_process_ms: 0.0,  // Will be updated after first buffers
            cpu_percent: 0.0,     // Will be updated after first buffers
            min_process_ms: 0.0,
            max_process_ms: 0.0,
        };
        
        if let Ok(json) = serde_json::to_string(&initial_stats) {
            let _ = self.output_sender.send(json);
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.input_stream.take() {
            drop(stream);
        }
        if let Some(stream) = self.output_stream.take() {
            drop(stream);
        }
        
        // Reset input state
        self.input_users.clear();
        self.current_input_device = None;
        
        // Clear buffers
        if let Ok(mut buffer) = self.input_buffer.lock() {
            buffer.clear();
        }
        if let Ok(mut buffer) = self.master_tap_buffer.lock() {
            buffer.clear();
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
            return Ok(());
        }

        // Get input device
        let input_device = if let Some(name) = &requested_device {
            self.audio_io.find_device_by_name(name, true)?
        } else {
            self.audio_io.default_input_device()?
        };

        // Use device native sample rate and buffer size for best compatibility
        // BUT: Force input to match output sample rate to avoid resampling issues
        // AND: Use same buffer size as output for synchronized callbacks
        let buffer_size = self.output_buffer_size;
        
        let input_config = self.audio_io.get_supported_config(&input_device, true, Some(self.sample_rate), buffer_size)?;
        
        if let Some(size) = buffer_size {
            eprintln!("[Engine] Input buffer size: {} frames (matched to output)", size);
        } else {
            eprintln!("[Engine] Input buffer size: DEFAULT (matched to output)");
        }
        
        // Store input sample rate
        self.input_sample_rate = input_config.sample_rate.0;
        self.input_channels.store(input_config.channels as usize, Ordering::Relaxed);
        
        let input_buffer_clone = Arc::clone(&self.input_buffer);
        
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
                
        Ok(())
    }

    fn close_audio_input(&mut self, track_id: usize) -> Result<()> {
        // Remove track from users set
        self.input_users.remove(&track_id);
        
        // Only actually close the stream when no users remain
        if self.input_users.is_empty() {
            if let Some(stream) = self.input_stream.take() {
                drop(stream);
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
    fn enable_master_tap(&self, file_path: String, _sample_rate: u32, bit_depth: u32, format: &str) {
        // Clear previous buffer
        if let Ok(mut buffer) = self.master_tap_buffer.lock() {
            buffer.clear();
        }
        // Set recording path
        if let Ok(mut path) = self.recording_path.lock() {
            *path = Some(PathBuf::from(file_path));
        }
        // Set recording parameters
        // Note: We always record at the audio device's sample rate (self.sample_rate)
        // because we capture samples directly from the audio callback.
        // The requested sample_rate is ignored to avoid quality loss from resampling.
        if let Ok(mut sr) = self.recording_sample_rate.lock() {
            *sr = self.sample_rate; // Use device sample rate, not requested rate
        }
        if let Ok(mut bd) = self.recording_bit_depth.lock() {
            *bd = bit_depth;
        }
        if let Ok(mut fmt) = self.recording_format.lock() {
            *fmt = format.to_string();
        }
        // Set start time
        let now = Instant::now();
        if let Ok(mut start_time) = self.recording_start_time.lock() {
            *start_time = Some(now);
        }
        if let Ok(mut last_stats_time) = self.recording_last_stats_time.lock() {
            *last_stats_time = Some(now);
        }
        self.master_tap_enabled.store(true, Ordering::Relaxed);
    }

    fn disable_master_tap(&self) {
        self.master_tap_enabled.store(false, Ordering::Relaxed);
        
        // Clear recording times
        if let Ok(mut start_time) = self.recording_start_time.lock() {
            *start_time = None;
        }
        if let Ok(mut last_stats_time) = self.recording_last_stats_time.lock() {
            *last_stats_time = None;
        }
        
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

        // Get recording parameters
        let sample_rate = if let Ok(sr) = self.recording_sample_rate.lock() {
            *sr
        } else {
            48000
        };
        let bit_depth = if let Ok(bd) = self.recording_bit_depth.lock() {
            *bd
        } else {
            16
        };

        // Save WAV file if we have samples and path
        if !samples.is_empty() && path.is_some() {
            let file_path = path.unwrap();
            match self.write_wav_file(&file_path, &samples, sample_rate, bit_depth) {
                Ok(_) => {},
                Err(e) => eprintln!("[Engine] ✗ Failed to save recording: {}", e),
            }
        } else {
            eprintln!("[Engine] ✓ Master tap disabled (no recording to save)");
        }
    }

    fn write_wav_file(&self, path: &PathBuf, samples: &[f32], sample_rate: u32, bit_depth: u32) -> Result<()> {
        let mut file = File::create(path)?;
        
        let num_samples = samples.len();
        let num_channels = 2u16; // Stereo
        let bits_per_sample = bit_depth as u16;
        let bytes_per_sample = bits_per_sample / 8;
        let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
        let block_align = num_channels * bytes_per_sample;
        let data_size = num_samples as u32 * bytes_per_sample as u32;
        
        // For 32-bit float, we use format code 3 (IEEE float), otherwise format code 1 (PCM)
        let format_code = if bit_depth == 32 { 3u16 } else { 1u16 };
        
        // Write WAV header
        file.write_all(b"RIFF")?;
        file.write_all(&(36 + data_size).to_le_bytes())?;
        file.write_all(b"WAVE")?;
        
        // fmt chunk
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // chunk size
        file.write_all(&format_code.to_le_bytes())?; // PCM (1) or IEEE Float (3)
        file.write_all(&num_channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&block_align.to_le_bytes())?;
        file.write_all(&bits_per_sample.to_le_bytes())?;
        
        // data chunk
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;
        
        // Write samples based on bit depth
        match bit_depth {
            16 => {
                // Convert f32 to i16
                for sample in samples {
                    let s = sample.max(-1.0).min(1.0);
                    let i16_sample = if s < 0.0 {
                        (s * 32768.0) as i16
                    } else {
                        (s * 32767.0) as i16
                    };
                    file.write_all(&i16_sample.to_le_bytes())?;
                }
            },
            24 => {
                // Convert f32 to i24 (stored as 3 bytes)
                for sample in samples {
                    let s = sample.max(-1.0).min(1.0);
                    let i32_sample = if s < 0.0 {
                        (s * 8388608.0) as i32  // 2^23
                    } else {
                        (s * 8388607.0) as i32
                    };
                    // Write only the lower 3 bytes (little-endian)
                    let bytes = i32_sample.to_le_bytes();
                    file.write_all(&bytes[0..3])?;
                }
            },
            32 => {
                // Write f32 directly (IEEE float format)
                for sample in samples {
                    file.write_all(&sample.to_le_bytes())?;
                }
            },
            _ => {
                return Err(anyhow::anyhow!("Unsupported bit depth: {}", bit_depth));
            }
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

    fn set_signal_frequency(&mut self, track: usize, frequency: f32) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            if let Some(ref mut generator) = t.signal_generator {
                generator.set_frequency(frequency);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Track {} has no signal generator", track))
            }
        } else {
            Err(anyhow::anyhow!("Track {} not found", track))
        }
    }

    fn set_signal_waveform(&mut self, track: usize, waveform: &str) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            if let Some(ref mut generator) = t.signal_generator {
                let wave = match waveform.to_lowercase().as_str() {
                    "sine" => WaveformType::Sine,
                    "square" => WaveformType::Square,
                    "sawtooth" => WaveformType::Sawtooth,
                    "triangle" => WaveformType::Triangle,
                    "white" => WaveformType::WhiteNoise,
                    "pink" => WaveformType::PinkNoise,
                    _ => return Err(anyhow::anyhow!("Unknown waveform: {}", waveform)),
                };
                generator.set_waveform(wave);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Track {} has no signal generator", track))
            }
        } else {
            Err(anyhow::anyhow!("Track {} not found", track))
        }
    }

    fn clear_track_source(&mut self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::clear_source(&mut router, track)
    }

    fn set_track_source_file(&mut self, track: usize, file_path: &str, artist: Option<&str>, title: Option<&str>) -> Result<()> {
        // Close input stream when track switches away from audio input
        if let Err(e) = self.close_audio_input(track) {
            eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
        }
        
        // CRITICAL: Load file WITHOUT holding router lock to avoid audio dropouts
        // This operation can take 100-500ms for large files
        let mut player = file_player::AudioFilePlayer::new();
        player.load_file(file_path)?;
        player.set_output_sample_rate(self.sample_rate);
        
        // Set metadata if provided
        if let Some(a) = artist {
            player.file_artist = Some(a.to_string());
        }
        if let Some(t) = title {
            player.file_title = Some(t.to_string());
        }
        
        // Now quickly assign the pre-loaded player to the track (fast operation)
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.set_file_player(player);
            t.source = routing::TrackSource::FilePlayer;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} not found", track))
        }
    }

    fn set_track_source_aux_return(&mut self, track: usize, aux: usize) -> Result<()> {
        // Close input stream when track switches away from audio input
        if let Err(e) = self.close_audio_input(track) {
            eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
        }
        
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.get_track_mut(track) {
            t.source = routing::TrackSource::AuxReturn(aux);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Track {} not found", track))
        }
    }

    fn play_file(&mut self, track: usize, file_path: Option<&str>, artist: Option<&str>, title: Option<&str>) -> Result<()> {
        // If file_path is provided, set the source file first
        if let Some(path) = file_path {
            self.set_track_source_file(track, path, artist, title)?;
        }
        
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

    fn set_phase_invert(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_phase_invert(&mut router, track, enabled);
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
        // When setting unified gain, also update left/right
        router.master.gain_left = gain.max(0.0);
        router.master.gain_right = gain.max(0.0);
        let gain_db = if gain > 0.0 { 20.0 * gain.log10() } else { -90.0 };
    }

    fn set_master_gain_left(&self, gain: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.gain_left = gain.max(0.0);
    }

    fn set_master_gain_right(&self, gain: f32) {
        let mut router = self.router.lock().unwrap();
        router.master.gain_right = gain.max(0.0);
    }

    fn set_master_mute(&self, mute: bool) {
        let mut router = self.router.lock().unwrap();
        router.master.mute = mute;
    }

    fn set_master_linked(&self, linked: bool) {
        let mut router = self.router.lock().unwrap();
        router.master.linked = linked;
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

    /// Get current state of all Master FX effects for synchronization
    fn get_master_fx_effects(&self) -> Vec<MasterFxEffect> {
        let router = self.router.lock().unwrap();
        let master = &router.master;
        
        let mut effects = Vec::new();
        
        // Only include effects that are "present" in the FX list
        if master.compressor_present {
            effects.push(MasterFxEffect::Compressor {
                enabled: master.compressor.is_enabled(),
                threshold: master.compressor.get_threshold(),
                ratio: master.compressor.get_ratio(),
                attack: master.compressor.get_attack(),
                release: master.compressor.get_release(),
            });
        }
        
        if master.limiter_present {
            effects.push(MasterFxEffect::Limiter {
                enabled: master.limiter.is_enabled(),
                threshold: master.limiter.get_ceiling(),
                release: master.limiter.get_release(),
            });
        }
        
        if master.delay_present {
            effects.push(MasterFxEffect::Delay {
                enabled: master.delay.is_enabled(),
                time_l: master.delay.get_delay_time_l_ms(),
                time_r: master.delay.get_delay_time_r_ms(),
                feedback: master.delay.get_feedback(),
                mix: master.delay.get_mix(),
            });
        }
        
        if master.reverb_present {
            effects.push(MasterFxEffect::Reverb {
                enabled: master.reverb.is_enabled(),
                room_size: master.reverb.get_room_size(),
                damping: master.reverb.get_damping(),
                wet: master.reverb.get_wet(),
                width: master.reverb.get_width(),
            });
        }
        
        println!("[Engine] get_master_fx_effects returning {} effects", effects.len());
        effects
    }

    /// Add a Master FX effect to the FX list
    fn add_master_fx_effect(&self, effect_type: &str) {
        let mut router = self.router.lock().unwrap();
        let master = &mut router.master;
        
        match effect_type {
            "compressor" => {
                master.compressor_present = true;
                println!("[Engine] Added Master Compressor to FX list");
            }
            "limiter" => {
                master.limiter_present = true;
                println!("[Engine] Added Master Limiter to FX list");
            }
            "delay" => {
                master.delay_present = true;
                println!("[Engine] Added Master Delay to FX list");
            }
            "reverb" => {
                master.reverb_present = true;
                println!("[Engine] Added Master Reverb to FX list");
            }
            _ => {
                eprintln!("[Engine] Unknown effect type: {}", effect_type);
            }
        }
    }

    /// Remove a Master FX effect from the FX list
    fn remove_master_fx_effect(&self, effect_type: &str) {
        let mut router = self.router.lock().unwrap();
        let master = &mut router.master;
        
        match effect_type {
            "compressor" => {
                master.compressor_present = false;
                master.compressor.set_enabled(false); // Also disable it
                println!("[Engine] Removed Master Compressor from FX list");
            }
            "limiter" => {
                master.limiter_present = false;
                master.limiter.set_enabled(false); // Also disable it
                println!("[Engine] Removed Master Limiter from FX list");
            }
            "delay" => {
                master.delay_present = false;
                master.delay.set_enabled(false); // Also disable it
                println!("[Engine] Removed Master Delay from FX list");
            }
            "reverb" => {
                master.reverb_present = false;
                master.reverb.set_enabled(false); // Also disable it
                println!("[Engine] Removed Master Reverb from FX list");
            }
            _ => {
                eprintln!("[Engine] Unknown effect type: {}", effect_type);
            }
        }
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

    fn set_aux_bus_route_to_subgroup(&self, aux: usize, subgroup: usize, route: bool) {
        let mut router = self.router.lock().unwrap();
        if aux < router.aux_buses.len() {
            if route {
                // Add subgroup to routing if not already present
                if !router.aux_buses[aux].route_to_subgroups.contains(&subgroup) {
                    router.aux_buses[aux].route_to_subgroups.push(subgroup);
                }
            } else {
                // Remove subgroup from routing
                router.aux_buses[aux].route_to_subgroups.retain(|&sg| sg != subgroup);
            }
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
            Command::EnableMasterTap { 
                file_path, 
                sample_rate, 
                bit_depth, 
                format 
            } => {
                self.enable_master_tap(
                    file_path, 
                    sample_rate.unwrap_or(48000),
                    bit_depth.unwrap_or(16),
                    format.as_deref().unwrap_or("wav")
                );
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
                    Err(e) => Some(Response::License {
                        key: "DEMO".to_string(),
                        license_type: "demo".to_string(),
                        expires_at: None,
                        is_valid: true,
                    }),
                }
            }
            Command::SaveAudioConfig { sample_rate, buffer_size } => {
                match save_audio_config_to_file(sample_rate, buffer_size) {
                    Ok(_) => Some(Response::Ok {
                        message: "Audio config saved".to_string(),
                    }),
                    Err(e) => Some(Response::Error {
                        message: format!("Failed to save audio config: {}", e),
                    }),
                }
            }
            Command::GetAudioConfig => {
                let config = load_audio_config_from_file();
                Some(Response::AudioConfig {
                    sample_rate: config.sample_rate,
                    buffer_size: config.buffer_size,
                })
            }
            Command::SetTrackSourceInput {
                track,
                left_channel,
                right_channel,
                device_name,
            } => {
                let _ = self.set_track_source_input(track, left_channel, right_channel, device_name.clone());
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: Some(format!("Audio Input ({})", device_name.unwrap_or_else(|| "Default".to_string()))),
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackSourceSignal {
                track,
                waveform,
                frequency,
            } => {
                let _ = self.set_track_source_signal(track, &waveform, frequency);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: Some(format!("Signal Generator ({} @ {} Hz)", waveform, frequency)),
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSignalFrequency { track, frequency } => {
                // Fire-and-forget command, no response needed
                let _ = self.set_signal_frequency(track, frequency);
                None
            }
            Command::SetSignalWaveform { track, waveform } => {
                // Fire-and-forget command, no response needed
                let _ = self.set_signal_waveform(track, &waveform);
                None
            }
            Command::ClearTrackSource { track } => {
                let _ = self.clear_track_source(track);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: Some("".to_string()),
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackSourceFile { track, file_path, artist, title } => {
                match self.set_track_source_file(track, &file_path, artist.as_deref(), title.as_deref()) {
                    Ok(_) => {
                        // Read track parameters after file is loaded
                        let mut router = self.router.lock().unwrap();
                        let (gain, is_stereo) = if let Some(t) = router.get_track_mut(track) {
                            let stereo = if let Some(ref player) = t.file_player {
                                player.channels >= 2
                            } else {
                                false
                            };
                            (Some(t.gain), Some(stereo))
                        } else {
                            (None, None)
                        };
                        drop(router);
                        
                        Some(Response::ParametersChanged {
                            tracks: Some(vec![TrackParameters {
                                track,
                                gain,
                                volume: None,
                                mute: None,
                                pan: None,
                                route_to_master: None,
                                route_to_subgroups: None,
                                pad_enabled: None,
                                hpf_enabled: None,
                                phase_inverted: None,
                                compressor_enabled: None,
                                compressor_threshold_db: None,
                                compressor_ratio: None,
                                compressor_attack_ms: None,
                                compressor_release_ms: None,
                                gate_enabled: None,
                                gate_threshold_db: None,
                                gate_range_db: None,
                                gate_attack_ms: None,
                                gate_release_ms: None,
                                eq_enabled: None,
                                eq_low: None,
                                eq_low_mid: None,
                                eq_high_mid: None,
                                eq_high: None,
                                parametric_eq_enabled: None,
                                eq_filters: None,
                                aux_sends: None,
                                file_name: Some(file_path.clone()),
                                file_artist: artist.clone(),
                                file_title: title.clone(),
                                is_stereo,
                                fft_data: None,
                            }]),
                            subgroups: None,
                            auxes: None,
                            master: None,
                        })
                    }
                    Err(e) => {
                        eprintln!("[Engine] SetTrackSourceFile FAILED for track {}: {}", track, e);
                        None
                    }
                }
            }
            Command::PlayFile { track, file_path, artist, title } => {
                match self.play_file(track, file_path.as_deref(), artist.as_deref(), title.as_deref()) {
                    Ok(_) => {
                        // Read track parameters after file is loaded
                        let mut router = self.router.lock().unwrap();
                        let (gain, is_stereo, file_name, file_artist, file_title) = if let Some(t) = router.get_track_mut(track) {
                            let stereo = if let Some(ref player) = t.file_player {
                                player.channels >= 2
                            } else {
                                false
                            };
                            
                            // If file_path is provided, use it and the provided metadata
                            // Otherwise, read metadata from existing file_player
                            let (fname, fartist, ftitle) = if file_path.is_some() {
                                // Extract filename from provided path
                                let fname = if let Some(ref path) = file_path {
                                    std::path::Path::new(path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| path.clone())
                                } else {
                                    String::new()
                                };
                                
                                // If artist/title not provided but file_player exists, preserve existing metadata
                                let (final_artist, final_title) = if artist.is_none() && title.is_none() {
                                    if let Some(ref player) = t.file_player {
                                        (player.file_artist.clone(), player.file_title.clone())
                                    } else {
                                        (None, None)
                                    }
                                } else {
                                    (artist, title)
                                };
                                
                                (Some(fname), final_artist, final_title)
                            } else {
                                // Read from existing file_player
                                if let Some(ref player) = t.file_player {
                                    (
                                        Some(player.file_name.clone()),
                                        player.file_artist.clone(),
                                        player.file_title.clone()
                                    )
                                } else {
                                    (Some(String::new()), None, None)
                                }
                            };
                            
                            (Some(t.gain), Some(stereo), fname, fartist, ftitle)
                        } else {
                            (None, None, Some(String::new()), None, None)
                        };
                        drop(router);
                        
                        Some(Response::ParametersChanged {
                            tracks: Some(vec![TrackParameters {
                                track,
                                gain,
                                volume: None,
                                mute: None,
                                pan: None,
                                route_to_master: None,
                                route_to_subgroups: None,
                                pad_enabled: None,
                                hpf_enabled: None,
                                phase_inverted: None,
                                compressor_enabled: None,
                                compressor_threshold_db: None,
                                compressor_ratio: None,
                                compressor_attack_ms: None,
                                compressor_release_ms: None,
                                gate_enabled: None,
                                gate_threshold_db: None,
                                gate_range_db: None,
                                gate_attack_ms: None,
                                gate_release_ms: None,
                                eq_enabled: None,
                                eq_low: None,
                                eq_low_mid: None,
                                eq_high_mid: None,
                                eq_high: None,
                                parametric_eq_enabled: None,
                                eq_filters: None,
                                aux_sends: None,
                                file_name,
                                file_artist,
                                file_title,
                                is_stereo,
                                fft_data: None,
                            }]),
                            subgroups: None,
                            auxes: None,
                            master: None,
                        })
                    }
                    Err(e) => {
                        eprintln!("[Engine] PlayFile FAILED for track {}: {}", track, e);
                        None
                    }
                }
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
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: Some(gain),
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetVolume { track, volume } => {
                self.set_volume(track, volume);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: Some(volume),
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetMute { track, mute } => {
                self.set_mute(track, mute);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: Some(mute),
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetRouteToMaster { track, route } => {
                self.set_route_to_master(track, route);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: Some(route),
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetPan { track, pan } => {
                self.set_pan(track, pan);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: Some(pan),
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackPad { track, enabled } => {
                self.set_pad(track, enabled);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: Some(enabled),
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackHPF { track, enabled } => {
                self.set_hpf(track, enabled);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: Some(enabled),
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackPhaseInvert { track, enabled } => {
                self.set_phase_invert(track, enabled);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: Some(enabled),
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
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
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: Some(enabled),
                        compressor_threshold_db: Some(threshold),
                        compressor_ratio: Some(ratio),
                        compressor_attack_ms: Some(attack),
                        compressor_release_ms: Some(release),
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
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
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: Some(enabled),
                        gate_threshold_db: Some(threshold),
                        gate_range_db: Some(range),
                        gate_attack_ms: Some(attack),
                        gate_release_ms: Some(release),
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetEQ {
                track,
                low,
                low_mid,
                high_mid,
                high,
            } => {
                self.set_eq(track, low, low_mid, high_mid, high);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: Some(low),
                        eq_low_mid: Some(low_mid),
                        eq_high_mid: Some(high_mid),
                        eq_high: Some(high),
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetEQEnabled { track, enabled } => {
                self.set_eq_enabled(track, enabled);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: Some(enabled),
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetParametricEQFilters { track, filters } => {
                self.set_parametric_eq_filters(track, &filters);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: Some(filters),
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetParametricEQEnabled { track, enabled } => {
                self.set_parametric_eq_enabled(track, enabled);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: Some(enabled),
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::ClearParametricEQ { track } => {
                self.clear_parametric_eq(track);
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: Some(vec![]),
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetMasterGain { gain } => {
                self.set_master_gain(gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: Some(gain),
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterGainLeft { gain } => {
                self.set_master_gain_left(gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: Some(gain),
                        gain_right: None,
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterGainRight { gain } => {
                self.set_master_gain_right(gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: Some(gain),
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterMute { mute } => {
                self.set_master_mute(mute);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: Some(mute),
                        linked: None,
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterLinked { linked } => {
                self.set_master_linked(linked);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: Some(linked),
                        selected_output: None,
                        eq_filters: None,
                        fx_effects: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterParametricEQFilters { filters } => {
                self.set_master_parametric_eq_filters(&filters);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: Some(filters),
                        fx_effects: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterParametricEQEnabled { enabled } => {
                self.set_master_parametric_eq_enabled(enabled);
                None
            }
            Command::ClearMasterParametricEQ => {
                self.clear_master_parametric_eq();
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        selected_output: None,
                        eq_filters: Some(vec![]),
                        fx_effects: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterOutputChannels {
                left_channel,
                right_channel,
            } => {
                self.set_master_output_channels(left_channel, right_channel);
                None
            }
            Command::SetSelectedMasterOutput { device_id } => {
                *self.selected_master_output.lock().unwrap() = device_id.clone();
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        eq_filters: None,
                        fx_effects: None,
                        selected_output: Some(device_id),
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterCompressor {
                enabled,
                threshold,
                ratio,
                attack,
                release,
            } => {
                println!("[Engine] SetMasterCompressor: enabled={}", enabled);
                self.set_master_compressor(enabled, threshold, ratio, attack, release);
                let fx_effects = self.get_master_fx_effects();
                println!("[Engine] Returning fx_effects: {:?}", fx_effects);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        eq_filters: None,
                        fx_effects: Some(fx_effects),
                        selected_output: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterLimiter {
                enabled,
                ceiling,
                release,
            } => {
                self.set_master_limiter(enabled, ceiling, release);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        eq_filters: None,
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterDelay {
                enabled,
                time_l,
                time_r,
                feedback,
                mix,
            } => {
                self.set_master_delay(enabled, time_l, time_r, feedback, mix);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        eq_filters: None,
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::SetMasterReverb {
                enabled,
                room_size,
                damping,
                wet,
                width,
            } => {
                self.set_master_reverb(enabled, room_size, damping, wet, width);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        eq_filters: None,
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::AddMasterFxEffect { effect_type } => {
                self.add_master_fx_effect(&effect_type);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        eq_filters: None,
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                    }),
                })
            }
            Command::RemoveMasterFxEffect { effect_type } => {
                self.remove_master_fx_effect(&effect_type);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: None,
                    master: Some(MasterParameters {
                        gain: None,
                        gain_left: None,
                        gain_right: None,
                        mute: None,
                        linked: None,
                        eq_filters: None,
                        fx_effects: Some(self.get_master_fx_effects()),
                        selected_output: None,
                        available_output_devices: None,
                    }),
                })
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
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: Some(gain),
                        mute: None,
                        route_to_master: None,
                        selected_output: None,
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSubgroupMute { subgroup, mute } => {
                self.set_subgroup_mute(subgroup, mute);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: None,
                        mute: Some(mute),
                        route_to_master: None,
                        selected_output: None,
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSubgroupOutputEnabled { subgroup, enabled } => {
                self.set_subgroup_output_enabled(subgroup, enabled);
                None
            }
            Command::SetSubgroupRouteToMaster { subgroup, route } => {
                self.set_subgroup_route_to_master(subgroup, route);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: None,
                        mute: None,
                        route_to_master: Some(route),
                        selected_output: None,
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            Command::SetSubgroupOutputChannels {
                subgroup,
                left_channel,
                right_channel,
            } => {
                self.set_subgroup_output_channels(subgroup, left_channel, right_channel);
                None
            }
            Command::SetSelectedSubgroupOutput { subgroup, device_id } => {
                if let Ok(mut router) = self.router.try_lock() {
                    if let Some(sg) = router.subgroups.iter_mut().find(|s| s.id == subgroup) {
                        sg.selected_output = device_id.clone();
                    }
                }
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: Some(vec![SubgroupParameters {
                        subgroup,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        selected_output: Some(device_id),
                    }]),
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackRouteToSubgroup {
                track,
                subgroup,
                route,
            } => {
                self.set_track_route_to_subgroup(track, subgroup, route);
                
                // Get updated routing list
                let route_to_subgroups = if let Ok(router) = self.router.try_lock() {
                    if track < router.tracks.len() {
                        router.tracks[track].route_to_subgroups.clone()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: Some(route_to_subgroups),
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: None,
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetTrackAuxSend {
                track,
                aux,
                level,
                pre_fader,
                muted,
            } => {
                self.set_track_aux_send(track, aux, level, pre_fader, muted);
                
                // Get updated aux sends for this track and convert to AuxSendData
                let aux_sends = if let Ok(router) = self.router.try_lock() {
                    if track < router.tracks.len() {
                        router.tracks[track].aux_sends.iter().map(|send| AuxSendData {
                            level: send.level,
                            pre_fader: send.pre_fader,
                            muted: send.muted,
                        }).collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                
                Some(Response::ParametersChanged {
                    tracks: Some(vec![TrackParameters {
                        track,
                        gain: None,
                        volume: None,
                        mute: None,
                        pan: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        pad_enabled: None,
                        hpf_enabled: None,
                        phase_inverted: None,
                        compressor_enabled: None,
                        compressor_threshold_db: None,
                        compressor_ratio: None,
                        compressor_attack_ms: None,
                        compressor_release_ms: None,
                        gate_enabled: None,
                        gate_threshold_db: None,
                        gate_range_db: None,
                        gate_attack_ms: None,
                        gate_release_ms: None,
                        eq_enabled: None,
                        eq_low: None,
                        eq_low_mid: None,
                        eq_high_mid: None,
                        eq_high: None,
                        parametric_eq_enabled: None,
                        eq_filters: None,
                        aux_sends: Some(aux_sends),
                        file_name: None,
                        file_artist: None,
                        file_title: None,
                        is_stereo: None,
                        fft_data: None,
                    }]),
                    subgroups: None,
                    auxes: None,
                    master: None,
                })
            }
            Command::SetAuxBusGain { aux, gain } => {
                self.set_aux_bus_gain(aux, gain);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: Some(gain),
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusMute { aux, mute } => {
                self.set_aux_bus_mute(aux, mute);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: Some(mute),
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
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
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: Some(AuxReverbParams {
                            enabled,
                            room_size,
                            damping,
                            wet,
                            width,
                        }),
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusDelay {
                aux,
                enabled,
                time,
                feedback,
                mix,
            } => {
                self.set_aux_bus_delay(aux, enabled, time, feedback, mix);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: Some(AuxDelayParams {
                            enabled,
                            delay_time_l_ms: time,
                            delay_time_r_ms: time,
                            feedback,
                            mix,
                        }),
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusRouteToMaster { aux, route } => {
                self.set_aux_bus_route_to_master(aux, route);
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: Some(route),
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
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
            Command::SetAuxBusRouteToSubgroup { aux, subgroup, route } => {
                self.set_aux_bus_route_to_subgroup(aux, subgroup, route);
                
                // Get updated routing list
                let route_to_subgroups = if let Ok(router) = self.router.try_lock() {
                    if let Some(aux_bus) = router.aux_buses.iter().find(|a| a.id == aux) {
                        aux_bus.route_to_subgroups.clone()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: Some(route_to_subgroups),
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: None,
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetAuxBusSelectedOutput { aux, device_id } => {
                if let Ok(mut router) = self.router.try_lock() {
                    if let Some(aux_bus) = router.aux_buses.iter_mut().find(|a| a.id == aux) {
                        aux_bus.selected_output = device_id.clone();
                    }
                }
                Some(Response::ParametersChanged {
                    tracks: None,
                    subgroups: None,
                    auxes: Some(vec![AuxParameters {
                        aux,
                        gain: None,
                        mute: None,
                        route_to_master: None,
                        route_to_subgroups: None,
                        output_enabled: None,
                        output_channel_selection_left: None,
                        output_channel_selection_right: None,
                        selected_output: Some(device_id),
                        reverb: None,
                        delay: None,
                    }]),
                    master: None,
                })
            }
            Command::SetTrackSourceAuxReturn { track, aux } => {
                match self.set_track_source_aux_return(track, aux) {
                    Ok(_) => None,
                    Err(e) => Some(Response::Error {
                        message: e.to_string(),
                    }),
                }
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
                                            is_default: false, // Individual channels are not marked as default
                                        }
                                    }).collect::<Vec<_>>()
                                } else {
                                    // Single or stereo channel device
                                    vec![device]
                                }
                            })
                            .collect();
                        
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
            
            // NDI Streaming commands
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
            },
            Command::StopNdi => {
                match self.ndi_stream.stop() {
                    Ok(_) => Some(Response::Ok {
                        message: "NDI stream stopped".to_string(),
                    }),
                    Err(e) => Some(Response::Error {
                        message: format!("Failed to stop NDI: {}", e),
                    }),
                }
            },
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
            },
            Command::SetNdiName { name } => {
                match self.ndi_stream.set_stream_name(name) {
                    Ok(_) => None, // Silent success
                    Err(e) => Some(Response::Error {
                        message: format!("Failed to set NDI name: {}", e),
                    }),
                }
            },
            Command::SetNdiVideoText { text } => {
                self.ndi_stream.set_video_text(text);
                None // Silent success
            },
            
            Command::GetLoudness => {
                let router = self.router.lock().unwrap();
                let loudness_data = router.loudness_meter.get_measurements();
                Some(Response::LoudnessData {
                    momentary_lufs: loudness_data.momentary,
                    short_term_lufs: loudness_data.short_term,
                    integrated_lufs: loudness_data.integrated,
                    loudness_range_lu: loudness_data.loudness_range,
                    true_peak_dbtp: loudness_data.true_peak_dbtp,
                })
            },
            
            Command::ResetLoudness => {
                let mut router = self.router.lock().unwrap();
                router.loudness_meter.reset();
                Some(Response::Ok {
                    message: "Loudness measurements reset".to_string(),
                })
            },
            
            Command::GetDynamicRange => {
                let router = self.router.lock().unwrap();
                let dr_data = router.dynamic_range_meter.get_measurements();
                Some(Response::DynamicRangeData {
                    peak_db_l: dr_data.peak_db_l,
                    peak_db_r: dr_data.peak_db_r,
                    rms_db_l: dr_data.rms_db_l,
                    rms_db_r: dr_data.rms_db_r,
                    dynamic_range_l: dr_data.dynamic_range_l,
                    dynamic_range_r: dr_data.dynamic_range_r,
                    dynamic_range_stereo: dr_data.dynamic_range_stereo,
                })
            },
            
            Command::ResetDynamicRange => {
                let mut router = self.router.lock().unwrap();
                router.dynamic_range_meter.reset();
                Some(Response::Ok {
                    message: "Dynamic range measurements reset".to_string(),
                })
            },
            
            Command::GetPhaseCorrelation => {
                let router = self.router.lock().unwrap();
                let phase_data = router.phase_correlation_meter.get_measurement();
                Some(Response::PhaseCorrelationData {
                    correlation: phase_data.correlation,
                    mono_compatible: phase_data.mono_compatible,
                })
            },
            
            Command::ResetPhaseCorrelation => {
                let mut router = self.router.lock().unwrap();
                router.phase_correlation_meter.reset();
                Some(Response::Ok {
                    message: "Phase correlation measurements reset".to_string(),
                })
            },
            
            Command::GetStereoWidth => {
                let router = self.router.lock().unwrap();
                let width_data = router.stereo_width_meter.get_measurement();
                Some(Response::StereoWidthData {
                    width_percent: width_data.width_percent,
                    mid_rms: width_data.mid_rms,
                    side_rms: width_data.side_rms,
                    balance: width_data.balance,
                })
            },
            
            Command::ResetStereoWidth => {
                let mut router = self.router.lock().unwrap();
                router.stereo_width_meter.reset();
                Some(Response::Ok {
                    message: "Stereo width measurements reset".to_string(),
                })
            },
            
            Command::GetHeadroom => {
                let router = self.router.lock().unwrap();
                let headroom_data = router.headroom_meter.get_measurement();
                Some(Response::HeadroomData {
                    peak_l: headroom_data.peak_l,
                    peak_r: headroom_data.peak_r,
                    headroom_l: headroom_data.headroom_l,
                    headroom_r: headroom_data.headroom_r,
                    headroom_stereo: headroom_data.headroom_stereo,
                })
            },
            
            Command::ResetHeadroom => {
                let mut router = self.router.lock().unwrap();
                router.headroom_meter.reset();
                Some(Response::Ok {
                    message: "Headroom measurements reset".to_string(),
                })
            },
        }
    }
}

// License management structures and functions
#[derive(Debug, Serialize, Deserialize, Clone)]
struct LicenseData {
    key: String,
    license_type: String,
    expires_at: Option<String>,
    is_valid: bool,
}

// Audio configuration structures and functions
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AudioConfigData {
    sample_rate: u32,  // 0 = Auto, otherwise specific rate (44100, 48000, 96000, 192000)
    buffer_size: u32,  // Buffer size in frames (64, 128, 256, 512, 1024)
}

impl Default for AudioConfigData {
    fn default() -> Self {
        Self {
            sample_rate: 44100,  // 44.1 kHz by default (most common for audio production)
            buffer_size: 256,    // 256 frames default
        }
    }
}

fn get_audio_config_file_path() -> PathBuf {
    // Use same base directory as license
    let base_path = std::env::var("LICENSE_PATH")
        .unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(base_path).join("audio_config.json");
    path
}

fn save_audio_config_to_file(sample_rate: u32, buffer_size: u32) -> Result<()> {
    let config = AudioConfigData {
        sample_rate,
        buffer_size,
    };
    
    let path = get_audio_config_file_path();
    let json = serde_json::to_string_pretty(&config)?;
    
    match std::fs::write(&path, &json) {
        Ok(_) => {
            let rate_str = if sample_rate == 0 {
                "Auto".to_string()
            } else {
                format!("{}", sample_rate)
            };
            eprintln!("[Engine] ✓ Audio config saved: {} Hz ({}=Auto), {} frames", 
                rate_str,
                if sample_rate == 0 { "0" } else { "" },
                buffer_size
            );
            Ok(())
        },
        Err(e) => {
            eprintln!("[Engine] ✗ Failed to write audio config file: {}", e);
            Err(e.into())
        }
    }
}

fn load_audio_config_from_file() -> AudioConfigData {
    let path = get_audio_config_file_path();
    
    if !path.exists() {
        eprintln!("[Engine] No audio config file found, using defaults (Auto, 256 frames)");
        return AudioConfigData::default();
    }
    
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            match serde_json::from_str::<AudioConfigData>(&json) {
                Ok(config) => {
                    eprintln!("[Engine] ✓ Audio config loaded: {} Hz, {} frames", 
                        if config.sample_rate == 0 { "Auto".to_string() } else { config.sample_rate.to_string() },
                        config.buffer_size
                    );
                    config
                },
                Err(e) => {
                    eprintln!("[Engine] ✗ Failed to parse audio config: {}, using defaults", e);
                    AudioConfigData::default()
                }
            }
        },
        Err(e) => {
            eprintln!("[Engine] ✗ Failed to read audio config file: {}, using defaults", e);
            AudioConfigData::default()
        }
    }
}

fn get_license_file_path() -> PathBuf {
    // Use environment variable or default to current directory
    let base_path = std::env::var("LICENSE_PATH")
        .unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(base_path).join("license.json");
    path
}

fn save_license_to_file(key: &str, license_type: &str, expires_at: Option<&str>) -> Result<()> {
    let license = LicenseData {
        key: key.to_string(),
        license_type: license_type.to_string(),
        expires_at: expires_at.map(|s| s.to_string()),
        is_valid: true,
    };
    
    let path = get_license_file_path();
    
    let json = serde_json::to_string_pretty(&license)?;
    
    match std::fs::write(&path, &json) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            eprintln!("[Engine] ✗ Failed to write license file: {}", e);
            Err(e.into())
        }
    }
}

fn load_license_from_file() -> Result<LicenseData> {
    let path = get_license_file_path();
    
    if !path.exists() {
        eprintln!("[Engine] No license file found at {:?}, using demo mode", path);
        return Ok(LicenseData {
            key: "DEMO".to_string(),
            license_type: "demo".to_string(),
            expires_at: None,
            is_valid: true,
        });
    }
    
    let json = std::fs::read_to_string(&path)?;
    let license: LicenseData = serde_json::from_str(&json)?;
    Ok(license)
}

fn send_response(response: &Response) {
    if let Ok(json) = serde_json::to_string(response) {
        println!("{}", json);
    }
}

fn main() -> Result<()> {
    eprintln!("[Engine] mMpro3 Audio Engine starting...");

    let mut engine = AudioEngine::new();
    
    // Auto-select default output device on startup
    match engine.list_devices() {
        Ok(devices) => {
            // Find default output device
            if let Some(default_device) = devices.iter().find(|d| d.is_default && d.output_channels > 0) {
                let default_selection = format!("{}:0:1", default_device.id);
                *engine.selected_master_output.lock().unwrap() = Some(default_selection.clone());
                eprintln!("[Engine] Auto-selected default output: {} ({})", default_device.name, default_selection);
            }
            // Store available output devices for serialization
            engine.available_output_devices = devices.into_iter()
                .filter(|d| d.output_channels > 0)
                .collect();
        }
        Err(e) => {
            eprintln!("[Engine] Warning: Could not enumerate devices for auto-selection: {}", e);
        }
    }
    
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

