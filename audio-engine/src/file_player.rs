/// Audio file player with support for multiple formats
use anyhow::{anyhow, Result};
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct AudioFilePlayer {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
    pub position: usize,
    pub looping: bool,
    pub playing: bool,
    // Resampling state
    pub output_sample_rate: u32,
    pub resample_position: f64,
}

impl AudioFilePlayer {
    /// Create a new empty player
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            channels: 0,
            sample_rate: 0,
            position: 0,
            looping: false,
            playing: false,
            output_sample_rate: 48000,
            resample_position: 0.0,
        }
    }

    /// Load audio file using Symphonia (supports MP3, FLAC, WAV, OGG, etc.)
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        eprintln!("[FilePlayer] Loading file: {:?}", path.as_ref());
        let file = File::open(&path)?;
        eprintln!("[FilePlayer] File opened successfully");
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint for the format detection
        let mut hint = Hint::new();
        if let Some(ext) = path.as_ref().extension() {
            if let Some(ext_str) = ext.to_str() {
                hint.with_extension(ext_str);
                eprintln!("[FilePlayer] File extension: {}", ext_str);
            }
        }

        // Probe the media source
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        eprintln!("[FilePlayer] Probing format...");
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)?;
        eprintln!("[FilePlayer] Format probed successfully");

        let mut format = probed.format;

        // Get the default track
        eprintln!("[FilePlayer] Looking for audio track...");
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("No supported audio track found"))?;

        let track_id = track.id;
        
        // Get audio specs
        let codec_params = &track.codec_params;
        self.channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
        self.sample_rate = codec_params.sample_rate.unwrap_or(48000);
        eprintln!("[FilePlayer] Track found: {} channels, {} Hz", self.channels, self.sample_rate);

        // Create decoder
        eprintln!("[FilePlayer] Creating decoder...");
        let mut decoder = symphonia::default::get_codecs()
            .make(&codec_params, &decoder_opts)?;
        eprintln!("[FilePlayer] Decoder created");

        // Decode all samples
        let mut all_samples = Vec::new();
        eprintln!("[FilePlayer] Decoding samples...");
        
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(_) => break,
            };

            // Skip packets that don't belong to the selected track
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Create a sample buffer to convert samples to f32
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
                    sample_buf.copy_interleaved_ref(decoded);
                    
                    all_samples.extend_from_slice(sample_buf.samples());
                }
                Err(e) => {
                    eprintln!("[FilePlayer] Warning: decode error: {}", e);
                    continue;
                }
            }
        }

        self.samples = all_samples;
        self.position = 0;
        
        // Calculate peak level in the file
        let peak = self.samples.iter()
            .map(|s| s.abs())
            .fold(0.0_f32, |max, val| max.max(val));
        
        let peak_db = if peak > 0.0 {
            20.0 * peak.log10()
        } else {
            -90.0
        };
        
        eprintln!(
            "[FilePlayer] Loaded: {} samples, {} channels, {} Hz, peak level: {:.2} dB ({:.3} linear)",
            self.samples.len() / self.channels as usize,
            self.channels,
            self.sample_rate,
            peak_db,
            peak
        );

        Ok(())
    }

    /// Play the loaded audio
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stop and reset to beginning
    pub fn stop(&mut self) {
        self.playing = false;
        self.position = 0;
        self.resample_position = 0.0;
    }

    /// Get next stereo frame (L, R) with resampling
    pub fn next_frame(&mut self) -> (f32, f32) {
        if !self.playing || self.samples.is_empty() {
            return (0.0, 0.0);
        }

        let frames_count = self.samples.len() / self.channels as usize;
        
        // Calculate resampling ratio
        let ratio = self.sample_rate as f64 / self.output_sample_rate as f64;
        
        // Get interpolated sample
        let source_position = self.resample_position;
        let source_index = source_position.floor() as usize;
        let fraction = source_position - source_position.floor();
        
        if source_index >= frames_count {
            if self.looping {
                self.resample_position = 0.0;
                return self.next_frame();
            } else {
                self.playing = false;
                return (0.0, 0.0);
            }
        }
        
        // Linear interpolation between samples
        let frame_offset = source_index * self.channels as usize;
        let next_frame_offset = ((source_index + 1).min(frames_count - 1)) * self.channels as usize;
        
        let left1 = self.samples.get(frame_offset).copied().unwrap_or(0.0);
        let left2 = self.samples.get(next_frame_offset).copied().unwrap_or(0.0);
        let left = left1 + (left2 - left1) * fraction as f32;
        
        let right = if self.channels > 1 {
            let right1 = self.samples.get(frame_offset + 1).copied().unwrap_or(0.0);
            let right2 = self.samples.get(next_frame_offset + 1).copied().unwrap_or(0.0);
            right1 + (right2 - right1) * fraction as f32
        } else {
            left // Mono to stereo
        };

        // Advance position by resampling ratio
        self.resample_position += ratio;

        (left, right)
    }

    /// Set output sample rate for resampling
    pub fn set_output_sample_rate(&mut self, sample_rate: u32) {
        if self.output_sample_rate != sample_rate {
            eprintln!("[FilePlayer] Changing output sample rate from {} Hz to {} Hz, resetting resample position",
                self.output_sample_rate, sample_rate);
            self.output_sample_rate = sample_rate;
            // Reset resample position to avoid audio glitches when sample rate changes
            // This prevents the accumulated position from causing pitch/speed issues
            self.resample_position = self.position as f64;
        }
    }
}

impl Default for AudioFilePlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_player() {
        let mut player = AudioFilePlayer::new();
        player.play();
        let (l, r) = player.next_frame();
        assert_eq!(l, 0.0);
        assert_eq!(r, 0.0);
    }
}
