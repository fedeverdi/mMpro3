use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Write audio samples to WAV file
/// Supports 16-bit, 24-bit, and 32-bit float formats
pub fn write_wav_file(
    path: &PathBuf,
    samples: &[f32],
    sample_rate: u32,
    bit_depth: u32,
) -> Result<()> {
    let mut file = File::create(path)?;
    
    let num_samples = samples.len();
    let num_channels = 2u16; // Stereo
    let bits_per_sample = bit_depth as u16;
    let bytes_per_sample = bits_per_sample / 8;
    let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
    let block_align = num_channels * bytes_per_sample;
    let data_size = num_samples as u32 * bytes_per_sample as u32;
    
    // For 32-bit float, we use format code 3 (IEEE float), otherwise format code 1 (PCM)
    let format_code = if bit_depth == 32 { 3u16 } else { 1u16 };
    
    // Write WAV header
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_size).to_le_bytes())?;
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
    file.write_all(&data_size.to_le_bytes())?;
    
    // Write samples based on bit depth
    match bit_depth {
        16 => write_samples_16bit(&mut file, samples)?,
        24 => write_samples_24bit(&mut file, samples)?,
        32 => write_samples_32bit(&mut file, samples)?,
        _ => return Err(anyhow::anyhow!("Unsupported bit depth: {}", bit_depth)),
    }
    
    Ok(())
}

/// Write samples as 16-bit PCM
fn write_samples_16bit(file: &mut File, samples: &[f32]) -> Result<()> {
    for sample in samples {
        let s = sample.clamp(-1.0, 1.0);
        let i16_sample = if s < 0.0 {
            (s * 32768.0) as i16
        } else {
            (s * 32767.0) as i16
        };
        file.write_all(&i16_sample.to_le_bytes())?;
    }
    Ok(())
}

/// Write samples as 24-bit PCM (3 bytes per sample)
fn write_samples_24bit(file: &mut File, samples: &[f32]) -> Result<()> {
    for sample in samples {
        let s = sample.clamp(-1.0, 1.0);
        let i32_sample = if s < 0.0 {
            (s * 8388608.0) as i32  // 2^23
        } else {
            (s * 8388607.0) as i32
        };
        // Write only the lower 3 bytes (little-endian)
        let bytes = i32_sample.to_le_bytes();
        file.write_all(&bytes[0..3])?;
    }
    Ok(())
}

/// Write samples as 32-bit IEEE float
fn write_samples_32bit(file: &mut File, samples: &[f32]) -> Result<()> {
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}
