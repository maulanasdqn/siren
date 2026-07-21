use siren_domain::DomainError;

pub const REST_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/models";
pub const WS_ENDPOINT: &str = "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent";

pub const TTS_MODEL: &str = "gemini-3.1-flash-tts-preview";
pub const STT_MODEL: &str = "gemini-flash-latest";
pub const LIVE_MODEL: &str = "gemini-3.1-flash-live-preview";
pub const DEFAULT_VOICE: &str = "Charon";

pub struct Credentials {
    api_key: String,
}

impl Credentials {
    pub fn from_env() -> Result<Self, DomainError> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| DomainError::model("GEMINI_API_KEY environment variable not set"))?;
        Ok(Self { api_key })
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn into_key(self) -> String {
        self.api_key
    }
}

pub fn rest_url(model: &str, method: &str) -> String {
    format!("{REST_ENDPOINT}/{model}:{method}")
}

pub fn ws_url(api_key: &str) -> String {
    format!("{WS_ENDPOINT}?key={api_key}")
}
