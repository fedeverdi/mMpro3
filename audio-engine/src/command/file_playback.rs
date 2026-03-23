use crate::engine::audio_engine::AudioEngine;
use crate::ipc::messages::{Command, Response};

impl AudioEngine {
    /// Handle file playback commands (PauseFile, StopFile, SeekFile, GetWaveformData, StopAllFiles)
    pub fn handle_file_playback_command(&self, command: Command) -> Option<Response> {
        match command {
            Command::PauseFile { track } => {
                let _ = self.pause_file(track);
                None
            }
            Command::StopFile { track } => {
                let _ = self.stop_file(track);
                None
            }
            Command::SeekFile { track, time_seconds } => {
                let _ = self.seek_file(track, time_seconds);
                None
            }
            Command::SetFilePlaybackRate { track, rate } => {
                let _ = self.set_file_playback_rate(track, rate);
                None
            }
            Command::GetWaveformData { track, num_points } => {
                match self.get_waveform_data(track, num_points) {
                    Ok((data, duration, sample_rate)) => {
                        Some(Response::WaveformData {
                            track,
                            data,
                            duration,
                            sample_rate,
                        })
                    }
                    Err(e) => {
                        eprintln!("[Engine] GetWaveformData FAILED for track {}: {}", track, e);
                        Some(Response::Error {
                            message: format!("Failed to get waveform data: {}", e),
                        })
                    }
                }
            }
            Command::StopAllFiles => {
                self.stop_all_files();
                None
            }
            _ => None,
        }
    }
}
