use crate::audio::{AudioSamples, Utterance};
use crate::error::DomainError;
use crate::transcript::Transcript;
use crate::waveform::Waveform;
use std::path::Path;

pub trait SpeechRecognizer {
    fn transcribe(&mut self, audio: &AudioSamples) -> Result<Transcript, DomainError>;
}

pub trait SpeechSynthesizer {
    fn synthesize(&mut self, text: &str) -> Result<Waveform, DomainError>;
}

pub trait StreamingSynthesizer {
    fn stream(&mut self, text: &str) -> Result<Box<dyn Iterator<Item = Waveform>>, DomainError>;
}

pub trait LiveVoiceAgent {
    fn converse(
        &mut self,
        mic: Box<dyn Iterator<Item = AudioSamples> + Send>,
    ) -> Result<Box<dyn Iterator<Item = Waveform>>, DomainError>;
}

pub trait CodingAgent {
    fn send(&mut self, prompt: &str) -> Result<String, DomainError>;
}

pub trait StreamingSpeaker {
    fn play_stream(&self, chunks: Box<dyn Iterator<Item = Waveform>>) -> Result<(), DomainError>;
}

pub trait WaveformWriter {
    fn write(&self, waveform: &Waveform, path: &Path) -> Result<(), DomainError>;
}

pub trait SpeakerSink {
    fn play(&self, waveform: &Waveform) -> Result<(), DomainError>;
}

pub trait AudioDecoder {
    fn decode(&self, path: &Path) -> Result<AudioSamples, DomainError>;
}

pub trait VoiceActivityDetector {
    fn push(&mut self, samples: &[f32]) -> Vec<Utterance>;
    fn flush(&mut self) -> Option<Utterance>;
}

pub trait MicrophoneSource {
    fn open(&self) -> Result<Box<dyn Iterator<Item = AudioSamples> + Send>, DomainError>;
}
