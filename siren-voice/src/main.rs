use anyhow::Result;
use clap::Parser;
use siren_agent_claude::ClaudeCodeAgent;
use siren_application::{VoiceEvent, VoiceLoop};
use siren_asr::WhisperRecognizer;
use siren_asr_gemini::GeminiRecognizer;
use siren_domain::{Language, ModelId, SpeechRecognizer};
use siren_mic::CpalMicrophone;
use siren_speaker::CpalStreamingSpeaker;
use siren_tts_gemini::GeminiSynthesizer;
use siren_vad::EnergyVad;

#[derive(Parser)]
#[command(
    name = "siren-voice",
    about = "Talk to Claude Code by voice: speak → transcribe → Claude Code → speak the reply"
)]
struct Cli {
    #[arg(long)]
    local: bool,
    #[arg(long, default_value = siren_gemini::DEFAULT_VOICE)]
    voice: String,
    #[arg(long, default_value = "en")]
    language: String,
    #[arg(long, default_value = "openai/whisper-base")]
    model: String,
    #[arg(long, default_value = siren_agent_claude::DEFAULT_PERMISSION_MODE)]
    permission_mode: String,
    #[arg(long)]
    system: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let microphone = CpalMicrophone;
    let mut vad = EnergyVad::new();
    let mut recognizer = load_recognizer(cli.local, &cli.model, &cli.language)?;
    let mut agent = ClaudeCodeAgent::new(cli.permission_mode, cli.system);
    let mut synthesizer =
        GeminiSynthesizer::load(siren_gemini::TTS_MODEL.to_string(), cli.voice.clone())?;
    let speaker = CpalStreamingSpeaker;

    let engine = if cli.local { "local Whisper" } else { "Gemini" };
    eprintln!("voice coding with Claude Code — speak; Ctrl-C to stop");
    eprintln!("stt: {engine} · reply voice: Gemini {}", cli.voice);

    VoiceLoop::new(&microphone, &mut vad, recognizer.as_mut(), &mut agent, &mut synthesizer, &speaker)
        .execute(|event| match event {
            VoiceEvent::Heard(text) => eprintln!("\nyou:    {text}"),
            VoiceEvent::Replied(text) => println!("claude: {text}"),
        })?;
    Ok(())
}

fn load_recognizer(
    local: bool,
    model: &str,
    language: &str,
) -> Result<Box<dyn SpeechRecognizer>> {
    if local {
        let recognizer = WhisperRecognizer::load(&ModelId::new(model), &Language::new(language))?;
        eprintln!("device: {}", recognizer.backend());
        Ok(Box::new(recognizer))
    } else {
        Ok(Box::new(GeminiRecognizer::load(
            siren_gemini::STT_MODEL.to_string(),
            language.to_string(),
        )?))
    }
}
