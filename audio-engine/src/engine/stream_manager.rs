// Stream Manager - Handles audio I/O stream operations
use anyhow::Result;
use cpal::Stream;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::collections::HashSet;

use crate::io::AudioIO;
use crate::processing::Router;

pub struct StreamManager {
    pub audio_io: AudioIO,
    pub router: Arc<Mutex<Router>>,
    pub input_stream: Option<Stream>,
    pub output_stream: Option<Stream>,
    pub sample_rate: u32,
    pub input_sample_rate: u32,
    pub output_buffer_size: Option<u32>,
    pub updates_suspended: Arc<AtomicBool>,
    pub active_stream_id: Arc<AtomicUsize>,
    pub input_buffer: Arc<Mutex<Vec<f32>>>,
    pub input_channels: Arc<AtomicUsize>,
    pub input_users: HashSet<usize>,
    pub current_input_device: Option<String>,
}

impl StreamManager {
    pub fn new(audio_io: AudioIO, router: Arc<Mutex<Router>>, sample_rate: u32) -> Self {
        Self {
            audio_io,
            router,
            input_stream: None,
            output_stream: None,
            sample_rate,
            input_sample_rate: 48000,
            output_buffer_size: None,
            updates_suspended: Arc::new(AtomicBool::new(false)),
            active_stream_id: Arc::new(AtomicUsize::new(0)),
            input_buffer: Arc::new(Mutex::new(Vec::new())),
            input_channels: Arc::new(AtomicUsize::new(0)),
            input_users: HashSet::new(),
            current_input_device: None,
        }
    }

    pub fn list_devices(&self) -> Result<Vec<crate::io::DeviceInfo>> {
        self.audio_io.list_devices()
    }

    pub fn stop(&mut self) -> Result<()> {
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
        
        Ok(())
    }

    pub fn open_audio_input(&mut self, track_id: usize, device_name: Option<String>) -> Result<()> {
        // Add track to users set
        let was_empty = self.input_users.is_empty();
        self.input_users.insert(track_id);
        
        // Implementation would go here (moved from AudioEngine)
        // This is just a skeleton
        Ok(())
    }

    pub fn close_audio_input(&mut self, track_id: usize) -> Result<()> {
        // Remove track from users
        self.input_users.remove(&track_id);
        
        // If no more users, close the input stream
        if self.input_users.is_empty() && self.input_stream.is_some() {
            if let Some(stream) = self.input_stream.take() {
                drop(stream);
            }
            
            // Clear buffer
            if let Ok(mut buffer) = self.input_buffer.lock() {
                buffer.clear();
            }
            
            // Clear current device
            self.current_input_device = None;
        }
        
        Ok(())
    }
}
