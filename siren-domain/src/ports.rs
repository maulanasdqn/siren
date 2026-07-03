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
    fn open(&self) -> Result<Box<dyn Iterator<Item = AudioSamples>>, DomainError>;
}
