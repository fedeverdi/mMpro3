use crate::engine::audio_engine::AudioEngine;
use crate::ipc::messages::{Command, Response};

impl AudioEngine {
    /// Handle metering commands (Get/Reset for Loudness, DynamicRange, PhaseCorrelation, StereoWidth, Headroom)
    pub fn handle_metering_command(&self, command: Command) -> Option<Response> {
        match command {
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
            }
            Command::ResetLoudness => {
                let mut router = self.router.lock().unwrap();
                router.loudness_meter.reset();
                Some(Response::Ok {
                    message: "Loudness measurements reset".to_string(),
                })
            }
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
            }
            Command::ResetDynamicRange => {
                let mut router = self.router.lock().unwrap();
                router.dynamic_range_meter.reset();
                Some(Response::Ok {
                    message: "Dynamic range measurements reset".to_string(),
                })
            }
            Command::GetPhaseCorrelation => {
                let router = self.router.lock().unwrap();
                let phase_data = router.phase_correlation_meter.get_measurement();
                Some(Response::PhaseCorrelationData {
                    correlation: phase_data.correlation,
                    mono_compatible: phase_data.mono_compatible,
                })
            }
            Command::ResetPhaseCorrelation => {
                let mut router = self.router.lock().unwrap();
                router.phase_correlation_meter.reset();
                Some(Response::Ok {
                    message: "Phase correlation measurements reset".to_string(),
                })
            }
            Command::GetStereoWidth => {
                let router = self.router.lock().unwrap();
                let width_data = router.stereo_width_meter.get_measurement();
                Some(Response::StereoWidthData {
                    width_percent: width_data.width_percent,
                    mid_rms: width_data.mid_rms,
                    side_rms: width_data.side_rms,
                    balance: width_data.balance,
                })
            }
            Command::ResetStereoWidth => {
                let mut router = self.router.lock().unwrap();
                router.stereo_width_meter.reset();
                Some(Response::Ok {
                    message: "Stereo width measurements reset".to_string(),
                })
            }
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
            }
            Command::ResetHeadroom => {
                let mut router = self.router.lock().unwrap();
                router.headroom_meter.reset();
                Some(Response::Ok {
                    message: "Headroom measurements reset".to_string(),
                })
            }
            _ => None,
        }
    }
}
