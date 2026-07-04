use crate::SAMPLE_RATE;
use anyhow::{anyhow, Context, Result};
use base64::Engine;
use futures_util::StreamExt;
use siren_domain::Waveform;
use std::sync::mpsc::Sender;

const ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct Request {
    pub api_key: String,
    pub model: String,
    pub voice: String,
    pub text: String,
}

pub async fn run(request: Request, tx: &Sender<Waveform>) -> Result<()> {
    let url = format!("{ENDPOINT}/{}:streamGenerateContent?alt=sse", request.model);
    let body = serde_json::json!({
        "contents": [{ "parts": [{ "text": request.text }] }],
        "generationConfig": {
            "responseModalities": ["AUDIO"],
            "speechConfig": {
                "voiceConfig": { "prebuiltVoiceConfig": { "voiceName": request.voice } }
            }
        }
    });

    let response = reqwest::Client::new()
        .post(url)
        .header("x-goog-api-key", &request.api_key)
        .json(&body)
        .send()
        .await
        .context("sending request to gemini")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow!("gemini http {status}: {text}"));
    }

    let mut body = response.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();
    while let Some(chunk) = body.next().await {
        buffer.extend_from_slice(&chunk.context("reading stream")?);
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            if let Some(waveform) = parse_line(&line)? {
                if tx.send(waveform).is_err() {
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

fn parse_line(line: &[u8]) -> Result<Option<Waveform>> {
    let line = String::from_utf8_lossy(line);
    let Some(json) = line.trim().strip_prefix("data:") else {
        return Ok(None);
    };
    let json = json.trim();
    if json.is_empty() || json == "[DONE]" {
        return Ok(None);
    }

    let value: serde_json::Value = serde_json::from_str(json).context("parsing sse json")?;
    let data = value["candidates"][0]["content"]["parts"][0]["inlineData"]["data"].as_str();
    let Some(data) = data else {
        return Ok(None);
    };

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .context("decoding audio base64")?;
    let samples: Vec<f32> = bytes
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
        .collect();
    Ok(Some(Waveform::new(samples, SAMPLE_RATE)))
}
