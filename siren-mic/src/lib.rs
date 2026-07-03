use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use siren_domain::{downmix_to_mono, resample_linear, AudioSamples, DomainError, MicrophoneSource};
use std::sync::mpsc::{self, Receiver};

pub struct CpalMicrophone;

impl MicrophoneSource for CpalMicrophone {
    fn open(&self) -> Result<Box<dyn Iterator<Item = AudioSamples>>, DomainError> {
        open_inner().map_err(DomainError::audio)
    }
}

fn open_inner() -> Result<Box<dyn Iterator<Item = AudioSamples>>> {
    let device = cpal::default_host()
        .default_input_device()
        .context("no default input device")?;
    let supported = device.default_input_config()?;

    let src_rate = supported.sample_rate();
    let channels = supported.channels() as usize;
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();

    if sample_format != cpal::SampleFormat::F32 {
        return Err(anyhow!("unsupported sample format {sample_format:?}"));
    }

    let (tx, rx) = mpsc::channel::<Vec<f32>>();
    let stream = device.build_input_stream(
        config,
        move |data: &[f32], _: &_| {
            let _ = tx.send(data.to_vec());
        },
        |e| eprintln!("audio stream error: {e}"),
        None,
    )?;
    stream.play()?;

    Ok(Box::new(MicStream { _stream: stream, rx, channels, src_rate }))
}

struct MicStream {
    _stream: Stream,
    rx: Receiver<Vec<f32>>,
    channels: usize,
    src_rate: u32,
}

impl Iterator for MicStream {
    type Item = AudioSamples;

    fn next(&mut self) -> Option<Self::Item> {
        let block = self.rx.recv().ok()?;
        let mono = downmix_to_mono(&block, self.channels);
        let resampled = resample_linear(&mono, self.src_rate, AudioSamples::SAMPLE_RATE);
        Some(AudioSamples::new(resampled))
    }
}
