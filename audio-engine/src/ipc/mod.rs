// IPC communication module
pub mod messages;

pub use messages::{
    Command, Response, ParametricFilter,
    TrackMeters, SubgroupMeters, AuxMeters,
    TrackParameters, SubgroupParameters, AuxParameters, MasterParameters,
    HeadroomDataStruct, LoudnessDataStruct,
    DynamicRangeDataStruct, PhaseCorrelationDataStruct, StereoWidthDataStruct,
    MasterFxEffect, AuxSendData, AuxReverbParams, AuxDelayParams,
    InsertEffectInfo, EQPresetData, EQPresetFilter,
};
