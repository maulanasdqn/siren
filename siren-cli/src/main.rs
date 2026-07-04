use anyhow::Result;
use clap::{Parser, Subcommand};
use siren_application::{StreamSpeech, SynthesizeText, TranscribeFile, TranscribeStream};
use siren_asr::WhisperRecognizer;
use siren_audio::{SymphoniaDecoder, WavWriter};
use siren_domain::{Language, ModelId, SpeakerSink, WaveformWriter};
use siren_mic::CpalMicrophone;
use siren_speaker::{CpalSpeaker, CpalStreamingSpeaker};
use siren_tts::{PiperSynthesizer, Prosody};
use siren_tts_gemini::GeminiSynthesizer;
use siren_vad::EnergyVad;
use std::path::PathBuf;

const DEFAULT_MODEL: &str = "cahya/whisper-tiny-id";
const DEFAULT_LANGUAGE: &str = "id";

#[derive(Parser)]
#[command(name = "siren", about = "Local-first speech-to-text and text-to-speech")]
struct Cli {
    #[arg(long, default_value = DEFAULT_MODEL, global = true)]
    model: String,
    #[arg(long, default_value = DEFAULT_LANGUAGE, global = true)]
    language: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Transcribe {
        file: PathBuf,
    },
    Listen,
    Speak {
        text: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        play: bool,
        #[arg(long, default_value_t = 1.15)]
        length_scale: f32,
        #[arg(long, default_value_t = 0.8)]
        noise_scale: f32,
        #[arg(long, default_value_t = 1.0)]
        noise_w: f32,
    },
    Say {
        text: String,
        #[arg(long, default_value = siren_tts_gemini::DEFAULT_VOICE)]
        voice: String,
        #[arg(long, default_value = siren_tts_gemini::DEFAULT_MODEL)]
        model: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Transcribe { file } => transcribe(&cli.model, &cli.language, &file),
        Command::Listen => listen(&cli.model, &cli.language),
        Command::Speak { text, output, play, length_scale, noise_scale, noise_w } => {
            let prosody = Prosody { length_scale, noise_scale, noise_w };
            speak(&text, output, play, prosody)
        }
        Command::Say { text, voice, model } => say(&text, voice, model),
    }
}

fn say(text: &str, voice: String, model: String) -> Result<()> {
    let mut synthesizer = GeminiSynthesizer::load(model, voice.clone())?;
    eprintln!("voice: Gemini {voice} (live streaming)");
    StreamSpeech::new(&mut synthesizer).execute(text, &CpalStreamingSpeaker)?;
    Ok(())
}

fn transcribe(model: &str, language: &str, file: &PathBuf) -> Result<()> {
    let mut recognizer = load_recognizer(model, language)?;
    let decoder = SymphoniaDecoder;
    let transcript = TranscribeFile::new(&decoder, &mut recognizer).execute(file)?;
    println!("{}", transcript.text());
    Ok(())
}

fn listen(model: &str, language: &str) -> Result<()> {
    let mut recognizer = load_recognizer(model, language)?;
    let microphone = CpalMicrophone;
    let mut vad = EnergyVad::new();
    eprintln!("listening — speak Indonesian; Ctrl-C to stop");
    TranscribeStream::new(&microphone, &mut vad, &mut recognizer)
        .execute(|transcript| println!("{}", transcript.text()))?;
    Ok(())
}

fn speak(text: &str, output: Option<PathBuf>, play: bool, prosody: Prosody) -> Result<()> {
    let mut synthesizer = PiperSynthesizer::load(prosody)?;
    eprintln!("voice: Piper id_ID (female)");
    let waveform = SynthesizeText::new(&mut synthesizer).execute(text)?;
    eprintln!("synthesized {:.1}s of audio", waveform.duration_secs());

    let path = output.unwrap_or_else(|| PathBuf::from("speech.wav"));
    WavWriter.write(&waveform, &path)?;
    eprintln!("wrote {}", path.display());

    if play {
        CpalSpeaker.play(&waveform)?;
    }
    Ok(())
}

fn load_recognizer(model: &str, language: &str) -> Result<WhisperRecognizer> {
    let recognizer = WhisperRecognizer::load(&ModelId::new(model), &Language::new(language))?;
    eprintln!("device: {}", recognizer.backend());
    Ok(recognizer)
}
