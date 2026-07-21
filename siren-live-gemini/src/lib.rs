mod protocol;
mod session;

use siren_domain::{AudioSamples, DomainError, LiveVoiceAgent, Waveform};
use siren_gemini::Credentials;
use std::sync::mpsc;

pub(crate) struct Config {
    pub(crate) api_key: String,
    pub(crate) model: String,
    pub(crate) voice: String,
    pub(crate) system: String,
}

pub struct GeminiLiveAgent {
    api_key: String,
    model: String,
    voice: String,
    system: String,
}

impl GeminiLiveAgent {
    pub fn load(model: String, voice: String, system: String) -> Result<Self, DomainError> {
        let api_key = Credentials::from_env()?.into_key();
        Ok(Self { api_key, model, voice, system })
    }
}

impl LiveVoiceAgent for GeminiLiveAgent {
    fn converse(
        &mut self,
        mic: Box<dyn Iterator<Item = AudioSamples> + Send>,
    ) -> Result<Box<dyn Iterator<Item = Waveform>>, DomainError> {
        let (out_tx, out_rx) = mpsc::channel::<Waveform>();
        let config = Config {
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            voice: self.voice.clone(),
            system: self.system.clone(),
        };
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            if let Err(e) = runtime.block_on(session::run(config, mic, out_tx)) {
                eprintln!("gemini live error: {e}");
            }
        });
        Ok(Box::new(out_rx.into_iter()))
    }
}
