use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete engine state snapshot for scene management
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineSnapshot {
    pub version: u32, // For future compatibility
    pub timestamp: u64,
    pub name: String,
    #[serde(default)]
    pub pinned: bool,
    pub tracks: Vec<TrackSnapshot>,
    pub master: MasterSnapshot,
    pub subgroups: Vec<SubgroupSnapshot>,
    pub auxes: Vec<AuxSnapshot>,
}

/// Track state snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackSnapshot {
    pub track_number: usize,
    
    // Source
    pub source_type: String, // "none", "signal", "file", "input"
    pub signal_waveform: Option<String>,
    pub signal_frequency: Option<f32>,
    pub file_path: Option<String>,
    pub file_artist: Option<String>,
    pub file_title: Option<String>,
    pub input_device: Option<String>,
    pub input_left_channel: Option<u16>,
    pub input_right_channel: Option<u16>,
    
    // Basic controls
    pub gain: f32,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub pad_enabled: bool,
    pub hpf_enabled: bool,
    pub phase_inverted: bool,
    pub pfl_enabled: bool,
    
    // Routing
    pub route_to_master: bool,
    pub routed_subgroups: Vec<usize>,
    
    // 3-band EQ
    pub eq_enabled: bool,
    pub eq_low: f32,
    pub eq_low_mid: f32,
    pub eq_high_mid: f32,
    pub eq_high: f32,
    
    // Parametric EQ
    pub parametric_eq_enabled: bool,
    pub parametric_eq_filters: Vec<ParametricFilterSnapshot>,
    
    // Dynamics
    pub compressor: Option<CompressorSnapshot>,
    pub gate: Option<GateSnapshot>,
    
    // Insert FX chain
    pub insert_effects: Vec<InsertEffectSnapshot>,
    
    // Aux sends
    pub aux_sends: HashMap<usize, f32>, // aux_id -> send_level
}

/// Parametric EQ filter snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParametricFilterSnapshot {
    pub filter_type: String,
    pub frequency: f32,
    pub gain: f32,
    pub q: f32,
}

/// Compressor snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompressorSnapshot {
    pub enabled: bool,
    pub threshold: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
}

/// Gate snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GateSnapshot {
    pub enabled: bool,
    pub threshold: f32,
    pub range: f32,
    pub attack: f32,
    pub release: f32,
}

/// Insert effect snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsertEffectSnapshot {
    pub id: usize,
    pub effect_type: String,
    pub enabled: bool,
    pub position: usize,
    pub parameters: InsertEffectParameters,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum InsertEffectParameters {
    #[serde(rename = "compressor")]
    Compressor {
        threshold: f32,
        ratio: f32,
        attack: f32,
        release: f32,
    },
    #[serde(rename = "gate")]
    Gate {
        threshold: f32,
        range: f32,
        attack: f32,
        release: f32,
    },
    #[serde(rename = "reverb")]
    Reverb {
        room_size: f32,
        damping: f32,
        wet: f32,
        width: f32,
        pre_delay: f32,
    },
    #[serde(rename = "delay")]
    Delay {
        time_l: f32,
        time_r: f32,
        feedback: f32,
        mix: f32,
    },
    #[serde(rename = "exciter")]
    Exciter {
        amount: f32,
        frequency: f32,
        mix: f32,
    },
    #[serde(rename = "deesser")]
    DeEsser {
        threshold: f32,
        frequency: f32,
        range: f32,
    },
    #[serde(rename = "chorus")]
    Chorus {
        rate: f32,
        depth: f32,
        mix: f32,
    },
}

/// Master bus snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MasterSnapshot {
    pub gain_left: f32,
    pub gain_right: f32,
    pub linked: bool,
    pub muted: bool,
    pub output_left_channel: u16,
    pub output_right_channel: u16,
    pub selected_output_device: Option<String>,
    
    // Master Parametric EQ
    pub parametric_eq_enabled: bool,
    pub parametric_eq_filters: Vec<ParametricFilterSnapshot>,
    
    // Master FX Chain
    pub compressor: Option<CompressorSnapshot>,
    pub limiter: Option<LimiterSnapshot>,
    pub exciter: Option<ExciterSnapshot>,
    pub stereo_width: Option<StereoWidthSnapshot>,
}

/// Limiter snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimiterSnapshot {
    pub enabled: bool,
    pub threshold: f32,
    pub release: f32,
}

/// Exciter snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExciterSnapshot {
    pub enabled: bool,
    pub amount: f32,
    pub frequency: f32,
    pub mix: f32,
}

/// Stereo width snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StereoWidthSnapshot {
    pub enabled: bool,
    pub width: f32,
}

/// Subgroup snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubgroupSnapshot {
    pub id: usize,
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub route_to_master: bool,
    pub output_left_channel: u16,
    pub output_right_channel: u16,
    pub selected_output_device: Option<String>,
}

/// Aux bus snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuxSnapshot {
    pub id: usize,
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub route_to_master: bool,
    pub routed_subgroups: Vec<usize>,
    pub output_left_channel: u16,
    pub output_right_channel: u16,
    pub selected_output_device: Option<String>,
    
    // Aux FX
    pub reverb: Option<ReverbSnapshot>,
    pub delay: Option<DelaySnapshot>,
}

/// Reverb snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReverbSnapshot {
    pub enabled: bool,
    pub room_size: f32,
    pub damping: f32,
    pub wet: f32,
    pub width: f32,
    pub pre_delay: f32,
}

/// Delay snapshot
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DelaySnapshot {
    pub enabled: bool,
    pub time_l: f32,
    pub time_r: f32,
    pub feedback: f32,
    pub mix: f32,
}
