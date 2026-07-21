use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub const DEFAULT_MODEL: &str = "cahya/whisper-tiny-id";
pub const DEFAULT_LANGUAGE: &str = "id";

#[derive(Parser)]
#[command(
    name = "siren",
    about = "Gemini-powered AI voice engine (TTS · STT · realtime), with an offline local fallback"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub local: bool,
    #[arg(long, default_value = DEFAULT_MODEL, global = true)]
    pub model: String,
    #[arg(long, default_value = DEFAULT_LANGUAGE, global = true)]
    pub language: String,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
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
        #[arg(long, default_value = siren_gemini::DEFAULT_VOICE)]
        voice: String,
        #[arg(long, default_value_t = 1.15)]
        length_scale: f32,
        #[arg(long, default_value_t = 0.8)]
        noise_scale: f32,
        #[arg(long, default_value_t = 1.0)]
        noise_w: f32,
    },
    Converse {
        #[arg(long, default_value = siren_gemini::DEFAULT_VOICE)]
        voice: String,
        #[arg(long, default_value = "")]
        system: String,
        #[arg(long = "live-model", default_value = siren_gemini::LIVE_MODEL)]
        live_model: String,
    },
}
