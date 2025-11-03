// Error types for imagent

use thiserror::Error;

/// Result type for imagent operations
pub type Result<T> = std::result::Result<T, ImageGenError>;

/// Errors that can occur during image generation
#[derive(Error, Debug)]
pub enum ImageGenError {
    #[error("Candle error: {0}")]
    Candle(#[from] candle_core::Error),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("Model loading error: {0}")]
    ModelLoading(String),

    #[error("Tokenization error: {0}")]
    Tokenization(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HuggingFace Hub error: {0}")]
    HfHub(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<image::ImageError> for ImageGenError {
    fn from(err: image::ImageError) -> Self {
        ImageGenError::ImageProcessing(err.to_string())
    }
}
