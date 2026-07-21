mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Transcribe { file } => {
            commands::transcribe(cli.local, &cli.model, &cli.language, &file)
        }
        Command::Listen => commands::listen(cli.local, &cli.model, &cli.language),
        Command::Speak { text, output, play, voice, length_scale, noise_scale, noise_w } => {
            commands::speak(cli.local, &text, output, play, voice, length_scale, noise_scale, noise_w)
        }
        Command::Converse { voice, system, live_model } => {
            commands::converse(live_model, voice, system)
        }
    }
}
