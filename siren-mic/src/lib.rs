use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use siren_domain::{downmix_to_mono, resample_linear, AudioSamples, DomainError, MicrophoneSource};
use std::sync::mpsc::{self, Sender};

pub struct CpalMicrophone;

impl MicrophoneSource for CpalMicrophone {
    fn open(&self) -> Result<Box<dyn Iterator<Item = AudioSamples> + Send>, DomainError> {
        open_inner().map_err(DomainError::audio)
    }
}

fn open_inner() -> Result<Box<dyn Iterator<Item = AudioSamples> + Send>> {
    let (audio_tx, audio_rx) = mpsc::channel::<AudioSamples>();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();

    std::thread::spawn(move || match run(audio_tx) {
        Ok(stream) => {
            let _ = ready_tx.send(Ok(()));
            let _keep = stream;
            loop {
                std::thread::park();
            }
        }
        Err(e) => {
            let _ = ready_tx.send(Err(anyhow!("{e}")));
        }
    });

    ready_rx.recv().context("microphone thread exited")??;
    Ok(Box::new(audio_rx.into_iter()))
}

fn run(audio_tx: Sender<AudioSamples>) -> Result<Stream> {
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

    let stream = device.build_input_stream(
        config,
        move |data: &[f32], _: &_| {
            let mono = downmix_to_mono(data, channels);
            let resampled = resample_linear(&mono, src_rate, AudioSamples::SAMPLE_RATE);
            let _ = audio_tx.send(AudioSamples::new(resampled));
        },
        |e| eprintln!("audio stream error: {e}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}
