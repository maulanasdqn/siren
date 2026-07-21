use crate::Config;
use anyhow::{Context, Result};
use base64::Engine;
use siren_domain::Waveform;

const OUTPUT_RATE: u32 = 24_000;
const INPUT_MIME: &str = "audio/pcm;rate=16000";

pub fn setup(config: &Config) -> String {
    let mut setup = serde_json::json!({
        "model": format!("models/{}", config.model),
        "generationConfig": {
            "responseModalities": ["AUDIO"],
            "speechConfig": {
                "voiceConfig": { "prebuiltVoiceConfig": { "voiceName": config.voice } }
            }
        }
    });
    if !config.system.is_empty() {
        setup["systemInstruction"] =
            serde_json::json!({ "parts": [{ "text": config.system }] });
    }
    serde_json::json!({ "setup": setup }).to_string()
}

pub fn realtime_input(pcm: &[u8]) -> String {
    let data = base64::engine::general_purpose::STANDARD.encode(pcm);
    serde_json::json!({
        "realtimeInput": { "audio": { "data": data, "mimeType": INPUT_MIME } }
    })
    .to_string()
}

pub fn pcm_bytes(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let value = (clamped * i16::MAX as f32) as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

pub fn parse(payload: &[u8]) -> Result<Vec<Waveform>> {
    if payload.is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value =
        serde_json::from_slice(payload).context("parsing live message")?;
    let Some(parts) = value["serverContent"]["modelTurn"]["parts"].as_array() else {
        return Ok(Vec::new());
    };
    let mut waves = Vec::new();
    for part in parts {
        if let Some(data) = part["inlineData"]["data"].as_str() {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .context("decoding live audio")?;
            let samples = bytes
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
                .collect();
            waves.push(Waveform::new(samples, OUTPUT_RATE));
        }
    }
    Ok(waves)
}
