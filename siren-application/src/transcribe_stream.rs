use siren_domain::{
    DomainError, MicrophoneSource, SpeechRecognizer, Transcript, VoiceActivityDetector,
};

pub struct TranscribeStream<'a> {
    microphone: &'a dyn MicrophoneSource,
    vad: &'a mut dyn VoiceActivityDetector,
    recognizer: &'a mut dyn SpeechRecognizer,
}

impl<'a> TranscribeStream<'a> {
    pub fn new(
        microphone: &'a dyn MicrophoneSource,
        vad: &'a mut dyn VoiceActivityDetector,
        recognizer: &'a mut dyn SpeechRecognizer,
    ) -> Self {
        Self { microphone, vad, recognizer }
    }

    pub fn execute(
        &mut self,
        mut on_transcript: impl FnMut(Transcript),
    ) -> Result<(), DomainError> {
        for block in self.microphone.open()? {
            for utterance in self.vad.push(block.as_slice()) {
                let transcript = self.recognizer.transcribe(utterance.audio())?;
                if !transcript.is_empty() {
                    on_transcript(transcript);
                }
            }
        }
        if let Some(utterance) = self.vad.flush() {
            let transcript = self.recognizer.transcribe(utterance.audio())?;
            if !transcript.is_empty() {
                on_transcript(transcript);
            }
        }
        Ok(())
    }
}
