use anyhow::{anyhow, Context, Result};
use hf_hub::api::sync::Api;
use piper_rs::Piper;
use siren_domain::{DomainError, SpeechSynthesizer, Waveform};
use std::path::PathBuf;

const VOICE_REPO: &str = "rhasspy/piper-voices";
const VOICE_ONNX: &str = "id/id_ID/news_tts/medium/id_ID-news_tts-medium.onnx";
const VOICE_CONFIG: &str = "id/id_ID/news_tts/medium/id_ID-news_tts-medium.onnx.json";

pub struct Prosody {
    pub length_scale: f32,
    pub noise_scale: f32,
    pub noise_w: f32,
}

impl Default for Prosody {
    fn default() -> Self {
        Self { length_scale: 1.15, noise_scale: 0.8, noise_w: 1.0 }
    }
}

pub struct PiperSynthesizer {
    piper: Piper,
    prosody: Prosody,
}

impl PiperSynthesizer {
    pub fn load(prosody: Prosody) -> Result<Self, DomainError> {
        load_inner(prosody).map_err(DomainError::model)
    }
}

impl SpeechSynthesizer for PiperSynthesizer {
    fn synthesize(&mut self, text: &str) -> Result<Waveform, DomainError> {
        let text = punctuate(text);
        let (samples, sample_rate) = self
            .piper
            .create(
                &text,
                false,
                None,
                Some(self.prosody.length_scale),
                Some(self.prosody.noise_scale),
                Some(self.prosody.noise_w),
            )
            .map_err(|e| DomainError::inference(format!("{e:?}")))?;
        Ok(Waveform::new(samples, sample_rate))
    }
}

fn punctuate(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.ends_with(['.', '!', '?', ',', ';', ':']) {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

fn load_inner(prosody: Prosody) -> Result<PiperSynthesizer> {
    let repo = Api::new()?.model(VOICE_REPO.to_string());
    let onnx = repo.get(VOICE_ONNX).context("fetching piper voice model")?;
    let config = repo.get(VOICE_CONFIG).context("fetching piper voice config")?;
    let config = patch_config(&config)?;
    let piper = Piper::new(&onnx, &config).map_err(|e| anyhow!("{e:?}"))?;
    Ok(PiperSynthesizer { piper, prosody })
}

fn patch_config(path: &std::path::Path) -> Result<PathBuf> {
    let mut value: serde_json::Value = serde_json::from_reader(std::fs::File::open(path)?)?;
    if value.get("speaker_id_map").is_none() {
        value["speaker_id_map"] = serde_json::json!({});
    }
    let patched = std::env::temp_dir().join("siren-piper-id.onnx.json");
    std::fs::write(&patched, serde_json::to_string(&value)?)?;
    Ok(patched)
}
