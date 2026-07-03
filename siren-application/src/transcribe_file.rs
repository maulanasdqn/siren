use siren_domain::{AudioDecoder, DomainError, SpeechRecognizer, Transcript};
use std::path::Path;

pub struct TranscribeFile<'a> {
    decoder: &'a dyn AudioDecoder,
    recognizer: &'a mut dyn SpeechRecognizer,
}

impl<'a> TranscribeFile<'a> {
    pub fn new(decoder: &'a dyn AudioDecoder, recognizer: &'a mut dyn SpeechRecognizer) -> Self {
        Self { decoder, recognizer }
    }

    pub fn execute(&mut self, path: &Path) -> Result<Transcript, DomainError> {
        let audio = self.decoder.decode(path)?;
        self.recognizer.transcribe(&audio)
    }
}
