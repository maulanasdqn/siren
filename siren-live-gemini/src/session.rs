use crate::{protocol, Config};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use siren_domain::{AudioSamples, Waveform};
use std::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub async fn run(
    config: Config,
    mic: Box<dyn Iterator<Item = AudioSamples> + Send>,
    out_tx: Sender<Waveform>,
) -> Result<()> {
    let (ws, _) = connect_async(siren_gemini::ws_url(&config.api_key)).await?;
    let (mut write, mut read) = ws.split();
    write.send(Message::from(protocol::setup(&config))).await?;

    let (pcm_tx, mut pcm_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    std::thread::spawn(move || {
        for block in mic {
            if pcm_tx.blocking_send(protocol::pcm_bytes(block.as_slice())).is_err() {
                break;
            }
        }
    });

    loop {
        tokio::select! {
            pcm = pcm_rx.recv() => match pcm {
                Some(bytes) => write.send(Message::from(protocol::realtime_input(&bytes))).await?,
                None => break,
            },
            message = read.next() => match message {
                Some(Ok(Message::Ping(payload))) => write.send(Message::Pong(payload)).await?,
                Some(Ok(message)) => {
                    for wave in protocol::parse(&payload(message))? {
                        if out_tx.send(wave).is_err() {
                            return Ok(());
                        }
                    }
                }
                Some(Err(e)) => return Err(e.into()),
                None => break,
            }
        }
    }
    Ok(())
}

fn payload(message: Message) -> Vec<u8> {
    match message {
        Message::Text(text) => text.as_bytes().to_vec(),
        Message::Binary(bytes) => bytes.to_vec(),
        _ => Vec::new(),
    }
}
