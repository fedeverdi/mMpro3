use anyhow::Result;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use std::path::PathBuf;

/// Get recordings directory path (same pattern as scenes)
pub fn get_recordings_dir() -> PathBuf {
    let data_dir = if cfg!(target_os = "macos") {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
    } else if cfg!(target_os = "windows") {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
    };
    
    data_dir.join("mMpro3").join("Recordings")
}

/// Generate a recording file path with timestamp
pub fn generate_recording_path() -> Result<PathBuf> {
    let recordings_dir = get_recordings_dir();
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&recordings_dir)?;
    
    // Generate filename with timestamp
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let filename = format!("Recording_{}.wav", timestamp);
    
    Ok(recordings_dir.join(filename))
}

/// Streaming WAV writer that writes samples incrementally to disk
/// Prevents memory overflow on long recordings by writing directly to file
pub struct StreamingWavWriter {
    file: File,
    bit_depth: u32,
    bytes_per_sample: u16,
    samples_written: u64, // Total samples (L+R) written
    last_sync_samples: u64, // Samples written at last sync (for auto-sync)
    sample_rate: u32, // Sample rate for auto-sync calculation
}

impl StreamingWavWriter {
    /// Create new streaming WAV writer and write initial header
    /// data_size will be updated when finalize() is called
    pub fn new(path: &PathBuf, sample_rate: u32, bit_depth: u32) -> Result<Self> {
        let mut file = File::create(path)?;
        
        let num_channels = 2u16; // Stereo
        let bits_per_sample = bit_depth as u16;
        let bytes_per_sample = bits_per_sample / 8;
        let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
        let block_align = num_channels * bytes_per_sample;
        
        // For 32-bit float, we use format code 3 (IEEE float), otherwise format code 1 (PCM)
        let format_code = if bit_depth == 32 { 3u16 } else { 1u16 };
        
        // Write WAV header with placeholder data_size (0)
        file.write_all(b"RIFF")?;
        file.write_all(&36u32.to_le_bytes())?; // Will be updated in finalize()
        file.write_all(b"WAVE")?;
        
        // fmt chunk
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // chunk size
        file.write_all(&format_code.to_le_bytes())?; // PCM (1) or IEEE Float (3)
        file.write_all(&num_channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&block_align.to_le_bytes())?;
        file.write_all(&bits_per_sample.to_le_bytes())?;
        
        // data chunk
        file.write_all(b"data")?;
        file.write_all(&0u32.to_le_bytes())?; // Will be updated in finalize()
        
        Ok(Self {
            file,
            bit_depth,
            bytes_per_sample,
            samples_written: 0,
            last_sync_samples: 0,
            sample_rate,
        })
    }
    
    /// Write a chunk of samples to the file
    /// samples must be interleaved stereo (L, R, L, R, ...)
    /// Auto-syncs to disk every second to keep file valid in case of crash
    pub fn write_samples(&mut self, samples: &[f32]) -> Result<()> {
        match self.bit_depth {
            16 => self.write_samples_16bit(samples)?,
            24 => self.write_samples_24bit(samples)?,
            32 => self.write_samples_32bit(samples)?,
            _ => return Err(anyhow::anyhow!("Unsupported bit depth: {}", self.bit_depth)),
        }
        self.samples_written += samples.len() as u64;
        
        // Auto-sync every second of audio (sample_rate * 2 samples for stereo)
        let sync_interval = self.sample_rate as u64 * 2; // 2 channels
        if self.samples_written - self.last_sync_samples >= sync_interval {
            self.sync_to_disk()?;
            self.last_sync_samples = self.samples_written;
        }
        
        Ok(())
    }
    
    /// Get the total number of samples written so far
    pub fn get_samples_written(&self) -> u64 {
        self.samples_written
    }
    
    /// Sync data to disk and update header with current size
    /// Makes the file readable even if the app crashes before finalize()
    /// Call this periodically (e.g., every second) during recording
    pub fn sync_to_disk(&mut self) -> Result<()> {
        let data_size = self.samples_written * self.bytes_per_sample as u64;
        let current_pos = self.file.stream_position()?;
        
        // Update RIFF chunk size (at byte 4)
        self.file.seek(SeekFrom::Start(4))?;
        let riff_size = (36 + data_size) as u32;
        self.file.write_all(&riff_size.to_le_bytes())?;
        
        // Update data chunk size (at byte 40)
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write_all(&(data_size as u32).to_le_bytes())?;
        
        // Return to end of file to continue writing
        self.file.seek(SeekFrom::Start(current_pos))?;
        
        // Flush and sync to disk (ensures data is physically written)
        self.file.flush()?;
        self.file.sync_all()?;
        
        Ok(())
    }
    
    /// Finalize the WAV file by updating header with actual data size
    pub fn finalize(mut self) -> Result<()> {
        // Use sync_to_disk to update header one last time
        self.sync_to_disk()?;
        Ok(())
    }
    
    /// Write samples as 16-bit PCM
    fn write_samples_16bit(&mut self, samples: &[f32]) -> Result<()> {
        for sample in samples {
            let s = sample.clamp(-1.0, 1.0);
            let i16_sample = if s < 0.0 {
                (s * 32768.0) as i16
            } else {
                (s * 32767.0) as i16
            };
            self.file.write_all(&i16_sample.to_le_bytes())?;
        }
        Ok(())
    }
    
    /// Write samples as 24-bit PCM (3 bytes per sample)
    fn write_samples_24bit(&mut self, samples: &[f32]) -> Result<()> {
        for sample in samples {
            let s = sample.clamp(-1.0, 1.0);
            let i32_sample = if s < 0.0 {
                (s * 8388608.0) as i32  // 2^23
            } else {
                (s * 8388607.0) as i32
            };
            // Write only the lower 3 bytes (little-endian)
            let bytes = i32_sample.to_le_bytes();
            self.file.write_all(&bytes[0..3])?;
        }
        Ok(())
    }
    
    /// Write samples as 32-bit IEEE float
    fn write_samples_32bit(&mut self, samples: &[f32]) -> Result<()> {
        for sample in samples {
            self.file.write_all(&sample.to_le_bytes())?;
        }
        Ok(())
    }
}
