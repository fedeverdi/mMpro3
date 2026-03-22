use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Stream;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use std::path::PathBuf;

use crate::io::{AudioIO, ChannelSelection, DeviceInfo, NdiStream};
use crate::processing::{Router, InsertEffectData};
use crate::ipc::*;
use crate::engine::{
    load_audio_config,
    load_license_from_file,
};
use crate::engine::callback::{frame_output, metering, fft, performance::PerformanceStats, recording as rec_stats};
use crate::processing::track;
use crate::processing::file_player;
use crate::io::ndi_stream;

pub struct AudioEngine {
    pub(crate) audio_io: AudioIO,
    pub(crate) router: Arc<Mutex<Router>>,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    sample_rate: u32,
    input_sample_rate: u32, // Track input sample rate (may differ from output)
    output_buffer_size: Option<u32>, // Track output buffer size for input matching
    pub(crate) updates_suspended: Arc<AtomicBool>,
    active_stream_id: Arc<AtomicUsize>, // ID of the currently active output stream
    input_buffer: Arc<Mutex<Vec<f32>>>, // Shared buffer - always contains latest input frame
    input_channels: Arc<AtomicUsize>,
    input_users: HashSet<usize>, // Track IDs that are using audio input
    current_input_device: Option<String>, // Currently open input device name
    recording_writer: Arc<Mutex<Option<crate::engine::recording::StreamingWavWriter>>>, // Streaming WAV writer for recording
    master_tap_enabled: Arc<AtomicBool>, // Enable/disable master tap
    recording_path: Arc<Mutex<Option<PathBuf>>>, // Path where to save the recording
    recording_start_time: Arc<Mutex<Option<Instant>>>, // Recording start time for elapsed calculation
    recording_last_stats_time: Arc<Mutex<Option<Instant>>>, // Last time stats were sent (for 1-second interval)
    recording_sample_rate: Arc<Mutex<u32>>, // Recording sample rate (configurable)
    recording_bit_depth: Arc<Mutex<u32>>, // Recording bit depth (16, 24, or 32)
    recording_format: Arc<Mutex<String>>, // Recording format ("wav", "mp3", "opus")
    output_sender: mpsc::SyncSender<String>, // Non-blocking channel for sending updates to frontend
    pub(crate) ndi_stream: Arc<NdiStream>, // NDI audio streaming
    pub(crate) selected_master_output: Arc<Mutex<Option<String>>>, // Selected master output device ID
    pub available_output_devices: Vec<DeviceInfo>, // List of available output devices
}

impl AudioEngine {
    pub fn new() -> Self {
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
        
        // Load audio configuration to get sample rate
        let config = load_audio_config();
        let initial_sample_rate = if config.sample_rate == 0 { 
            48000.0  // Default to 48kHz if Auto
        } else { 
            config.sample_rate as f32 
        };
        
        let router = Arc::new(Mutex::new(Router::new(24, num_aux_buses, initial_sample_rate))); // Support up to 24 tracks + dynamic aux count
        let updates_suspended = Arc::new(AtomicBool::new(false));
        let input_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let input_channels = Arc::new(AtomicUsize::new(2)); // Default stereo
        let recording_writer = Arc::new(Mutex::new(None)); // Streaming WAV writer (created on recording start)
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
            recording_writer,
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

    pub fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        self.audio_io.list_devices()
    }

    /// Clean up existing streams before restart
    fn cleanup_existing_streams(&mut self) {
        if self.input_stream.is_none() && self.output_stream.is_none() {
            return;
        }
        
        eprintln!("[Engine] Streams still active, forcing stop before restart...");
        
        // CRITICAL: Suspend updates to prevent race conditions during device/sample rate change
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
        
        // Clear buffers
        if let Ok(mut buffer) = self.input_buffer.lock() {
            buffer.clear();
        }
        // Finalize any active recording
        if let Ok(mut writer) = self.recording_writer.lock() {
            if let Some(w) = writer.take() {
                let _ = w.finalize(); // Ignore errors during cleanup
            }
        }
        
        // Wait for OS to release audio hardware
        std::thread::sleep(std::time::Duration::from_millis(100));
        eprintln!("[Engine] System cleanup complete");
    }

    /// Log device configuration details
    fn log_device_configuration(
        &self,
        device_name: &str,
        output_channels: usize,
        buffer_size: cpal::BufferSize,
        is_bluetooth: bool,
    ) {
        match buffer_size {
            cpal::BufferSize::Fixed(size) => {
                let latency_ms = (size as f32 / self.sample_rate as f32) * 1000.0;
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
                eprintln!("[Engine] Audio Configuration");
                eprintln!("[Engine] ───────────────────────────────────────────────────");
                eprintln!("[Engine] Device: {}", device_name);
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
                eprintln!("[Engine] Device: {}", device_name);
                eprintln!("[Engine] Sample Rate: {} Hz", self.sample_rate);
                eprintln!("[Engine] Output Channels: {}", output_channels);
                eprintln!("[Engine] Buffer Size: DEFAULT (system auto)");
                eprintln!("[Engine] Type: {}", if is_bluetooth { "Bluetooth" } else { "Wired" });
                eprintln!("[Engine] ═══════════════════════════════════════════════════");
            },
        }
    }

    /// Update sample rate for all audio components (router, tracks, aux buses, master)
    fn update_sample_rate_for_components(&mut self, sample_rate_changed: bool) {
        let mut router = self.router.lock().unwrap();
        
        for track in router.tracks.iter_mut() {
            track.set_sample_rate(self.sample_rate as f32);
        }
        
        for aux_bus in router.aux_buses.iter_mut() {
            aux_bus.set_sample_rate(self.sample_rate as f32);
        }
        
        router.master.parametric_eq.set_sample_rate(self.sample_rate as f32);
        router.master.set_sample_rate(self.sample_rate as f32);
        
        if sample_rate_changed {
            eprintln!("[Engine] All components updated to {} Hz", self.sample_rate);
        }
    }

    pub(crate) fn start(
        &mut self,
        input_device_name: Option<String>,
        output_device_name: Option<String>,
        sample_rate: Option<u32>,
        buffer_size: Option<u32>,
    ) -> Result<()> {
        
        // Load saved audio configuration if not explicitly specified
        let (final_sample_rate, final_buffer_size) = if sample_rate.is_none() && buffer_size.is_none() {
            // Both not specified - load from file
            let config = load_audio_config();
            let sr = if config.sample_rate == 0 { None } else { Some(config.sample_rate) };
            let bs = if config.buffer_size == 0 { None } else { Some(config.buffer_size) };
            (sr, bs)
        } else {
            // At least one was specified - use what was provided
            (sample_rate, buffer_size)
        };
        
        // Force stop if streams are still active (restart scenario)
        self.cleanup_existing_streams();

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
        let actual_buffer_size = output_config.buffer_size;
        
        // Log configuration
        self.log_device_configuration(&output_device_name, output_channels, actual_buffer_size, is_bluetooth);
                
        // Update sample rate for all active file players and equalizers
        self.update_sample_rate_for_components(sample_rate_changed);

        // Update NDI stream sample rate
        let _ = self.ndi_stream.set_sample_rate(self.sample_rate);

        let _input_channels = 2; // Default stereo (not used since input is disabled)

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
        let recording_writer = Arc::clone(&self.recording_writer);
        let master_tap_enabled = Arc::clone(&self.master_tap_enabled);
        let recording_start_time = Arc::clone(&self.recording_start_time);
        let recording_last_stats_time = Arc::clone(&self.recording_last_stats_time);
        let recording_path = Arc::clone(&self.recording_path);
        let recording_bit_depth = Arc::clone(&self.recording_bit_depth);
        let ndi_stream = Arc::clone(&self.ndi_stream);
        
        // Clone Arc for selected output (thread-safe shared access)
        let _selected_master_output = Arc::clone(&self.selected_master_output);
        
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

                        // Write master output to file (if recording enabled)
                        if master_tap_enabled.load(Ordering::Relaxed) {
                            if let Ok(mut writer_opt) = recording_writer.try_lock() {
                                if let Some(writer) = writer_opt.as_mut() {
                                    // Write stereo pair directly to disk (streaming)
                                    let frame = [master_left.clamp(-1.0, 1.0), master_right.clamp(-1.0, 1.0)];
                                    let _ = writer.write_samples(&frame); // Ignore errors in real-time callback
                                }
                            }
                        }

                        // Send audio to NDI stream (if active)
                        // Get the selected source audio
                        if ndi_stream.is_active() {
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

                        // Write all outputs to buffer using frame_output helpers
                        frame_output::write_output_frame(data, frame_idx, output_channels, master_left, master_right, &router);
                        frame_output::write_subgroup_outputs(data, frame_idx, output_channels, &router);
                        frame_output::write_aux_outputs(data, frame_idx, output_channels, &router);
                    }

                    // Check if we need to send meter updates
                    let mut counter = meter_update_frames.lock().unwrap();
                    *counter += frames;
                    
                    let levels_to_send = if *counter >= meter_interval {
                        *counter = 0;
                        
                        // Collect all meter data using helper function
                        Some(metering::collect_meter_data(&mut router))
                    } else {
                        None
                    };

                    levels_to_send
                }; // Lock is released here
                
                // Collect FFT data outside the lock using helpers
                let fft_data = {
                    let mut router = router_output.lock().unwrap();
                    fft::collect_master_fft(&mut router)
                };
                let track_fft_data = {
                    let mut router = router_output.lock().unwrap();
                    fft::collect_track_fft(&mut router)
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
                let _cpu_usage = (process_time_us as f32 / buffer_duration_us as f32) * 100.0;
                
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
                if let Some((elapsed_seconds, file_size_bytes, available_space_gb)) = rec_stats::check_recording_stats(
                    &master_tap_enabled,
                    &recording_start_time,
                    &recording_last_stats_time,
                    &recording_writer,
                    &recording_bit_depth,
                    &recording_path,
                ) {
                    let response = Response::RecordingStats {
                        elapsed_seconds,
                        file_size_bytes,
                        available_space_gb,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = output_sender.try_send(json);
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

    pub(crate) fn stop(&mut self) -> Result<()> {
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
        // Finalize any active recording
        if let Ok(mut writer) = self.recording_writer.lock() {
            if let Some(w) = writer.take() {
                let _ = w.finalize(); // Ignore errors during cleanup
            }
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
    pub(crate) fn enable_master_tap(&self, file_path: String, _sample_rate: u32, bit_depth: u32, format: &str) {
        // Create streaming WAV writer and write header immediately
        let path = PathBuf::from(file_path);
        
        // Close any existing writer first
        if let Ok(mut writer) = self.recording_writer.lock() {
            if let Some(w) = writer.take() {
                let _ = w.finalize(); // Finalize previous recording
            }
            
            // Create new streaming writer
            // Note: We always record at the audio device's sample rate (self.sample_rate)
            // because we capture samples directly from the audio callback.
            // The requested sample_rate is ignored to avoid quality loss from resampling.
            match crate::engine::recording::StreamingWavWriter::new(&path, self.sample_rate, bit_depth) {
                Ok(w) => {
                    *writer = Some(w);
                },
                Err(e) => {
                    eprintln!("[Engine] ✗ Failed to create recording file: {}", e);
                    return;
                }
            }
        }
        
        // Set recording path for stats reporting
        if let Ok(mut rec_path) = self.recording_path.lock() {
            *rec_path = Some(path);
        }
        
        // Set recording parameters
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
        
        // Enable recording (audio callback will start writing samples)
        self.master_tap_enabled.store(true, Ordering::Relaxed);
    }

    pub(crate) fn disable_master_tap(&self) {
        // Disable recording flag first (stops audio callback from writing)
        self.master_tap_enabled.store(false, Ordering::Relaxed);
        
        // Clear recording times
        if let Ok(mut start_time) = self.recording_start_time.lock() {
            *start_time = None;
        }
        if let Ok(mut last_stats_time) = self.recording_last_stats_time.lock() {
            *last_stats_time = None;
        }
        
        // Finalize the WAV file (updates header and closes file)
        let path = if let Ok(mut p) = self.recording_path.lock() {
            p.take()
        } else {
            None
        };
        
        if let Ok(mut writer) = self.recording_writer.lock() {
            if let Some(w) = writer.take() {
                match w.finalize() {
                    Ok(_) => {
                        if let Some(file_path) = path {
                            eprintln!("[Engine] ✓ Recording saved: {:?}", file_path);
                        }
                    },
                    Err(e) => eprintln!("[Engine] ✗ Failed to finalize recording: {}", e),
                }
            } else {
                eprintln!("[Engine] ✓ Master tap disabled (no active recording)");
            }
        }
    }

    // Track source commands
    pub(crate) fn set_track_source_input(&mut self, track: usize, left_ch: u16, right_ch: u16, device_name: Option<String>) -> Result<()> {
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

    pub(crate) fn set_track_source_signal(&mut self, track: usize, waveform: &str, frequency: f32) -> Result<()> {
        // Close input stream when track switches away from audio input
        if let Err(e) = self.close_audio_input(track) {
            eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
        }
        
        let mut router = self.router.lock().unwrap();
        track::set_source_signal(&mut router, track, waveform, frequency, self.sample_rate)
    }

    pub(crate) fn set_signal_frequency(&mut self, track: usize, frequency: f32) -> Result<()> {
        crate::engine::track_control::set_signal_frequency_impl(&self.router, track, frequency)
    }

    pub(crate) fn set_signal_waveform(&mut self, track: usize, waveform: &str) -> Result<()> {
        crate::engine::track_control::set_signal_waveform_impl(&self.router, track, waveform)
    }

    pub(crate) fn clear_track_source(&mut self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::clear_source(&mut router, track)
    }

    pub(crate) fn set_track_source_file(&mut self, track: usize, file_path: &str, artist: Option<&str>, title: Option<&str>, playlist_id: Option<&str>, playlist_name: Option<&str>, playlist_index: Option<usize>) -> Result<()> {
        // Close input stream when track switches away from audio input
        if let Err(e) = self.close_audio_input(track) {
            eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
        }
        
        // CRITICAL: Load file WITHOUT holding router lock to avoid audio dropouts
        // This operation can take 100-500ms for large files
        let mut player = file_player::AudioFilePlayer::new();
        player.load_file(file_path)?;
        // Store the full absolute path so snapshots can restore the file correctly.
        // load_file() only keeps the basename, which breaks snapshot reloads.
        player.file_name = file_path.to_string();
        player.set_output_sample_rate(self.sample_rate);
        
        // Set metadata if provided
        if let Some(a) = artist {
            player.file_artist = Some(a.to_string());
        }
        if let Some(t) = title {
            player.file_title = Some(t.to_string());
        }
        
        // Now quickly assign the pre-loaded player to the track (fast operation)
        crate::engine::track_control::set_track_source_file_impl(
            &self.router,
            track,
            player,
            playlist_id.map(|s| s.to_string()),
            playlist_name.map(|s| s.to_string()),
            playlist_index,
        )
    }

    pub(crate) fn set_track_source_aux_return(&mut self, track: usize, aux: usize) -> Result<()> {
        // Close input stream when track switches away from audio input
        if let Err(e) = self.close_audio_input(track) {
            eprintln!("[Engine] Failed to close audio input for track {}: {}", track, e);
        }
        
        crate::engine::track_control::set_track_source_aux_return_impl(&self.router, track, aux)
    }

    pub(crate) fn play_file(&mut self, track: usize, file_path: Option<&str>, artist: Option<&str>, title: Option<&str>) -> Result<()> {
        // If file_path is provided, set the source file first
        if let Some(path) = file_path {
            self.set_track_source_file(track, path, artist, title, None, None, None)?;
        }
        
        let mut router = self.router.lock().unwrap();
        track::play_file(&mut router, track, self.sample_rate)
    }

    pub(crate) fn pause_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::pause_file(&mut router, track)
    }

    pub(crate) fn stop_file(&self, track: usize) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::stop_file(&mut router, track)
    }

    pub(crate) fn seek_file(&self, track: usize, time_seconds: f32) -> Result<()> {
        let mut router = self.router.lock().unwrap();
        track::seek_file(&mut router, track, time_seconds)
    }

    pub(crate) fn get_waveform_data(&self, track: usize, num_points: usize) -> Result<(Vec<f32>, f32, u32)> {
        crate::engine::track_control::get_waveform_data_impl(&self.router, track, num_points)
    }

    pub(crate) fn stop_all_files(&self) {
        let mut router = self.router.lock().unwrap();
        router.stop_all_files();
    }

    // Track controls
    pub(crate) fn set_gain(&self, track: usize, gain: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_gain(&mut router, track, gain);
    }

    pub(crate) fn set_volume(&self, track: usize, volume: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_volume(&mut router, track, volume);
    }

    pub(crate) fn set_mute(&self, track: usize, mute: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_mute(&mut router, track, mute);
    }

    pub(crate) fn set_solo(&self, track: usize, solo: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_solo(&mut router, track, solo);
    }

    pub(crate) fn set_route_to_master(&self, track: usize, route: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_route_to_master(&mut router, track, route);
    }

    pub(crate) fn set_pan(&self, track: usize, pan: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_pan(&mut router, track, pan);
    }

    pub(crate) fn set_pad(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_pad(&mut router, track, enabled);
    }

    pub(crate) fn set_hpf(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_hpf(&mut router, track, enabled);
    }

    pub(crate) fn set_phase_invert(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_phase_invert(&mut router, track, enabled);
    }

    pub(crate) fn set_pfl(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_pfl(&mut router, track, enabled);
    }

    // Track dynamics controls
    pub(crate) fn set_compressor(&self, track: usize, enabled: bool, threshold: f32, ratio: f32, attack: f32, release: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.tracks.get_mut(track) {
            // Find first compressor in insert chain and modify it
            // TODO: Add logic to create new compressor if none exists
            for slot in &mut t.inserts {
                if let InsertEffectData::Compressor(comp) = &mut slot.effect {
                    slot.enabled = enabled;
                    comp.set_enabled(enabled);
                    comp.set_threshold(threshold);
                    comp.set_ratio(ratio);
                    comp.set_attack(attack);
                    comp.set_release(release);
                    break; // Modify only the first compressor found
                }
            }
        }
    }

    pub(crate) fn set_gate(&self, track: usize, enabled: bool, threshold: f32, range: f32, attack: f32, release: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(t) = router.tracks.get_mut(track) {
            // Find first gate in insert chain and modify it
            // TODO: Add logic to create new gate if none exists
            for slot in &mut t.inserts {
                if let InsertEffectData::Gate(gate) = &mut slot.effect {
                    slot.enabled = enabled;
                    gate.set_enabled(enabled);
                    gate.set_threshold(threshold);
                    gate.set_range(range);
                    gate.set_attack(attack);
                    gate.set_release(release);
                    break; // Modify only the first gate found
                }
            }
        }
    }

    // Track EQ controls
    pub(crate) fn set_eq(&self, track: usize, low: f32, low_mid: f32, high_mid: f32, high: f32) {
        let mut router = self.router.lock().unwrap();
        track::set_eq(&mut router, track, low, low_mid, high_mid, high);
    }

    pub(crate) fn set_eq_enabled(&self, track: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        track::set_eq_enabled(&mut router, track, enabled);
    }

    // Parametric EQ controls
    pub(crate) fn set_parametric_eq_filters(&self, track: usize, filters: &[ParametricFilter]) {
        crate::engine::track_control::set_parametric_eq_filters_impl(&self.router, track, filters);
    }

    pub(crate) fn set_parametric_eq_enabled(&self, track: usize, enabled: bool) {
        crate::engine::track_control::set_parametric_eq_enabled_impl(&self.router, track, enabled);
    }

    pub(crate) fn clear_parametric_eq(&self, track: usize) {
        crate::engine::track_control::clear_parametric_eq_impl(&self.router, track);
    }

    // Master controls
    pub(crate) fn set_master_gain(&self, gain: f32) {
        crate::engine::master_control::set_master_gain_impl(&self.router, gain);
    }

    pub(crate) fn set_master_gain_left(&self, gain: f32) {
        crate::engine::master_control::set_master_gain_left_impl(&self.router, gain);
    }

    pub(crate) fn set_master_gain_right(&self, gain: f32) {
        crate::engine::master_control::set_master_gain_right_impl(&self.router, gain);
    }

    pub(crate) fn set_master_mute(&self, mute: bool) {
        crate::engine::master_control::set_master_mute_impl(&self.router, mute);
    }

    pub(crate) fn set_master_linked(&self, linked: bool) {
        crate::engine::master_control::set_master_linked_impl(&self.router, linked);
    }

    pub(crate) fn set_master_parametric_eq_filters(&self, filters: &[ParametricFilter]) {
        crate::engine::master_control::set_master_parametric_eq_filters_impl(&self.router, filters);
    }

    pub(crate) fn set_master_parametric_eq_enabled(&self, enabled: bool) {
        crate::engine::master_control::set_master_parametric_eq_enabled_impl(&self.router, enabled);
    }

    pub(crate) fn clear_master_parametric_eq(&self) {
        crate::engine::master_control::clear_master_parametric_eq_impl(&self.router);
    }

    pub(crate) fn set_master_output_channels(&self, left_ch: u16, right_ch: u16) {
        crate::engine::master_control::set_master_output_channels_impl(&self.router, left_ch, right_ch);
    }

    // Master FX methods
    pub(crate) fn set_master_compressor(&self, enabled: bool, threshold: f32, ratio: f32, attack: f32, release: f32) {
        crate::engine::master_fx_control::set_master_compressor_impl(&self.router, enabled, threshold, ratio, attack, release);
    }

    pub(crate) fn set_master_limiter(&self, enabled: bool, ceiling: f32, release: f32) {
        crate::engine::master_fx_control::set_master_limiter_impl(&self.router, enabled, ceiling, release);
    }

    pub(crate) fn set_master_delay(&self, enabled: bool, time_l: f32, time_r: f32, feedback: f32, mix: f32) {
        crate::engine::master_fx_control::set_master_delay_impl(&self.router, enabled, time_l, time_r, feedback, mix);
    }

    pub(crate) fn set_master_reverb(&self, enabled: bool, room_size: f32, damping: f32, wet: f32, width: f32, pre_delay: f32) {
        crate::engine::master_fx_control::set_master_reverb_impl(&self.router, enabled, room_size, damping, wet, width, pre_delay);
    }

    /// Get current state of all Master FX effects for synchronization
    pub(crate) fn get_master_fx_effects(&self) -> Vec<MasterFxEffect> {
        crate::engine::master_fx_control::get_master_fx_effects_impl(&self.router)
    }

    /// Add a Master FX effect to the FX list
    pub(crate) fn add_master_fx_effect(&self, effect_type: &str) {
        crate::engine::master_fx_control::add_master_fx_effect_impl(&self.router, effect_type);
    }

    /// Remove a Master FX effect from the FX list
    pub(crate) fn remove_master_fx_effect(&self, effect_type: &str) {
        crate::engine::master_fx_control::remove_master_fx_effect_impl(&self.router, effect_type);
    }

    // Subgroup methods
    pub(crate) fn add_subgroup(&self) -> usize {
        let mut router = self.router.lock().unwrap();
        router.add_subgroup()
    }

    pub(crate) fn remove_subgroup(&self, subgroup: usize) {
        let mut router = self.router.lock().unwrap();
        router.remove_subgroup(subgroup);
    }

    pub(crate) fn set_subgroup_gain(&self, subgroup: usize, gain: f32) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.gain = gain.max(0.0);
        }
    }

    pub(crate) fn set_subgroup_mute(&self, subgroup: usize, mute: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.mute = mute;
        }
    }

    pub(crate) fn set_subgroup_output_enabled(&self, subgroup: usize, enabled: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.output_enabled = enabled;
        }
    }

    pub(crate) fn set_subgroup_route_to_master(&self, subgroup: usize, route: bool) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.route_to_master = route;
        }
    }

    pub(crate) fn set_subgroup_output_channels(&self, subgroup: usize, left_ch: u16, right_ch: u16) {
        let mut router = self.router.lock().unwrap();
        if let Some(sg) = router.get_subgroup_mut(subgroup) {
            sg.output_channel_selection = ChannelSelection::new(left_ch, right_ch);
        }
    }

    pub(crate) fn set_track_route_to_subgroup(&self, track: usize, subgroup: usize, route: bool) {
        crate::engine::track_control::set_track_route_to_subgroup_impl(&self.router, track, subgroup, route);
    }

    // Aux bus methods
    pub(crate) fn set_track_aux_send(&self, track: usize, aux: usize, level: f32, pre_fader: bool, muted: bool) {
        crate::engine::track_control::set_track_aux_send_impl(&self.router, track, aux, level, pre_fader, muted);
    }

    pub(crate) fn set_aux_bus_gain(&self, aux: usize, gain: f32) {
        crate::engine::aux_control::set_aux_bus_gain_impl(&self.router, aux, gain);
    }

    pub(crate) fn set_aux_bus_mute(&self, aux: usize, mute: bool) {
        crate::engine::aux_control::set_aux_bus_mute_impl(&self.router, aux, mute);
    }

    pub(crate) fn set_aux_bus_reverb(&self, aux: usize, enabled: bool, room_size: f32, damping: f32, wet: f32, width: f32, pre_delay: f32) {
        crate::engine::aux_control::set_aux_bus_reverb_impl(&self.router, aux, enabled, room_size, damping, wet, width, pre_delay);
    }

    pub(crate) fn set_aux_bus_delay(&self, aux: usize, enabled: bool, time: f32, feedback: f32, mix: f32) {
        crate::engine::aux_control::set_aux_bus_delay_impl(&self.router, aux, enabled, time, feedback, mix);
    }

    pub(crate) fn set_aux_bus_route_to_master(&self, aux: usize, route: bool) {
        crate::engine::aux_control::set_aux_bus_route_to_master_impl(&self.router, aux, route);
    }

    pub(crate) fn set_aux_bus_route_to_subgroup(&self, aux: usize, subgroup: usize, route: bool) {
        crate::engine::aux_control::set_aux_bus_route_to_subgroup_impl(&self.router, aux, subgroup, route);
    }

    pub(crate) fn set_aux_bus_output_enabled(&self, aux: usize, enabled: bool) {
        crate::engine::aux_control::set_aux_bus_output_enabled_impl(&self.router, aux, enabled);
    }

    pub(crate) fn set_aux_bus_output_channels(&self, aux: usize, left_ch: u16, right_ch: u16) {
        crate::engine::aux_control::set_aux_bus_output_channels_impl(&self.router, aux, left_ch, right_ch);
    }

    /// Handle a command and return an optional response (only for critical operations)
    pub fn handle_command(&mut self, command: Command) -> Option<Response> {
        match command {
            // ===== Stream and device commands =====
            Command::Start { .. } | Command::Stop | 
            Command::EnableMasterTap { .. } | Command::DisableMasterTap => {
                self.handle_stream_command(command)
            }
            
            // ===== Configuration commands =====
            Command::SaveLicense { .. } | Command::GetLicense |
            Command::SaveAudioConfig { .. } | Command::GetAudioConfig => {
                self.handle_config_command(command)
            }
            
            // ===== File playback commands =====
            Command::PauseFile { .. } | Command::StopFile { .. } | Command::SeekFile { .. } |
            Command::GetWaveformData { .. } | Command::StopAllFiles => {
                self.handle_file_playback_command(command)
            }
            
            // ===== Audio input commands =====
            Command::OpenAudioInput { .. } | Command::CloseAudioInput | 
            Command::ListAudioInputs => {
                self.handle_audio_input_command(command)
            }
            
            // ===== Metering commands =====
            Command::GetLoudness | Command::ResetLoudness |
            Command::GetDynamicRange | Command::ResetDynamicRange |
            Command::GetPhaseCorrelation | Command::ResetPhaseCorrelation |
            Command::GetStereoWidth | Command::ResetStereoWidth |
            Command::GetHeadroom | Command::ResetHeadroom => {
                self.handle_metering_command(command)
            }
            
            // ===== NDI streaming commands =====
            Command::StartNdi { .. } | Command::StopNdi |
            Command::SetNdiSource { .. } | Command::SetNdiName { .. } |
            Command::SetNdiVideoText { .. } => {
                self.handle_ndi_command(command)
            }
            
            // ===== Miscellaneous commands =====
            Command::ListDevices | Command::SetUpdatesSuspended { .. } => {
                self.handle_misc_command(command)
            }
            
            // ===== EQ Presets commands =====
            Command::GetEQPresets | Command::ApplyEQPreset { .. } => {
                self.handle_eq_presets_command(command)
            }
            
            // ===== Track Source commands =====
            Command::SetTrackSourceInput { .. } | Command::SetTrackSourceSignal { .. } |
            Command::SetSignalFrequency { .. } | Command::SetSignalWaveform { .. } |
            Command::ClearTrackSource { .. } | Command::SetTrackSourceFile { .. } |
            Command::PlayFile { .. } | Command::SetTrackSourceAuxReturn { .. } => {
                self.handle_track_source_command(command)
            }
            
            // ===== Track Control commands =====
            Command::SetGain { .. } | Command::SetVolume { .. } | Command::SetMute { .. } |
            Command::SetSolo { .. } | Command::SetPan { .. } | Command::SetRouteToMaster { .. } |
            Command::SetTrackPad { .. } | Command::SetTrackHPF { .. } | Command::SetTrackPhaseInvert { .. } |
            Command::SetTrackPFL { .. } | Command::SetTrackRouteToSubgroup { .. } => {
                self.handle_track_control_command(command)
            }
            
            // ===== Track Effects commands =====
            Command::SetCompressor { .. } | Command::SetGate { .. } | Command::SetEQ { .. } |
            Command::SetEQEnabled { .. } | Command::SetParametricEQFilters { .. } |
            Command::SetParametricEQEnabled { .. } | Command::ClearParametricEQ { .. } |
            Command::SetTrackAuxSend { .. } => {
                self.handle_track_effects_command(command)
            }
            
            // ===== Track Inserts commands =====
            Command::AddTrackInsert { .. } | Command::RemoveTrackInsert { .. } |
            Command::MoveTrackInsert { .. } | Command::SetTrackInsertEnabled { .. } |
            Command::SetTrackInsertCompressor { .. } | Command::SetTrackInsertGate { .. } |
            Command::SetTrackInsertReverb { .. } | Command::SetTrackInsertDelay { .. } |
            Command::SetTrackInsertExciter { .. } | Command::SetTrackInsertDeEsser { .. } |
            Command::SetTrackInsertChorus { .. } => {
                self.handle_track_inserts_command(command)
            }
            
            // ===== Master Control commands =====
            Command::SetMasterGain { .. } | Command::SetMasterGainLeft { .. } |
            Command::SetMasterGainRight { .. } | Command::SetMasterMute { .. } |
            Command::SetMasterLinked { .. } | Command::SetMasterParametricEQFilters { .. } |
            Command::SetMasterParametricEQEnabled { .. } | Command::ClearMasterParametricEQ |
            Command::SetMasterOutputChannels { .. } | Command::SetSelectedMasterOutput { .. } => {
                self.handle_master_control_command(command)
            }
            
            // ===== Master Effects commands =====
            Command::SetMasterCompressor { .. } | Command::SetMasterLimiter { .. } |
            Command::SetMasterDelay { .. } | Command::SetMasterReverb { .. } |
            Command::AddMasterFxEffect { .. } | Command::RemoveMasterFxEffect { .. } => {
                self.handle_master_effects_command(command)
            }
            
            // ===== Subgroups commands =====
            Command::AddSubgroup | Command::RemoveSubgroup { .. } |
            Command::SetSubgroupGain { .. } | Command::SetSubgroupMute { .. } |
            Command::SetSubgroupOutputEnabled { .. } | Command::SetSubgroupRouteToMaster { .. } |
            Command::SetSubgroupOutputChannels { .. } | Command::SetSelectedSubgroupOutput { .. } => {
                self.handle_subgroups_command(command)
            }
            
            // ===== Aux Buses commands =====
            Command::SetAuxBusGain { .. } | Command::SetAuxBusMute { .. } |
            Command::SetAuxBusReverb { .. } | Command::SetAuxBusDelay { .. } |
            Command::SetAuxBusRouteToMaster { .. } | Command::SetAuxBusOutputEnabled { .. } |
            Command::SetAuxBusOutputChannels { .. } | Command::SetAuxBusRouteToSubgroup { .. } |
            Command::SetAuxBusSelectedOutput { .. } => {
                self.handle_aux_buses_command(command)
            }
            
            // ===== Scene Snapshots commands =====
            Command::SaveSnapshot { name } => {
                if let Err(e) = crate::engine::snapshot_control::save_snapshot_impl(&self.router, &name) {
                    eprintln!("[Engine] Error saving snapshot: {}", e);
                }
                None
            }
            Command::LoadSnapshot { name } => {
                match crate::engine::snapshot_control::load_snapshot_impl(&self.router, &name) {
                    Err(e) => {
                        eprintln!("[Engine] Error loading snapshot: {}", e);
                        return None;
                    }
                    Ok(files_to_load) => {
                        // Reload actual audio files (requires AudioEngine, not just Router)
                        for item in files_to_load {
                            let (track, file_path, artist, title): (usize, String, Option<String>, Option<String>) = item;

                            // Legacy snapshots may store only the basename.  Resolve such paths
                            // against the Library directory ({userData}/Library on macOS) so that
                            // files imported via the library panel can still be found.
                            let resolved_path = if std::path::Path::new(&file_path).is_absolute() {
                                file_path.clone()
                            } else {
                                // Reconstruct full path: ~/Library/Application Support/mMpro3/Library/<file>
                                dirs::data_local_dir()
                                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                                    .join("mMpro3")
                                    .join("Library")
                                    .join(&file_path)
                                    .to_string_lossy()
                                    .to_string()
                            };

                            if let Err(e) = self.set_track_source_file(
                                track, &resolved_path,
                                artist.as_deref(), title.as_deref(),
                                None, None, None
                            ) {
                                eprintln!("[Engine] Warning: could not reload file for track {}: {}", track, e);
                            }
                        }
                    }
                }
                // Broadcast full state so frontend updates all UI elements
                Some(crate::engine::snapshot_control::build_full_parameters_changed(&self.router))
            }
            Command::ListSnapshots => {
                crate::engine::snapshot_control::list_snapshots_impl()
            }
            Command::GetSnapshot { name } => {
                crate::engine::snapshot_control::get_snapshot_impl(&name)
            }
            Command::DeleteSnapshot { name } => {
                if let Err(e) = crate::engine::snapshot_control::delete_snapshot_impl(&name) {
                    eprintln!("[Engine] Error deleting snapshot: {}", e);
                }
                None
            }
            Command::RenameSnapshot { old_name, new_name } => {
                if let Err(e) = crate::engine::snapshot_control::rename_snapshot_impl(&old_name, &new_name) {
                    eprintln!("[Engine] Error renaming snapshot: {}", e);
                }
                None
            }
            Command::PinSnapshot { name } => {
                match crate::engine::snapshot_control::pin_snapshot_impl(&name) {
                    Ok(_) => {},
                    Err(e) => eprintln!("[Engine] Error pinning snapshot: {}", e),
                }
                None
            }
        }
    }
}

