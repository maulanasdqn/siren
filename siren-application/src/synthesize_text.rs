use siren_domain::{DomainError, SpeechSynthesizer, Waveform};

pub struct SynthesizeText<'a> {
    synthesizer: &'a mut dyn SpeechSynthesizer,
}

impl<'a> SynthesizeText<'a> {
    pub fn new(synthesizer: &'a mut dyn SpeechSynthesizer) -> Self {
        Self { synthesizer }
    }

    pub fn execute(&mut self, text: &str) -> Result<Waveform, DomainError> {
        self.synthesizer.synthesize(text)
    }
}
