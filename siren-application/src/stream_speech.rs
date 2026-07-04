use siren_domain::{DomainError, StreamingSpeaker, StreamingSynthesizer};

pub struct StreamSpeech<'a> {
    synthesizer: &'a mut dyn StreamingSynthesizer,
}

impl<'a> StreamSpeech<'a> {
    pub fn new(synthesizer: &'a mut dyn StreamingSynthesizer) -> Self {
        Self { synthesizer }
    }

    pub fn execute(&mut self, text: &str, sink: &dyn StreamingSpeaker) -> Result<(), DomainError> {
        let chunks = self.synthesizer.stream(text)?;
        sink.play_stream(chunks)
    }
}
