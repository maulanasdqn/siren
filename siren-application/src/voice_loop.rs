use siren_domain::{
    CodingAgent, DomainError, MicrophoneSource, SpeechRecognizer, StreamingSpeaker,
    StreamingSynthesizer, VoiceActivityDetector,
};

pub enum VoiceEvent {
    Heard(String),
    Replied(String),
}

pub struct VoiceLoop<'a> {
    microphone: &'a dyn MicrophoneSource,
    vad: &'a mut dyn VoiceActivityDetector,
    recognizer: &'a mut dyn SpeechRecognizer,
    agent: &'a mut dyn CodingAgent,
    synthesizer: &'a mut dyn StreamingSynthesizer,
    speaker: &'a dyn StreamingSpeaker,
}

impl<'a> VoiceLoop<'a> {
    pub fn new(
        microphone: &'a dyn MicrophoneSource,
        vad: &'a mut dyn VoiceActivityDetector,
        recognizer: &'a mut dyn SpeechRecognizer,
        agent: &'a mut dyn CodingAgent,
        synthesizer: &'a mut dyn StreamingSynthesizer,
        speaker: &'a dyn StreamingSpeaker,
    ) -> Self {
        Self { microphone, vad, recognizer, agent, synthesizer, speaker }
    }

    pub fn execute(
        &mut self,
        mut on_event: impl FnMut(VoiceEvent),
    ) -> Result<(), DomainError> {
        for block in self.microphone.open()? {
            for utterance in self.vad.push(block.as_slice()) {
                self.turn(utterance.audio(), &mut on_event)?;
            }
        }
        Ok(())
    }

    fn turn(
        &mut self,
        audio: &siren_domain::AudioSamples,
        on_event: &mut impl FnMut(VoiceEvent),
    ) -> Result<(), DomainError> {
        let transcript = self.recognizer.transcribe(audio)?;
        if transcript.is_empty() {
            return Ok(());
        }
        on_event(VoiceEvent::Heard(transcript.text().to_string()));
        let reply = self.agent.send(transcript.text())?;
        on_event(VoiceEvent::Replied(reply.clone()));
        let chunks = self.synthesizer.stream(&reply)?;
        self.speaker.play_stream(chunks)?;
        self.vad.flush();
        Ok(())
    }
}
