use anyhow::{anyhow, Context, Result};
use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as m, model::Whisper, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use siren_domain::{DomainError, Language, ModelId};
use tokenizers::Tokenizer;

use crate::device;
use crate::recognizer::WhisperRecognizer;

const MEL_FILTERS_80: &[u8] = include_bytes!("../assets/melfilters.bytes");
const TOKENIZER_FALLBACK_REPO: &str = "openai/whisper-tiny";

impl WhisperRecognizer {
    pub fn load(model_id: &ModelId, language: &Language) -> Result<Self, DomainError> {
        load_inner(model_id.as_str(), language.code()).map_err(DomainError::model)
    }
}

fn load_inner(model_id: &str, language: &str) -> Result<WhisperRecognizer> {
    let device = device::pick_device().map_err(|e| anyhow!("{e}"))?;
    let backend = device::label(&device);

    let api = Api::new()?;
    let repo = api.repo(Repo::with_revision(
        model_id.to_string(),
        RepoType::Model,
        "main".to_string(),
    ));

    let config: Config =
        serde_json::from_reader(std::fs::File::open(repo.get("config.json")?)?)?;

    let tokenizer_path = match repo.get("tokenizer.json") {
        Ok(path) => path,
        Err(_) => api
            .model(TOKENIZER_FALLBACK_REPO.to_string())
            .get("tokenizer.json")?,
    };
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow!("{e}"))?;

    let vb = match repo.get("model.safetensors") {
        Ok(path) => unsafe { VarBuilder::from_mmaped_safetensors(&[path], m::DTYPE, &device)? },
        Err(_) => {
            let path = repo.get("pytorch_model.bin").context("missing model weights")?;
            VarBuilder::from_pth(&path, m::DTYPE, &device)?
        }
    };
    let model = Whisper::load(&vb, config.clone())?;
    let mel_filters = load_mel_filters(config.num_mel_bins)?;

    let suppress: Vec<f32> = (0..config.vocab_size as u32)
        .map(|i| if config.suppress_tokens.contains(&i) { f32::NEG_INFINITY } else { 0.0 })
        .collect();
    let suppress = Tensor::new(suppress.as_slice(), &device)?;

    let token = |t: &str| tokenizer.token_to_id(t).ok_or_else(|| anyhow!("missing token {t}"));

    Ok(WhisperRecognizer {
        sot_token: token(m::SOT_TOKEN)?,
        transcribe_token: token(m::TRANSCRIBE_TOKEN)?,
        eot_token: token(m::EOT_TOKEN)?,
        no_timestamps_token: token(m::NO_TIMESTAMPS_TOKEN)?,
        language_token: token(&format!("<|{language}|>"))?,
        suppress,
        model,
        tokenizer,
        config,
        mel_filters,
        device,
        backend,
    })
}

fn load_mel_filters(num_mel_bins: usize) -> Result<Vec<f32>> {
    let bytes = match num_mel_bins {
        80 => MEL_FILTERS_80,
        other => return Err(anyhow!("unsupported num_mel_bins={other}")),
    };
    Ok(bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}
