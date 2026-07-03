use candle_core::{Device, Error as CandleError, IndexOp, Result as CandleResult, Tensor};
use candle_transformers::models::whisper::{self as m, audio, model::Whisper, Config};
use siren_domain::{AudioSamples, DomainError, SpeechRecognizer, Transcript};
use tokenizers::Tokenizer;

pub struct WhisperRecognizer {
    pub(crate) model: Whisper,
    pub(crate) tokenizer: Tokenizer,
    pub(crate) config: Config,
    pub(crate) mel_filters: Vec<f32>,
    pub(crate) device: Device,
    pub(crate) suppress: Tensor,
    pub(crate) sot_token: u32,
    pub(crate) transcribe_token: u32,
    pub(crate) eot_token: u32,
    pub(crate) no_timestamps_token: u32,
    pub(crate) language_token: u32,
    pub(crate) backend: &'static str,
}

impl WhisperRecognizer {
    pub fn backend(&self) -> &'static str {
        self.backend
    }

    fn decode_chunk(&mut self, pcm: &[f32]) -> CandleResult<String> {
        let mel = audio::pcm_to_mel(&self.config, pcm, &self.mel_filters);
        let n_mel = self.config.num_mel_bins;
        let frames = mel.len() / n_mel;
        let mel = Tensor::from_vec(mel, (1, n_mel, frames), &self.device)?;
        let mel = mel.narrow(2, 0, m::N_FRAMES)?;

        let features = self.model.encoder.forward(&mel, true)?;
        let sample_len = self.config.max_target_positions / 2;
        let mut tokens = vec![
            self.sot_token,
            self.language_token,
            self.transcribe_token,
            self.no_timestamps_token,
        ];

        for i in 0..sample_len {
            let input = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?;
            let ys = self.model.decoder.forward(&input, &features, i == 0)?;
            let (_, seq_len, _) = ys.dims3()?;
            let logits = self
                .model
                .decoder
                .final_linear(&ys.i((..1, seq_len - 1..))?)?
                .i(0)?
                .i(0)?;
            let next = logits.broadcast_add(&self.suppress)?.argmax(0)?.to_scalar::<u32>()?;
            if next == self.eot_token {
                break;
            }
            tokens.push(next);
        }

        self.tokenizer
            .decode(&tokens[4..], true)
            .map_err(|e| CandleError::Msg(e.to_string()))
    }
}

impl SpeechRecognizer for WhisperRecognizer {
    fn transcribe(&mut self, audio: &AudioSamples) -> Result<Transcript, DomainError> {
        let mut transcript = Transcript::default();
        for chunk in audio.as_slice().chunks(m::N_SAMPLES) {
            let mut padded = chunk.to_vec();
            padded.resize(m::N_SAMPLES, 0.0);
            let text = self.decode_chunk(&padded).map_err(DomainError::inference)?;
            transcript.push_segment(&text);
        }
        Ok(transcript)
    }
}
