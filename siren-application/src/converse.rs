use siren_domain::{AudioSamples, DomainError, LiveVoiceAgent, StreamingSpeaker};

pub struct Converse<'a> {
    agent: &'a mut dyn LiveVoiceAgent,
}

impl<'a> Converse<'a> {
    pub fn new(agent: &'a mut dyn LiveVoiceAgent) -> Self {
        Self { agent }
    }

    pub fn execute(
        &mut self,
        mic: Box<dyn Iterator<Item = AudioSamples> + Send>,
        speaker: &dyn StreamingSpeaker,
    ) -> Result<(), DomainError> {
        let chunks = self.agent.converse(mic)?;
        speaker.play_stream(chunks)
    }
}
