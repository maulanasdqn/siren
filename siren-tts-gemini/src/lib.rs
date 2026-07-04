mod stream;

use siren_domain::{DomainError, StreamingSynthesizer, Waveform};
use std::sync::mpsc;

pub const DEFAULT_MODEL: &str = "gemini-3.1-flash-tts-preview";
pub const DEFAULT_VOICE: &str = "Charon";
pub const SAMPLE_RATE: u32 = 24_000;

pub struct GeminiSynthesizer {
    api_key: String,
    model: String,
    voice: String,
}

impl GeminiSynthesizer {
    pub fn load(model: String, voice: String) -> Result<Self, DomainError> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| DomainError::model("GEMINI_API_KEY environment variable not set"))?;
        Ok(Self { api_key, model, voice })
    }
}

impl StreamingSynthesizer for GeminiSynthesizer {
    fn stream(&mut self, text: &str) -> Result<Box<dyn Iterator<Item = Waveform>>, DomainError> {
        let request = stream::Request {
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            voice: self.voice.clone(),
            text: text.to_string(),
        };
        let (tx, rx) = mpsc::channel::<Waveform>();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            if let Err(e) = runtime.block_on(stream::run(request, &tx)) {
                eprintln!("gemini tts error: {e}");
            }
        });
        Ok(Box::new(rx.into_iter()))
    }
}
