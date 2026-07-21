use anyhow::Result;
use siren_application::{TranscribeFile, TranscribeStream};
use siren_asr::WhisperRecognizer;
use siren_asr_gemini::GeminiRecognizer;
use siren_audio::SymphoniaDecoder;
use siren_domain::{Language, ModelId, SpeechRecognizer};
use siren_mic::CpalMicrophone;
use siren_vad::EnergyVad;
use std::path::PathBuf;

pub fn transcribe(local: bool, model: &str, language: &str, file: &PathBuf) -> Result<()> {
    let mut recognizer = load_recognizer(local, model, language)?;
    let decoder = SymphoniaDecoder;
    let transcript = TranscribeFile::new(&decoder, recognizer.as_mut()).execute(file)?;
    println!("{}", transcript.text());
    Ok(())
}

pub fn listen(local: bool, model: &str, language: &str) -> Result<()> {
    let mut recognizer = load_recognizer(local, model, language)?;
    let microphone = CpalMicrophone;
    let mut vad = EnergyVad::new();
    eprintln!("listening — speak; Ctrl-C to stop");
    TranscribeStream::new(&microphone, &mut vad, recognizer.as_mut())
        .execute(|transcript| println!("{}", transcript.text()))?;
    Ok(())
}

fn load_recognizer(
    local: bool,
    model: &str,
    language: &str,
) -> Result<Box<dyn SpeechRecognizer>> {
    if local {
        let recognizer = WhisperRecognizer::load(&ModelId::new(model), &Language::new(language))?;
        eprintln!("engine: local Whisper ({})", recognizer.backend());
        Ok(Box::new(recognizer))
    } else {
        let recognizer =
            GeminiRecognizer::load(siren_gemini::STT_MODEL.to_string(), language.to_string())?;
        eprintln!("engine: Gemini {}", siren_gemini::STT_MODEL);
        Ok(Box::new(recognizer))
    }
}
