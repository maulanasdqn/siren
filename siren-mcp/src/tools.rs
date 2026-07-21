use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use siren_application::{StreamSpeech, SynthesizeText, TranscribeFile};
use siren_asr::WhisperRecognizer;
use siren_asr_gemini::GeminiRecognizer;
use siren_audio::{SymphoniaDecoder, WavWriter};
use siren_domain::{Language, ModelId, SpeakerSink, SpeechRecognizer, WaveformWriter};
use siren_speaker::{CpalSpeaker, CpalStreamingSpeaker};
use siren_tts::{PiperSynthesizer, Prosody};
use siren_tts_gemini::GeminiSynthesizer;
use std::path::PathBuf;

const WHISPER_MODEL: &str = "cahya/whisper-tiny-id";
const VOICES: [&str; 7] =
    ["Charon", "Orus", "Fenrir", "Iapetus", "Enceladus", "Puck", "Aoede"];

pub fn list() -> Value {
    json!([
        {
            "name": "siren_speak",
            "description": "Speak text aloud with a natural voice (Google Gemini by default). Plays through the machine's speaker, or writes a WAV when 'output' is set.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to speak." },
                    "voice": { "type": "string", "description": "Gemini voice name (default Charon)." },
                    "output": { "type": "string", "description": "Optional WAV path; saves instead of playing live." },
                    "local": { "type": "boolean", "description": "Use the offline Piper voice (no API key or network)." }
                },
                "required": ["text"]
            }
        },
        {
            "name": "siren_transcribe",
            "description": "Transcribe an audio file to text (Google Gemini by default; wav/mp3/flac/ogg/etc).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the audio file." },
                    "language": { "type": "string", "description": "Spoken-language hint (default id)." },
                    "local": { "type": "boolean", "description": "Use offline Whisper instead of Gemini." }
                },
                "required": ["path"]
            }
        },
        {
            "name": "siren_list_voices",
            "description": "List the available Gemini voice names for siren_speak.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

pub fn call(name: &str, args: &Value) -> Result<String> {
    match name {
        "siren_speak" => speak(args),
        "siren_transcribe" => transcribe(args),
        "siren_list_voices" => Ok(VOICES.join(", ")),
        _ => Err(anyhow!("unknown tool: {name}")),
    }
}

fn speak(args: &Value) -> Result<String> {
    let text = args["text"].as_str().ok_or_else(|| anyhow!("missing 'text'"))?;
    let output = args["output"].as_str().map(PathBuf::from);
    if args["local"].as_bool().unwrap_or(false) {
        let mut synth = PiperSynthesizer::load(Prosody::default())?;
        let wave = SynthesizeText::new(&mut synth).execute(text)?;
        return match output {
            Some(path) => {
                WavWriter.write(&wave, &path)?;
                Ok(format!("wrote {} ({:.1}s, local Piper)", path.display(), wave.duration_secs()))
            }
            None => {
                CpalSpeaker.play(&wave)?;
                Ok(format!("spoke {:.1}s with the local Piper voice", wave.duration_secs()))
            }
        };
    }
    let voice = args["voice"].as_str().unwrap_or(siren_gemini::DEFAULT_VOICE).to_string();
    let mut synth = GeminiSynthesizer::load(siren_gemini::TTS_MODEL.to_string(), voice.clone())?;
    match output {
        Some(path) => {
            let wave = SynthesizeText::new(&mut synth).execute(text)?;
            WavWriter.write(&wave, &path)?;
            Ok(format!("wrote {} ({:.1}s, Gemini {voice})", path.display(), wave.duration_secs()))
        }
        None => {
            StreamSpeech::new(&mut synth).execute(text, &CpalStreamingSpeaker)?;
            Ok(format!("spoke with the Gemini voice {voice}"))
        }
    }
}

fn transcribe(args: &Value) -> Result<String> {
    let path = args["path"].as_str().ok_or_else(|| anyhow!("missing 'path'"))?;
    let language = args["language"].as_str().unwrap_or("id").to_string();
    let decoder = SymphoniaDecoder;
    let mut recognizer: Box<dyn SpeechRecognizer> = if args["local"].as_bool().unwrap_or(false) {
        Box::new(WhisperRecognizer::load(&ModelId::new(WHISPER_MODEL), &Language::new(&language))?)
    } else {
        Box::new(GeminiRecognizer::load(siren_gemini::STT_MODEL.to_string(), language)?)
    };
    let transcript = TranscribeFile::new(&decoder, recognizer.as_mut()).execute(&PathBuf::from(path))?;
    Ok(transcript.text().to_string())
}
