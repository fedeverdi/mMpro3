// NDI Audio Stream Module
// Custom FFI implementation - audio + static video frame

use anyhow::{Result, Context};
use crossbeam_channel::{bounded, Sender, Receiver};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use super::ndi_ffi::NdiSender;

/// NDI Stream source selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NdiSource {
    Master,
    Subgroup(usize), // Dynamic subgroup ID
}

impl NdiSource {
    pub fn from_str(s: &str) -> Option<Self> {
        if s == "master" {
            return Some(NdiSource::Master);
        }
        
        // Parse "subgroupN" format
        if s.starts_with("subgroup") {
            if let Ok(id) = s[8..].parse::<usize>() {
                return Some(NdiSource::Subgroup(id));
            }
        }
        
        None
    }
}

/// Audio frame to send to NDI thread
struct AudioFrame {
    samples: Vec<f32>, // Interleaved stereo L, R, L, R...
    sample_rate: u32,
}

pub struct NdiStream {
    active: Arc<AtomicBool>,
    stream_name: Arc<Mutex<String>>,
    source: Arc<Mutex<NdiSource>>,
    sample_rate: Arc<Mutex<u32>>,
    video_text: Arc<Mutex<String>>,
    
    // Channel to send audio to NDI thread (non-blocking from RT thread)
    audio_tx: Arc<Mutex<Option<Sender<AudioFrame>>>>,
    
    // NDI thread handle
    ndi_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    
    // Accumulator for samples (collect multiple callbacks into one NDI frame)
    sample_buffer: Arc<Mutex<Vec<f32>>>,
    target_samples_per_frame: Arc<Mutex<usize>>, // ~10ms worth of audio
}

impl NdiStream {
    pub fn new() -> Self {
        eprintln!("[NDI] Initializing NDI Stream module");
        
        Self {
            active: Arc::new(AtomicBool::new(false)),
            stream_name: Arc::new(Mutex::new("MMpro3 Audio".to_string())),
            source: Arc::new(Mutex::new(NdiSource::Master)),
            sample_rate: Arc::new(Mutex::new(48000)),
            video_text: Arc::new(Mutex::new("MMpro3".to_string())),
            audio_tx: Arc::new(Mutex::new(None)),
            ndi_thread: Arc::new(Mutex::new(None)),
            sample_buffer: Arc::new(Mutex::new(Vec::with_capacity(16384))),
            target_samples_per_frame: Arc::new(Mutex::new(1920)), // 40ms at 48kHz
        }
    }

    /// Start NDI streaming
    pub fn start(&self, stream_name: String, source: NdiSource) -> Result<()> {
        eprintln!("[NDI] Starting stream: '{}' from source: {:?}", stream_name, source);
        
        // Check if already active
        if self.active.load(Ordering::Relaxed) {
            eprintln!("[NDI] Stream already active");
            return Ok(());
        }
        
        // Update configuration
        if let Ok(mut name) = self.stream_name.lock() {
            *name = stream_name.clone();
        }
        if let Ok(mut src) = self.source.lock() {
            *src = source;
        }
        
        // Initialize NDI (try to load SDK)
        match self.start_ndi_sender(stream_name) {
            Ok((tx, handle)) => {
                // Store channel and thread
                if let Ok(mut audio_tx) = self.audio_tx.lock() {
                    *audio_tx = Some(tx);
                }
                if let Ok(mut ndi_thread) = self.ndi_thread.lock() {
                    *ndi_thread = Some(handle);
                }
                
                self.active.store(true, Ordering::Relaxed);
                eprintln!("[NDI] ✓ Stream started successfully");
                Ok(())
            }
            Err(e) => {
                eprintln!("[NDI] ✗ Failed to start NDI sender: {}", e);
                eprintln!("[NDI] ");
                eprintln!("[NDI] TROUBLESHOOTING:");
                eprintln!("[NDI] 1. Install NDI Runtime from: https://ndi.video/tools/");
                eprintln!("[NDI]    Download 'NDI Tools' and install the NDI Runtime");
                eprintln!("[NDI] 2. On macOS, you may need to grant network permissions");
                eprintln!("[NDI] 3. Check firewall settings allow NDI traffic");
                eprintln!("[NDI] ");
                Err(e)
            }
        }
    }

    /// Set custom text for video frame
    pub fn set_video_text(&self, text: String) {
        if let Ok(mut video_text) = self.video_text.lock() {
            *video_text = text;
        }
    }

    /// Internal: Start NDI sender thread
    fn start_ndi_sender(&self, stream_name: String) -> Result<(Sender<AudioFrame>, thread::JoinHandle<()>)> {
        // Create channel for audio frames with extra large capacity
        let (tx, rx): (Sender<AudioFrame>, Receiver<AudioFrame>) = bounded(128);
        
        // Clone video_text Arc to pass to thread
        let video_text = Arc::clone(&self.video_text);
        
        // Spawn NDI sender thread
        let handle = thread::Builder::new()
            .name("ndi-sender".to_string())
            .spawn(move || {
                ndi_sender_thread(stream_name, rx, video_text);
            })
            .context("Failed to spawn NDI thread")?;
        
        Ok((tx, handle))
    }

    /// Stop NDI streaming
    pub fn stop(&self) -> Result<()> {
        eprintln!("[NDI] Stopping stream...");
        
        if !self.active.load(Ordering::Relaxed) {
            eprintln!("[NDI] Stream already stopped");
            return Ok(());
        }
        
        self.active.store(false, Ordering::Relaxed);
        
        // Drop channel to signal thread to stop
        if let Ok(mut audio_tx) = self.audio_tx.lock() {
            *audio_tx = None;
        }
        
        // Wait for thread to finish
        if let Ok(mut ndi_thread) = self.ndi_thread.lock() {
            if let Some(handle) = ndi_thread.take() {
                let _ = handle.join();
            }
        }
        
        // Clear buffer
        if let Ok(mut buffer) = self.sample_buffer.lock() {
            buffer.clear();
        }
        
        eprintln!("[NDI] ✓ Stream stopped");
        Ok(())
    }

    /// Check if streaming is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Set stream name (requires restart to take effect)
    pub fn set_stream_name(&self, name: String) -> Result<()> {
        eprintln!("[NDI] Setting stream name: '{}' (restart required)", name);
        
        if let Ok(mut stream_name) = self.stream_name.lock() {
            *stream_name = name;
        }
        
        Ok(())
    }

    /// Set audio source (master or subgroup)
    pub fn set_source(&self, source: NdiSource) -> Result<()> {
        eprintln!("[NDI] Setting source: {:?}", source);
        
        if let Ok(mut src) = self.source.lock() {
            *src = source;
        }
        
        Ok(())
    }

    /// Get current source
    pub fn get_source(&self) -> NdiSource {
        self.source.lock().map(|s| *s).unwrap_or(NdiSource::Master)
    }

    /// Update sample rate (call when audio engine sample rate changes)
    pub fn set_sample_rate(&self, sample_rate: u32) -> Result<()> {
        if let Ok(mut sr) = self.sample_rate.lock() {
            if *sr != sample_rate {
                eprintln!("[NDI] Sample rate changed: {} Hz -> {} Hz", *sr, sample_rate);
                *sr = sample_rate;
                
                // Update target samples per frame (40ms for ultra-smooth audio)
                if let Ok(mut target) = self.target_samples_per_frame.lock() {
                    *target = (sample_rate as f32 * 0.040) as usize; // 40ms
                }
            }
        }
        Ok(())
    }

    /// Send audio samples to NDI stream (called from audio callback - RT-safe)
    /// 
    /// # Arguments
    /// * `samples` - Stereo pair [L, R]
    pub fn send_audio(&self, samples: &[f32]) -> Result<()> {
        use std::sync::atomic::{AtomicBool, Ordering};
        static FIRST_CALL: AtomicBool = AtomicBool::new(true);
        
        if !self.is_active() || samples.len() != 2 {
            return Ok(());
        }
        
        if FIRST_CALL.swap(false, Ordering::Relaxed) {
            eprintln!("[NDI Stream] Audio streaming started");
        }
        
        // Get target frame size
        let target_size = self.target_samples_per_frame.lock()
            .map(|t| *t)
            .unwrap_or(1440);
        
        // Accumulate samples in buffer
        if let Ok(mut buffer) = self.sample_buffer.try_lock() {
            buffer.push(samples[0]); // Left
            buffer.push(samples[1]); // Right
            
            // When we have enough samples (~10ms), send to NDI thread
            if buffer.len() >= target_size * 2 {
                if let Ok(audio_tx) = self.audio_tx.lock() {
                    if let Some(ref tx) = *audio_tx {
                        let sample_rate = self.sample_rate.lock()
                            .map(|sr| *sr)
                            .unwrap_or(48000);
                        
                        let _max_sample = buffer.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
                        
                        let frame = AudioFrame {
                            samples: buffer.clone(),
                            sample_rate,
                        };
                        
                        // Try send (non-blocking, drops if channel full - acceptable for realtime)
                        let _ = tx.try_send(frame);
                        
                        buffer.clear();
                    }
                }
            }
        }
        
        Ok(())
    }

    fn get_config(&self) -> String {
        let active = self.is_active();
        let name = self.stream_name.lock()
            .map(|n| n.clone())
            .unwrap_or_else(|_| "Unknown".to_string());
        let source = self.get_source();
        let sr = self.sample_rate.lock()
            .map(|s| *s)
            .unwrap_or(0);
        
        format!(
            "NDI Stream: {} | Source: {:?} | {}Hz | Active: {}",
            name, source, sr, active
        )
    }
}

impl Drop for NdiStream {
    fn drop(&mut self) {
        eprintln!("[NDI] Shutting down NDI module...");
        let _ = self.stop();
    }
}

/// Create a static video frame (1920x1080) with custom text and audio info
fn create_video_frame(text: &str, sample_rate: u32, channels: u32, elapsed_secs: f64, frame_count: u64) -> Vec<u8> {
    use image::{Rgba, RgbaImage};
    use imageproc::drawing::draw_text_mut;
    use ab_glyph::{FontRef, PxScale};
    
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;
    
    // Create dark blue background (RGBA first, then convert to BGRA)
    let dark_blue = Rgba([15u8, 25u8, 60u8, 255u8]); // RGB: 15, 25, 60 (dark blue)
    let mut img = RgbaImage::from_pixel(WIDTH, HEIGHT, dark_blue);
    
    // Load embedded font (cross-platform Roboto font)
    let font_data = include_bytes!("../../assets/Roboto-Regular.ttf");
    let font = match FontRef::try_from_slice(font_data) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("[NDI Video] Warning: Could not load font, using plain background");
            // Convert RGBA to BGRA and return
            let mut bgra_frame = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
            for (i, pixel) in img.pixels().enumerate() {
                let idx = i * 4;
                bgra_frame[idx + 0] = pixel[2]; // B
                bgra_frame[idx + 1] = pixel[1]; // G
                bgra_frame[idx + 2] = pixel[0]; // R
                bgra_frame[idx + 3] = pixel[3]; // A
            }
            return bgra_frame;
        }
    };
    
    // Draw custom text in center
    let scale = PxScale::from(120.0);
    let white = Rgba([255u8, 255u8, 255u8, 255u8]);
    
    // Calculate text width for proper centering
    use ab_glyph::{Font, ScaleFont};
    let scaled_font = font.as_scaled(scale);
    
    let mut text_width = 0.0f32;
    let mut prev_glyph_id = None;
    
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let advance_width = scaled_font.h_advance(glyph_id);
        
        // Add kerning if available
        if let Some(prev) = prev_glyph_id {
            text_width += scaled_font.kern(prev, glyph_id);
        }
        
        text_width += advance_width;
        prev_glyph_id = Some(glyph_id);
    }
    
    // Center the text
    let text_x = ((WIDTH as f32 - text_width) / 2.0).max(0.0);
    let text_y = (HEIGHT / 2) - 60;
    
    draw_text_mut(&mut img, white, text_x as i32, text_y as i32, scale, &font, text);
    
    // Draw footer with audio stream info
    let footer_scale = PxScale::from(24.0);
    let gray = Rgba([180u8, 180u8, 180u8, 255u8]);
    
    let status_text = format!(
        "🔴 STREAMING  |  {}ch @ {}Hz  |  {} frames  |  {:.1}s",
        channels, sample_rate, frame_count, elapsed_secs
    );
    let footer_y = HEIGHT - 50;
    let footer_x = 40;
    
    draw_text_mut(&mut img, gray, footer_x, footer_y as i32, footer_scale, &font, &status_text);
    
    // Convert RGBA to BGRA (NDI expects BGRA)
    let mut bgra_frame = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    for (i, pixel) in img.pixels().enumerate() {
        let idx = i * 4;
        bgra_frame[idx + 0] = pixel[2]; // B
        bgra_frame[idx + 1] = pixel[1]; // G
        bgra_frame[idx + 2] = pixel[0]; // R
        bgra_frame[idx + 3] = pixel[3]; // A
    }
    
    bgra_frame
}

/// NDI sender thread - custom FFI implementation
fn ndi_sender_thread(stream_name: String, rx: Receiver<AudioFrame>, video_text: Arc<Mutex<String>>) {
    eprintln!("[NDI Thread] Initializing NDI...");
    
    // Create NDI sender
    let sender = match NdiSender::new(&stream_name) {
        Some(s) => {
            eprintln!("[NDI Thread] ✓ NDI sender '{}' created!", stream_name);
            eprintln!("[NDI Thread] Stream is now discoverable on the network");
            s
        }
        None => {
            eprintln!("[NDI Thread] ✗ Failed to create NDI sender");
            eprintln!("[NDI Thread] ");
            eprintln!("[NDI Thread] TROUBLESHOOTING:");
            eprintln!("[NDI Thread] 1. Install NDI Tools from: https://ndi.video/tools/");
            eprintln!("[NDI Thread]    This installs the NDI runtime library");
            eprintln!("[NDI Thread] 2. Restart the application after installation");
            eprintln!("[NDI Thread] ");
            eprintln!("[NDI Thread] Falling back to simulation mode...");
            
            // Fallback
            simulate_ndi_sender(stream_name, rx, video_text);
            return;
        }
    };
    
    // Create dark blue video frame with custom text
    let text = video_text.lock().unwrap_or_else(|e| {
        eprintln!("[NDI Thread] Warning: video_text mutex poisoned, using default");
        e.into_inner()
    }).clone();
    let mut video_frame = create_video_frame(&text, 48000, 2, 0.0, 0);
    let width = 1920i32;
    let height = 1080i32;
    eprintln!("[NDI Thread] Video frame ready with text '{}' ({} bytes)", text, video_frame.len());
    
    // Send initial video frame first
    eprintln!("[NDI Thread] Sending initial video frame...");
    sender.send_video(&video_frame, width, height, 10, 1);
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Send audio frames to establish format (40ms frames @ 48kHz, planar)
    eprintln!("[NDI Thread] Establishing audio format (40ms frames, planar)...");
    let mut init_audio: Vec<f32> = Vec::with_capacity(3840); // 1920 samples/ch * 2 channels
    for i in 0..1920 {
        let t = i as f32 / 48000.0;
        let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.05;
        init_audio.push(sample); // Left
        init_audio.push(sample); // Right
    }
    
    // Send 5 frames (200ms total)
    for _i in 0..5 {
        sender.send_audio(&init_audio, 48000, 2);
        std::thread::sleep(std::time::Duration::from_millis(40));
    }
    
    eprintln!("[NDI Thread] Audio format established (2ch, 48kHz, 1920 samples/ch, 40ms, PLANAR)");
    
    // Statistics
    let mut frame_count = 0u64;
    let mut total_samples = 0u64;
    let mut video_frame_count = 0u64;
    let start_time = std::time::Instant::now();
    let mut last_video_time = start_time;
    let mut last_video_update = start_time; // Track when to regenerate video frame
    let mut current_text = text.clone();
    
    eprintln!("[NDI Thread] Ready to stream audio + video...");
    
    let mut first_frame = true;
    
    // Main loop
    while let Ok(audio_frame) = rx.recv() {
        let num_samples = audio_frame.samples.len() / 2; // Stereo pairs
        let sample_rate = audio_frame.sample_rate;
        
        if first_frame {
            eprintln!("[NDI Thread] First audio frame received, streaming started");
            first_frame = false;
        }
        
        // Send audio to NDI (interleaved stereo float32)
        sender.send_audio(&audio_frame.samples, sample_rate as i32, 2);
        
        // Check if video text has changed and update frame if needed
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(start_time).as_secs_f64();
        
        if now.duration_since(last_video_update).as_secs() >= 1 {
            let new_text = video_text.lock().unwrap().clone();
            if new_text != current_text {
                current_text = new_text.clone();
            }
            // Always regenerate to update stats
            video_frame = create_video_frame(&current_text, sample_rate, 2, elapsed, frame_count);
            last_video_update = now;
        }
        
        // Send black video frame at 10fps so NDI monitor shows something
        if now.duration_since(last_video_time).as_millis() >= 100 {
            sender.send_video(&video_frame, width, height, 10, 1);
            video_frame_count += 1;
            last_video_time = now;
        }
        
        // Update statistics
        frame_count += 1;
        total_samples += num_samples as u64;
    }
    
    // Cleanup
    let elapsed = std::time::Instant::now().duration_since(start_time).as_secs_f64();
    eprintln!(
        "[NDI Thread] Stopped after {:.1}s: sent {} audio frames, {} video frames, {} samples total",
        elapsed, frame_count, video_frame_count, total_samples
    );
    
    drop(sender);
    eprintln!("[NDI Thread] ✓ NDI sender destroyed");
}

/// Simulation mode fallback when NDI SDK is not available
fn simulate_ndi_sender(stream_name: String, rx: Receiver<AudioFrame>, _video_text: Arc<Mutex<String>>) {
    eprintln!("[NDI Thread] Simulation mode: streaming audio from '{}'", stream_name);
    
    // Statistics
    let mut frame_count = 0u64;
    let mut total_samples = 0u64;
    let start_time = std::time::Instant::now();
    let mut last_log_time = start_time;
    
    while let Ok(audio_frame) = rx.recv() {
        let num_samples = audio_frame.samples.len() / 2; // Stereo pairs
        
        frame_count += 1;
        total_samples += num_samples as u64;
        
        // Log statistics every 5 seconds
        let now = std::time::Instant::now();
        if now.duration_since(last_log_time).as_secs() >= 5 {
            let elapsed = now.duration_since(start_time).as_secs_f64();
            let avg_samples_per_frame = if frame_count > 0 {
                total_samples as f64 / frame_count as f64
            } else {
                0.0
            };
            let frames_per_sec = frame_count as f64 / elapsed;
            
            eprintln!(
                "[NDI Thread] [SIMULATION] {} frames ({:.1} fps), {} samples, {:.1} samples/frame",
                frame_count, frames_per_sec, total_samples, avg_samples_per_frame
            );
            
            last_log_time = now;
        }
    }
    
    let elapsed = std::time::Instant::now().duration_since(start_time).as_secs_f64();
    eprintln!(
        "[NDI Thread] [SIMULATION] Stopped after {:.1}s: processed {} frames, {} samples total",
        elapsed, frame_count, total_samples
    );
}
