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
    pub file_ended: bool, // NEW: Track when file finishes playing
    pub file_name: String, // File name or path
    pub file_artist: Option<String>, // Metadata: artist
    pub file_title: Option<String>, // Metadata: title
    // Resampling state
    pub output_sample_rate: u32,
    pub resample_position: f64,
    // Playback rate control (1.0 = normal speed, 0.0 = stopped, 2.0 = double speed)
    pub playback_rate: f32,
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
            file_ended: false,
            playing: false,
            file_name: String::new(),
            file_artist: None,
            file_title: None,
            output_sample_rate: 44100,
            resample_position: 0.0,
            playback_rate: 1.0, // Default to normal speed
        }
    }

    /// Load audio file using Symphonia (supports MP3, FLAC, WAV, OGG, etc.)
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let file = File::open(&path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint for the format detection
        let mut hint = Hint::new();
        if let Some(ext) = path.as_ref().extension() {
            if let Some(ext_str) = ext.to_str() {
                hint.with_extension(ext_str);
            }
        }

        // Probe the media source
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)?;
        let mut format = probed.format;

        // Get the default track
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("No supported audio track found"))?
;
        let track_id = track.id;
        
        // Get audio specs
        let codec_params = &track.codec_params;
        let declared_channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
        self.sample_rate = codec_params.sample_rate.unwrap_or(44100);

        // Create decoder
        let mut decoder = symphonia::default::get_codecs()
            .make(&codec_params, &decoder_opts)?;

        // Decode all samples
        let mut all_samples = Vec::new();
        let mut actual_channels: Option<u16> = None;
        
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
                    
                    // Capture actual channel count from decoded data (first packet)
                    if actual_channels.is_none() {
                        let decoded_channels = spec.channels.count() as u16;
                        actual_channels = Some(decoded_channels);
                    }
                    
                    all_samples.extend_from_slice(sample_buf.samples());
                }
                Err(e) => {
                    eprintln!("[FilePlayer] Warning: decode error: {}", e);
                    continue;
                }
            }
        }

        self.samples = all_samples;
        self.channels = actual_channels.unwrap_or(declared_channels);
        self.position = 0;
        self.resample_position = 0.0; // CRITICAL: Reset resample position when loading new file
        self.file_ended = false;
        
        // Extract filename from path
        if let Some(filename) = path.as_ref().file_name() {
            if let Some(filename_str) = filename.to_str() {
                self.file_name = filename_str.to_string();
            }
        }
        
        // Calculate peak level in the file
        let peak = self.samples.iter()
            .map(|s| s.abs())
            .fold(0.0_f32, |max, val| max.max(val));
        
        let peak_db = if peak > 0.0 {
            20.0 * peak.log10()
        } else {
            -90.0
        };

        Ok(())
    }

    /// Play the loaded audio
    pub fn play(&mut self) {
        // If the file had ended, restart from the beginning
        if self.file_ended {
            self.position = 0;
            self.resample_position = 0.0;
        }
        self.file_ended = false; // Reset when starting playback
        self.playing = true;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stop and reset to beginning
    pub fn stop(&mut self) {
        self.playing = false;
        self.file_ended = false; // Reset on stop
        self.position = 0;
        self.resample_position = 0.0;
    }

    /// Get next stereo frame (L, R) with resampling
    pub fn next_frame(&mut self) -> (f32, f32) {
        if !self.playing || self.samples.is_empty() {
            return (0.0, 0.0);
        }

        let frames_count = self.samples.len() / self.channels as usize;
        
        // Calculate resampling ratio with playback rate
        // playback_rate: 1.0 = normal, 0.0 = stopped, 2.0 = double speed
        let ratio = (self.sample_rate as f64 / self.output_sample_rate as f64) * self.playback_rate as f64;
        
        // Get interpolated sample
        let source_position = self.resample_position;
        let source_index = source_position.floor() as usize;
        let fraction = (source_position - source_position.floor()) as f32;
        
        if source_index >= frames_count {
            if self.looping {
                // Loop back to start
                self.resample_position = 0.0;
                return self.next_frame();
            } else {
                // File reached the end, signal that it ended naturally
                self.playing = false;
                self.file_ended = true;
                return (0.0, 0.0);
            }
        }
        
        // 4-point Hermite interpolation for better timing precision on transients
        // This significantly improves accuracy on tight beats (1/16 notes, etc.)
        let left = self.interpolate_hermite(source_index, fraction, 0);
        let right = if self.channels > 1 {
            self.interpolate_hermite(source_index, fraction, 1)
        } else {
            left // Mono to stereo
        };

        // Advance position by resampling ratio
        self.resample_position += ratio;

        (left, right)
    }
    
    /// 4-point Hermite cubic interpolation - superior timing precision
    /// channel_offset: 0 for left, 1 for right
    #[inline]
    fn interpolate_hermite(&self, index: usize, frac: f32, channel_offset: usize) -> f32 {
        let frames_count = self.samples.len() / self.channels as usize;
        let ch = self.channels as usize;
        
        // Get 4 points for cubic interpolation (x0, x1, x2, x3)
        // x1 is the current sample, x2 is the next
        let idx0 = if index > 0 { (index - 1) * ch + channel_offset } else { index * ch + channel_offset };
        let idx1 = index * ch + channel_offset;
        let idx2 = ((index + 1).min(frames_count - 1)) * ch + channel_offset;
        let idx3 = ((index + 2).min(frames_count - 1)) * ch + channel_offset;
        
        let x0 = self.samples.get(idx0).copied().unwrap_or(0.0);
        let x1 = self.samples.get(idx1).copied().unwrap_or(0.0);
        let x2 = self.samples.get(idx2).copied().unwrap_or(0.0);
        let x3 = self.samples.get(idx3).copied().unwrap_or(0.0);
        
        // Hermite cubic interpolation formula
        // Provides smooth curve with proper derivatives at boundaries
        let c0 = x1;
        let c1 = 0.5 * (x2 - x0);
        let c2 = x0 - 2.5 * x1 + 2.0 * x2 - 0.5 * x3;
        let c3 = 0.5 * (x3 - x0) + 1.5 * (x1 - x2);
        
        ((c3 * frac + c2) * frac + c1) * frac + c0
    }

    /// Set output sample rate for resampling
    pub fn set_output_sample_rate(&mut self, sample_rate: u32) {
        self.output_sample_rate = sample_rate;
        // Note: resample_position is NOT reset here because it represents
        // the current position in the source file, which is independent
        // of the output sample rate. The resampling ratio adjusts automatically.
    }
    
    /// Seek to a specific time position in seconds
    pub fn seek(&mut self, time_seconds: f32) {
        if self.samples.is_empty() || self.sample_rate == 0 {
            return;
        }
        
        let frames_count = self.samples.len() / self.channels as usize;
        let max_time = frames_count as f32 / self.sample_rate as f32;
        
        // Clamp time to valid range
        let time = time_seconds.max(0.0).min(max_time);
        
        // Calculate frame position
        let frame_position = (time * self.sample_rate as f32) as f64;
        
        // Set resample position
        self.resample_position = frame_position;
        
        // Clear file_ended flag when seeking
        self.file_ended = false;
    }
    
    /// Get current playback position in seconds
    pub fn get_position(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        
        let frames_count = self.samples.len() / self.channels as usize;
        let current_frame = self.resample_position.min(frames_count as f64);
        
        (current_frame / self.sample_rate as f64) as f32
    }
    
    /// Get total duration in seconds
    pub fn get_duration(&self) -> f32 {
        if self.sample_rate == 0 || self.samples.is_empty() {
            return 0.0;
        }
        
        let frames_count = self.samples.len() / self.channels as usize;
        frames_count as f32 / self.sample_rate as f32
    }
    
    /// Set playback rate (1.0 = normal, 0.0 = stopped, 2.0 = double speed)
    /// For vinyl scratch effect: rate can be negative for reverse playback
    pub fn set_playback_rate(&mut self, rate: f32) {
        self.playback_rate = rate;
    }
    
    /// Get decimated waveform data for visualization
    /// Returns mono samples (averaged if stereo) with the specified number of points
    pub fn get_waveform_data(&self, num_points: usize) -> Vec<f32> {
        if self.samples.is_empty() || self.channels == 0 {
            return vec![0.0; num_points];
        }
        
        let frames_count = self.samples.len() / self.channels as usize;
        
        if frames_count == 0 {
            return vec![0.0; num_points];
        }
        
        let mut waveform = Vec::with_capacity(num_points);
        let samples_per_point = frames_count / num_points;
        
        if samples_per_point == 0 {
            // If we have fewer frames than points, just return what we have
            for i in 0..frames_count.min(num_points) {
                let frame_offset = i * self.channels as usize;
                let sample = if self.channels > 1 {
                    // Average stereo to mono
                    let left = self.samples[frame_offset];
                    let right = self.samples[frame_offset + 1];
                    (left + right) / 2.0
                } else {
                    self.samples[frame_offset]
                };
                waveform.push(sample);
            }
            // Pad with zeros if needed
            while waveform.len() < num_points {
                waveform.push(0.0);
            }
        } else {
            // Decimate by finding min/max in each segment
            for i in 0..num_points {
                let start_frame = i * samples_per_point;
                let end_frame = ((i + 1) * samples_per_point).min(frames_count);
                
                let mut min_val = 1.0f32;
                let mut max_val = -1.0f32;
                
                for frame in start_frame..end_frame {
                    let frame_offset = frame * self.channels as usize;
                    let sample = if self.channels > 1 {
                        // Average stereo to mono
                        let left = self.samples.get(frame_offset).copied().unwrap_or(0.0);
                        let right = self.samples.get(frame_offset + 1).copied().unwrap_or(0.0);
                        (left + right) / 2.0
                    } else {
                        self.samples.get(frame_offset).copied().unwrap_or(0.0)
                    };
                    
                    min_val = min_val.min(sample);
                    max_val = max_val.max(sample);
                }
                
                // Use average of min/max for visualization
                waveform.push((min_val + max_val) / 2.0);
            }
        }
        
        waveform
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
