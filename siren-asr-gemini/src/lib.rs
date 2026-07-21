use anyhow::{anyhow, Context, Result};
use base64::Engine;
use siren_audio::wav_bytes;
use siren_domain::{AudioSamples, DomainError, SpeechRecognizer, Transcript};
use siren_gemini::{rest_url, Credentials};

pub struct GeminiRecognizer {
    api_key: String,
    model: String,
    language: String,
    client: reqwest::blocking::Client,
}

impl GeminiRecognizer {
    pub fn load(model: String, language: String) -> Result<Self, DomainError> {
        let api_key = Credentials::from_env()?.into_key();
        let client = reqwest::blocking::Client::new();
        Ok(Self { api_key, model, language, client })
    }
}

impl SpeechRecognizer for GeminiRecognizer {
    fn transcribe(&mut self, audio: &AudioSamples) -> Result<Transcript, DomainError> {
        transcribe_inner(self, audio).map_err(DomainError::inference)
    }
}

fn transcribe_inner(recognizer: &GeminiRecognizer, audio: &AudioSamples) -> Result<Transcript> {
    let wav = wav_bytes(audio.as_slice(), AudioSamples::SAMPLE_RATE);
    let encoded = base64::engine::general_purpose::STANDARD.encode(&wav);
    let prompt = format!(
        "Transcribe this audio. The spoken language is '{}'. \
         Return only the transcript text, with no commentary or labels.",
        recognizer.language
    );
    let body = serde_json::json!({
        "contents": [{
            "parts": [
                { "text": prompt },
                { "inlineData": { "mimeType": "audio/wav", "data": encoded } }
            ]
        }]
    });

    let response = recognizer
        .client
        .post(rest_url(&recognizer.model, "generateContent"))
        .header("x-goog-api-key", &recognizer.api_key)
        .json(&body)
        .send()
        .context("sending request to gemini")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        return Err(anyhow!("gemini http {status}: {text}"));
    }

    let value: serde_json::Value = response.json().context("parsing gemini response")?;
    Ok(Transcript::new(collect_text(&value)))
}

fn collect_text(value: &serde_json::Value) -> String {
    let Some(parts) = value["candidates"][0]["content"]["parts"].as_array() else {
        return String::new();
    };
    parts
        .iter()
        .filter_map(|part| part["text"].as_str())
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}
