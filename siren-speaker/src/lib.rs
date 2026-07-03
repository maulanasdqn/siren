use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use siren_domain::{resample_linear, DomainError, SpeakerSink, Waveform};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct CpalSpeaker;

impl SpeakerSink for CpalSpeaker {
    fn play(&self, waveform: &Waveform) -> Result<(), DomainError> {
        play_inner(waveform).map_err(DomainError::audio)
    }
}

fn play_inner(waveform: &Waveform) -> Result<()> {
    let device = cpal::default_host()
        .default_output_device()
        .context("no default output device")?;
    let supported = device.default_output_config()?;

    let out_rate = supported.sample_rate();
    let channels = supported.channels() as usize;
    if supported.sample_format() != cpal::SampleFormat::F32 {
        return Err(anyhow!("unsupported output format {:?}", supported.sample_format()));
    }
    let config: cpal::StreamConfig = supported.into();

    let samples = Arc::new(resample_linear(waveform.samples(), waveform.sample_rate(), out_rate));
    let cursor = Arc::new(AtomicUsize::new(0));

    let (stream_samples, stream_cursor) = (samples.clone(), cursor.clone());
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &_| {
            let mut i = stream_cursor.load(Ordering::Relaxed);
            for frame in data.chunks_mut(channels) {
                let value = stream_samples.get(i).copied().unwrap_or(0.0);
                for slot in frame {
                    *slot = value;
                }
                if i < stream_samples.len() {
                    i += 1;
                }
            }
            stream_cursor.store(i, Ordering::Relaxed);
        },
        |e| eprintln!("output stream error: {e}"),
        None,
    )?;
    stream.play()?;

    let seconds = samples.len() as f32 / out_rate as f32;
    std::thread::sleep(Duration::from_secs_f32(seconds + 0.3));
    Ok(())
}
