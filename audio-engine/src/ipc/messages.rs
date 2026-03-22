// IPC Messages - Commands and Responses for communication with Electron frontend

use serde::{Deserialize, Serialize};
use crate::io::DeviceInfo;

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
pub enum Command {
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
        waveform: String,
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
        waveform: String,
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
        playlist_id: Option<String>,
        playlist_name: Option<String>,
        playlist_index: Option<usize>,
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
    #[serde(rename = "seek_file")]
    SeekFile { track: usize, time_seconds: f32 },
    #[serde(rename = "get_waveform_data")]
    GetWaveformData { track: usize, num_points: usize },
    #[serde(rename = "stop_all_files")]
    StopAllFiles,

    // Track controls
    #[serde(rename = "set_gain")]
    SetGain { track: usize, gain: f32 },
    #[serde(rename = "set_volume")]
    SetVolume { track: usize, volume: f32 },
    #[serde(rename = "set_mute")]
    SetMute { track: usize, mute: bool },
    #[serde(rename = "set_solo")]
    SetSolo { track: usize, solo: bool },
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
    #[serde(rename = "set_track_pfl")]
    SetTrackPFL { track: usize, enabled: bool },
    
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
        range: f32,
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
        pre_delay: f32,
    },
    #[serde(rename = "add_master_fx_effect")]
    AddMasterFxEffect {
        effect_type: String,
    },
    #[serde(rename = "remove_master_fx_effect")]
    RemoveMasterFxEffect {
        effect_type: String,
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
        route: bool,
    },

    // Master tap (for recording)
    #[serde(rename = "enable_master_tap")]
    EnableMasterTap {
        file_path: String,
        sample_rate: Option<u32>,
        bit_depth: Option<u32>,
        format: Option<String>,
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
        sample_rate: u32,
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
        pre_delay: f32,
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

    // Input stream management
    #[serde(rename = "open_audio_input")]
    OpenAudioInput,
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
        source: String,
    },
    #[serde(rename = "stop_ndi")]
    StopNdi,
    #[serde(rename = "set_ndi_source")]
    SetNdiSource { source: String },
    #[serde(rename = "set_ndi_name")]
    SetNdiName { name: String },
    #[serde(rename = "set_ndi_video_text")]
    SetNdiVideoText { text: String },
    
    // Loudness Metering
    #[serde(rename = "get_loudness")]
    GetLoudness,
    #[serde(rename = "reset_loudness")]
    ResetLoudness,
    
    // Dynamic Range Metering
    #[serde(rename = "get_dynamic_range")]
    GetDynamicRange,
    #[serde(rename = "reset_dynamic_range")]
    ResetDynamicRange,
    
    // Phase Correlation Metering
    #[serde(rename = "get_phase_correlation")]
    GetPhaseCorrelation,
    #[serde(rename = "reset_phase_correlation")]
    ResetPhaseCorrelation,
    
    // Stereo Width Metering
    #[serde(rename = "get_stereo_width")]
    GetStereoWidth,
    #[serde(rename = "reset_stereo_width")]
    ResetStereoWidth,
    
    // Headroom Metering
    #[serde(rename = "get_headroom")]
    GetHeadroom,
    #[serde(rename = "reset_headroom")]
    ResetHeadroom,
    
    // Track Insert Effects Chain
    #[serde(rename = "add_track_insert")]
    AddTrackInsert {
        track: usize,
        effect_type: String,
        position: Option<usize>,
    },
    #[serde(rename = "remove_track_insert")]
    RemoveTrackInsert {
        track: usize,
        insert_id: usize,
    },
    #[serde(rename = "move_track_insert")]
    MoveTrackInsert {
        track: usize,
        insert_id: usize,
        new_position: usize,
    },
    #[serde(rename = "set_track_insert_enabled")]
    SetTrackInsertEnabled {
        track: usize,
        insert_id: usize,
        enabled: bool,
    },
    #[serde(rename = "set_track_insert_compressor")]
    SetTrackInsertCompressor {
        track: usize,
        insert_id: usize,
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
    },
    #[serde(rename = "set_track_insert_gate")]
    SetTrackInsertGate {
        track: usize,
        insert_id: usize,
        threshold: f32,
        range: f32,
        attack: f32,
        release: f32,
    },
    #[serde(rename = "set_track_insert_reverb")]
    SetTrackInsertReverb {
        track: usize,
        insert_id: usize,
        room_size: f32,
        damping: f32,
        wet: f32,
        width: f32,
        pre_delay: f32,
    },
    #[serde(rename = "set_track_insert_delay")]
    SetTrackInsertDelay {
        track: usize,
        insert_id: usize,
        time_l: f32,
        time_r: f32,
        feedback: f32,
        mix: f32,
    },
    #[serde(rename = "set_track_insert_exciter")]
    SetTrackInsertExciter {
        track: usize,
        insert_id: usize,
        amount: f32,
        frequency: f32,
        mix: f32,
    },
    #[serde(rename = "set_track_insert_deesser")]
    SetTrackInsertDeEsser {
        track: usize,
        insert_id: usize,
        threshold: f32,
        frequency: f32,
        range: f32,
    },
    #[serde(rename = "set_track_insert_chorus")]
    SetTrackInsertChorus {
        track: usize,
        insert_id: usize,
        rate: f32,
        depth: f32,
        mix: f32,
    },
    
    // EQ Presets
    #[serde(rename = "get_eq_presets")]
    GetEQPresets,
    #[serde(rename = "apply_eq_preset")]
    ApplyEQPreset {
        preset_name: String,
    },
}

/// Risposta inviata a Electron via stdout
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Response {
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
        sample_rate: u32,
        buffer_size: u32,
    },
    #[serde(rename = "waveform_data")]
    WaveformData {
        track: usize,
        data: Vec<f32>,
        duration: f32,
        sample_rate: u32,
    },
    #[serde(rename = "eq_presets")]
    EQPresets {
        presets: Vec<EQPresetData>,
    },
}

// === Supporting Structs ===

#[derive(Debug, Serialize)]
pub struct EQPresetData {
    pub name: String,
    pub filters: Vec<EQPresetFilter>,
}

#[derive(Debug, Serialize)]
pub struct EQPresetFilter {
    pub filter_type: String,
    pub frequency: f32,
    pub gain: f32,
    pub q: f32,
}

#[derive(Debug, Serialize)]
pub struct TrackMeters {
    pub track: usize,
    pub level_l: f32,
    pub level_r: f32,
    pub level_pre_fader_l: f32,
    pub level_pre_fader_r: f32,
    pub waveform: Vec<f32>,
    pub phase_correlation: f32,
    pub compressor_input_db: f32,
    pub compressor_reduction_db: f32,
    pub gate_input_db: f32,
    pub gate_attenuation_db: f32,
    pub file_ended: bool,
    pub is_playing: bool,
    pub bpm: f32,
    pub gain: f32,
    pub volume: f32,
    pub mute: bool,
    pub pan: f32,
    pub is_stereo: bool,
    pub file_name: String,
    pub file_artist: Option<String>,
    pub file_title: Option<String>,
    pub playlist_id: Option<String>,
    pub playlist_name: Option<String>,
    pub playlist_current_index: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SubgroupMeters {
    pub subgroup: usize,
    pub level_l: f32,
    pub level_r: f32,
}

#[derive(Debug, Serialize)]
pub struct AuxMeters {
    pub aux: usize,
    pub level_l: f32,
    pub level_r: f32,
}

#[derive(Debug, Serialize)]
pub struct HeadroomDataStruct {
    pub peak_l: f32,
    pub peak_r: f32,
    pub headroom_l: f32,
    pub headroom_r: f32,
    pub headroom_stereo: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrackParameters {
    pub track: usize,
    pub gain: Option<f32>,
    pub volume: Option<f32>,
    pub mute: Option<bool>,
    pub solo: Option<bool>,
    pub pan: Option<f32>,
    pub route_to_master: Option<bool>,
    pub route_to_subgroups: Option<Vec<usize>>,
    pub pad_enabled: Option<bool>,
    pub hpf_enabled: Option<bool>,
    pub phase_inverted: Option<bool>,
    pub pfl_enabled: Option<bool>,
    pub compressor_enabled: Option<bool>,
    pub compressor_threshold_db: Option<f32>,
    pub compressor_ratio: Option<f32>,
    pub compressor_attack_ms: Option<f32>,
    pub compressor_release_ms: Option<f32>,
    pub gate_enabled: Option<bool>,
    pub gate_threshold_db: Option<f32>,
    pub gate_range_db: Option<f32>,
    pub gate_attack_ms: Option<f32>,
    pub gate_release_ms: Option<f32>,
    pub eq_enabled: Option<bool>,
    pub eq_low: Option<f32>,
    pub eq_low_mid: Option<f32>,
    pub eq_high_mid: Option<f32>,
    pub eq_high: Option<f32>,
    pub parametric_eq_enabled: Option<bool>,
    pub eq_filters: Option<Vec<ParametricFilter>>,
    pub aux_sends: Option<Vec<AuxSendData>>,
    pub file_name: Option<String>,
    pub file_artist: Option<String>,
    pub file_title: Option<String>,
    pub is_stereo: Option<bool>,
    pub fft_data: Option<FFTDataSimple>,
    pub inserts: Option<Vec<InsertEffectInfo>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct InsertEffectInfo {
    pub id: usize,
    pub effect_type: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct FFTDataSimple {
    pub bins_left: Vec<f32>,
    pub bins_right: Vec<f32>,
    pub sample_rate: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct SubgroupParameters {
    pub subgroup: usize,
    pub gain: Option<f32>,
    pub mute: Option<bool>,
    pub route_to_master: Option<bool>,
    pub selected_output: Option<Option<String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuxParameters {
    pub aux: usize,
    pub gain: Option<f32>,
    pub mute: Option<bool>,
    pub route_to_master: Option<bool>,
    pub route_to_subgroups: Option<Vec<usize>>,
    pub output_enabled: Option<bool>,
    pub output_channel_selection_left: Option<u16>,
    pub output_channel_selection_right: Option<u16>,
    pub selected_output: Option<Option<String>>,
    pub reverb: Option<AuxReverbParams>,
    pub delay: Option<AuxDelayParams>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum MasterFxEffect {
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
pub struct MasterParameters {
    pub gain: Option<f32>,
    pub gain_left: Option<f32>,
    pub gain_right: Option<f32>,
    pub mute: Option<bool>,
    pub linked: Option<bool>,
    pub eq_filters: Option<Vec<ParametricFilter>>,
    pub current_eq_preset: Option<Option<String>>,
    pub fx_effects: Option<Vec<MasterFxEffect>>,
    pub selected_output: Option<Option<String>>,
    pub available_output_devices: Option<Vec<DeviceInfo>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuxSendData {
    pub level: f32,
    pub pre_fader: bool,
    pub muted: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuxReverbParams {
    pub enabled: bool,
    pub room_size: f32,
    pub damping: f32,
    pub wet: f32,
    pub width: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuxDelayParams {
    pub enabled: bool,
    pub delay_time_l_ms: f32,
    pub delay_time_r_ms: f32,
    pub feedback: f32,
    pub mix: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct LoudnessDataStruct {
    pub momentary_lufs: f32,
    pub short_term_lufs: f32,
    pub integrated_lufs: f32,
    pub loudness_range_lu: f32,
    pub true_peak_dbtp: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct DynamicRangeDataStruct {
    pub peak_db_l: f32,
    pub peak_db_r: f32,
    pub rms_db_l: f32,
    pub rms_db_r: f32,
    pub dynamic_range_l: f32,
    pub dynamic_range_r: f32,
    pub dynamic_range_stereo: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct PhaseCorrelationDataStruct {
    pub correlation: f32,
    pub mono_compatible: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct StereoWidthDataStruct {
    pub width_percent: f32,
    pub mid_rms: f32,
    pub side_rms: f32,
    pub balance: f32,
}
