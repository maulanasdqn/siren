use anyhow::Result;
use siren_application::Converse;
use siren_domain::MicrophoneSource;
use siren_live_gemini::GeminiLiveAgent;
use siren_mic::CpalMicrophone;
use siren_speaker::CpalStreamingSpeaker;

pub fn converse(model: String, voice: String, system: String) -> Result<()> {
    let mut agent = GeminiLiveAgent::load(model, voice.clone(), system)?;
    let mic = CpalMicrophone.open()?;
    eprintln!("live: Gemini {voice} — speak; Ctrl-C to stop");
    Converse::new(&mut agent).execute(mic, &CpalStreamingSpeaker)?;
    Ok(())
}
