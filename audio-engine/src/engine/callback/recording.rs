use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use crate::engine::get_available_disk_space_gb;
use crate::engine::recording::StreamingWavWriter;

/// Check if recording stats should be sent (every 1 second)
/// Returns (elapsed_seconds, file_size_bytes, available_space_gb) if ready
pub fn check_recording_stats(
    master_tap_enabled: &Arc<AtomicBool>,
    recording_start_time: &Arc<Mutex<Option<Instant>>>,
    recording_last_stats_time: &Arc<Mutex<Option<Instant>>>,
    recording_writer: &Arc<Mutex<Option<StreamingWavWriter>>>,
    recording_bit_depth: &Arc<Mutex<u32>>,
    recording_path: &Arc<Mutex<Option<PathBuf>>>,
) -> Option<(u64, u64, f32)> {
    // Check if recording is enabled
    if !master_tap_enabled.load(Ordering::Relaxed) {
        return None;
    }
    
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
                
                // Get number of samples written from streaming writer
                let num_samples = if let Ok(writer_opt) = recording_writer.try_lock() {
                    if let Some(writer) = writer_opt.as_ref() {
                        writer.get_samples_written()
                    } else {
                        0
                    }
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
                        get_available_disk_space_gb(p) as f32
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                
                // Update last stats time
                *last_stats_time = Some(now);
                
                return Some((elapsed_seconds, file_size_bytes, available_space_gb));
            }
        }
    }
    None
}
