use core::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("audio error: {0}")]
    Audio(String),
    #[error("model error: {0}")]
    Model(String),
    #[error("inference error: {0}")]
    Inference(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
}

impl DomainError {
    pub fn audio(e: impl Display) -> Self {
        Self::Audio(e.to_string())
    }

    pub fn model(e: impl Display) -> Self {
        Self::Model(e.to_string())
    }

    pub fn inference(e: impl Display) -> Self {
        Self::Inference(e.to_string())
    }

    pub fn io(e: impl Display) -> Self {
        Self::Io(e.to_string())
    }

    pub fn unsupported(e: impl Display) -> Self {
        Self::Unsupported(e.to_string())
    }
}
