use anyhow::{anyhow, Context, Result};
use siren_domain::{downmix_to_mono, resample_linear, AudioDecoder, AudioSamples, DomainError};
use std::path::Path;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

pub struct SymphoniaDecoder;

impl AudioDecoder for SymphoniaDecoder {
    fn decode(&self, path: &Path) -> Result<AudioSamples, DomainError> {
        decode_inner(path).map_err(DomainError::audio)
    }
}

fn decode_inner(path: &Path) -> Result<AudioSamples> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("opening {}", path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let mut format = symphonia::default::get_probe().probe(
        &hint,
        mss,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;

    let track = format
        .default_track(TrackType::Audio)
        .context("no default audio track")?
        .clone();
    let track_id = track.id;
    let params = track
        .codec_params
        .as_ref()
        .and_then(|c| c.audio())
        .ok_or_else(|| anyhow!("track has no audio codec parameters"))?;

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(params, &AudioDecoderOptions::default())?;

    let mut src_rate = params.sample_rate.unwrap_or(AudioSamples::SAMPLE_RATE);
    let mut channels = 1usize;
    let mut interleaved: Vec<f32> = Vec::new();
    let mut frame: Vec<f32> = Vec::new();

    while let Some(packet) = format.next_packet()? {
        if packet.track_id != track_id {
            continue;
        }
        let decoded = decoder.decode(&packet)?;
        let spec = decoded.spec();
        channels = spec.channels().count().max(1);
        src_rate = spec.rate();
        decoded.copy_to_vec_interleaved(&mut frame);
        interleaved.extend_from_slice(&frame);
    }

    let mono = downmix_to_mono(&interleaved, channels);
    let resampled = resample_linear(&mono, src_rate, AudioSamples::SAMPLE_RATE);
    Ok(AudioSamples::new(resampled))
}
