use anyhow::Result;
use siren_application::{StreamSpeech, SynthesizeText};
use siren_audio::WavWriter;
use siren_domain::{SpeakerSink, WaveformWriter};
use siren_speaker::{CpalSpeaker, CpalStreamingSpeaker};
use siren_tts::{PiperSynthesizer, Prosody};
use siren_tts_gemini::GeminiSynthesizer;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub fn speak(
    local: bool,
    text: &str,
    output: Option<PathBuf>,
    play: bool,
    voice: String,
    length_scale: f32,
    noise_scale: f32,
    noise_w: f32,
) -> Result<()> {
    if local {
        let prosody = Prosody { length_scale, noise_scale, noise_w };
        speak_local(text, output, play, prosody)
    } else {
        speak_gemini(text, output, play, voice)
    }
}

fn speak_gemini(text: &str, output: Option<PathBuf>, play: bool, voice: String) -> Result<()> {
    let mut synthesizer = GeminiSynthesizer::load(siren_gemini::TTS_MODEL.to_string(), voice.clone())?;
    match output {
        None => {
            eprintln!("engine: Gemini {voice} (live streaming)");
            StreamSpeech::new(&mut synthesizer).execute(text, &CpalStreamingSpeaker)?;
        }
        Some(path) => {
            eprintln!("engine: Gemini {voice}");
            let waveform = SynthesizeText::new(&mut synthesizer).execute(text)?;
            eprintln!("synthesized {:.1}s of audio", waveform.duration_secs());
            WavWriter.write(&waveform, &path)?;
            eprintln!("wrote {}", path.display());
            if play {
                CpalSpeaker.play(&waveform)?;
            }
        }
    }
    Ok(())
}

fn speak_local(text: &str, output: Option<PathBuf>, play: bool, prosody: Prosody) -> Result<()> {
    let mut synthesizer = PiperSynthesizer::load(prosody)?;
    eprintln!("engine: local Piper id_ID (female)");
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
