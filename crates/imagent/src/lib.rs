// imagent - AI Image Generation Library
// Provides interface for generating images using Flux models via Candle

// Full implementation (work in progress - in flux_wip directory)
// mod flux;

// Stub implementations (currently active)
mod flux_stub;
mod stable_diffusion;

pub mod error;

pub use error::{ImageGenError, Result};
pub use flux_stub::{FluxGenerator, FluxModel};
pub use stable_diffusion::{StableDiffusionGenerator, StableDiffusionVersion};

use serde::{Deserialize, Serialize};

/// Configuration for image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenConfig {
    /// Text prompt describing the image to generate
    pub prompt: String,

    /// Width of the generated image (must be multiple of 8)
    pub width: usize,

    /// Height of the generated image (must be multiple of 8)
    pub height: usize,

    /// Number of inference steps (more steps = better quality but slower)
    pub num_steps: usize,

    /// Random seed for reproducibility (None for random)
    pub seed: Option<u64>,

    /// Use quantized models (faster, less memory, slightly lower quality)
    pub quantized: bool,

    /// Use CPU instead of GPU
    pub use_cpu: bool,
}

impl Default for ImageGenConfig {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            width: 1024,
            height: 1024,
            num_steps: 4, // Schnell default
            seed: None,
            quantized: false,
            use_cpu: false,
        }
    }
}

/// Result of image generation
#[derive(Debug, Clone)]
pub struct GeneratedImage {
    /// Image data as RGB bytes
    pub data: Vec<u8>,

    /// Image width
    pub width: u32,

    /// Image height
    pub height: u32,

    /// Prompt used to generate the image
    pub prompt: String,

    /// Seed used for generation
    pub seed: u64,
}

impl GeneratedImage {
    /// Save the image to a file
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        use image::{ImageBuffer, Rgb};

        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(
            self.width,
            self.height,
            self.data.clone(),
        )
        .ok_or_else(|| ImageGenError::ImageProcessing("Failed to create image buffer".into()))?;

        img.save(path)
            .map_err(|e| ImageGenError::ImageProcessing(e.to_string()))?;

        Ok(())
    }
}

/// Trait for image generation backends
pub trait ImageGenerator {
    /// Generate an image from the given configuration
    fn generate(&mut self, config: &ImageGenConfig) -> Result<GeneratedImage>;
}
