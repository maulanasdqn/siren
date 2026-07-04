use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use siren_domain::{resample_linear, DomainError, StreamingSpeaker, Waveform};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct CpalStreamingSpeaker;

impl StreamingSpeaker for CpalStreamingSpeaker {
    fn play_stream(&self, chunks: Box<dyn Iterator<Item = Waveform>>) -> Result<(), DomainError> {
        play_inner(chunks).map_err(DomainError::audio)
    }
}

fn play_inner(chunks: Box<dyn Iterator<Item = Waveform>>) -> Result<()> {
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

    let queue: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));
    let callback_queue = queue.clone();
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &_| {
            let mut buffer = callback_queue.lock().unwrap();
            for frame in data.chunks_mut(channels) {
                let sample = buffer.pop_front().unwrap_or(0.0);
                for slot in frame {
                    *slot = sample;
                }
            }
        },
        |e| eprintln!("output stream error: {e}"),
        None,
    )?;
    stream.play()?;

    for chunk in chunks {
        if chunk.is_empty() {
            continue;
        }
        let resampled = resample_linear(chunk.samples(), chunk.sample_rate(), out_rate);
        queue.lock().unwrap().extend(resampled);
    }

    loop {
        std::thread::sleep(Duration::from_millis(30));
        if queue.lock().unwrap().is_empty() {
            break;
        }
    }
    std::thread::sleep(Duration::from_millis(150));
    Ok(())
}
