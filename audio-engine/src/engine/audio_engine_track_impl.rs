// Implementation of TrackControl trait for AudioEngine

use super::AudioEngine;
use super::track_control::TrackControl;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use crate::processing::Router;

impl TrackControl for AudioEngine {
    fn get_router(&self) -> &Arc<Mutex<Router>> {
        &self.router
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn open_audio_input(&mut self, track_id: usize, device_name: Option<String>) -> Result<()> {
        self.open_audio_input(track_id, device_name)
    }

    fn close_audio_input(&mut self, track_id: usize) -> Result<()> {
        self.close_audio_input(track_id)
    }
}
