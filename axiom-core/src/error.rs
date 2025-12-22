use thiserror::Error;

#[derive(Error, Debug)]
pub enum AxiomError {
    #[error("Axiom violation: {0}")]
    Violation(String),

    #[error("Axiom not found: {0}")]
    NotFound(String),

    #[error("Invalid axiom configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Axiom evaluation failed: {0}")]
    EvaluationFailed(String),
}

#[derive(Error, Debug)]
pub enum AxiomHiveError {
    #[error("Axiom error: {0}")]
    AxiomError(#[from] AxiomError),

    #[error("Substrate verification failed: {0}")]
    SubstrateError(String),

    #[error("HTTP request failed: {0}")]
    RequestError(String),

    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),

    #[error("LST logging failed: {0}")]
    LoggingError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl AxiomHiveError {
    pub fn substrate_error(msg: impl Into<String>) -> Self {
        Self::SubstrateError(msg.into())
    }

    pub fn request_error(msg: impl Into<String>) -> Self {
        Self::RequestError(msg.into())
    }

    pub fn logging_error(msg: impl Into<String>) -> Self {
        Self::LoggingError(msg.into())
    }
}
