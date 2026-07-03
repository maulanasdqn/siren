use anyhow::Result;
use siren_domain::{DomainError, Waveform, WaveformWriter};
use std::io::Write;
use std::path::Path;

pub struct WavWriter;

impl WaveformWriter for WavWriter {
    fn write(&self, waveform: &Waveform, path: &Path) -> Result<(), DomainError> {
        write_inner(waveform, path).map_err(DomainError::io)
    }
}

fn write_inner(waveform: &Waveform, path: &Path) -> Result<()> {
    let samples = waveform.samples();
    let sample_rate = waveform.sample_rate();
    let data_len = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * 2;

    let mut out = std::fs::File::create(path)?;
    out.write_all(b"RIFF")?;
    out.write_all(&(36 + data_len).to_le_bytes())?;
    out.write_all(b"WAVE")?;
    out.write_all(b"fmt ")?;
    out.write_all(&16u32.to_le_bytes())?;
    out.write_all(&1u16.to_le_bytes())?;
    out.write_all(&1u16.to_le_bytes())?;
    out.write_all(&sample_rate.to_le_bytes())?;
    out.write_all(&byte_rate.to_le_bytes())?;
    out.write_all(&2u16.to_le_bytes())?;
    out.write_all(&16u16.to_le_bytes())?;
    out.write_all(b"data")?;
    out.write_all(&data_len.to_le_bytes())?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let value = (clamped * i16::MAX as f32) as i16;
        out.write_all(&value.to_le_bytes())?;
    }
    Ok(())
}
