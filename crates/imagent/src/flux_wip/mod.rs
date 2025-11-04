// Flux model implementation for image generation

pub mod model;
pub mod sampling;

use crate::{GeneratedImage, ImageGenConfig, ImageGenError, ImageGenerator, Result};
use candle_core::{DType, Device, IndexOp, Module, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::{clip, flux, t5};
use flux::WithForward;
use rand::SeedableRng;
use std::path::PathBuf;
use tokenizers::Tokenizer;

/// Flux model variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluxModel {
    /// Schnell variant - 4 steps, faster
    Schnell,
    /// Dev variant - 50 steps, higher quality
    Dev,
}

impl FluxModel {
    pub fn repo(&self) -> &str {
        match self {
            Self::Schnell => "black-forest-labs/FLUX.1-schnell",
            Self::Dev => "black-forest-labs/FLUX.1-dev",
        }
    }

    pub fn default_steps(&self) -> usize {
        match self {
            Self::Schnell => 4,
            Self::Dev => 50,
        }
    }
}

/// Flux image generator
pub struct FluxGenerator {
    model: FluxModel,
    device: Device,
    dtype: DType,
    flux_model: Option<flux::model::Flux>,
    t5_encoder: Option<t5::T5EncoderModel>,
    clip_encoder: Option<clip::text_model::ClipTextTransformer>,
    ae: Option<flux::autoencoder::AutoEncoder>,
    t5_tokenizer: Option<Tokenizer>,
    clip_tokenizer: Option<Tokenizer>,
    cache_dir: PathBuf,
}

impl FluxGenerator {
    /// Create a new Flux generator
    pub fn new(model: FluxModel, use_cpu: bool) -> Result<Self> {
        let device = if use_cpu {
            Device::Cpu
        } else {
            Device::cuda_if_available(0)?
        };

        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };

        // Get cache directory
        let cache_dir = std::env::var("HF_HOME")
            .or_else(|_| std::env::var("XDG_CACHE_HOME").map(|d| format!("{}/huggingface", d)))
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.cache/huggingface", home)
            });

        Ok(Self {
            model,
            device,
            dtype,
            flux_model: None,
            t5_encoder: None,
            clip_encoder: None,
            ae: None,
            t5_tokenizer: None,
            clip_tokenizer: None,
            cache_dir: PathBuf::from(cache_dir),
        })
    }

    /// Load all required models
    fn load_models(&mut self, quantized: bool) -> Result<()> {
        tracing::info!("Loading Flux models (quantized: {})", quantized);

        // Load tokenizers
        self.load_tokenizers()?;

        // Load T5 encoder
        self.load_t5_encoder()?;

        // Load CLIP encoder
        self.load_clip_encoder()?;

        // Load Flux model
        self.load_flux_model(quantized)?;

        // Load AutoEncoder
        self.load_autoencoder(quantized)?;

        tracing::info!("All models loaded successfully");
        Ok(())
    }

    fn load_tokenizers(&mut self) -> Result<()> {
        tracing::info!("Loading tokenizers");

        // For now, use pre-existing CLIP tokenizer from OpenAI
        // The Flux tokenizer files are in a different format that requires more complex loading
        tracing::info!("Loading CLIP tokenizer from OpenAI CLIP repo");
        let clip_api = hf_hub::api::sync::Api::new()
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;
        let clip_tokenizer_path = clip_api
            .model("openai/clip-vit-large-patch14".to_string())
            .get("tokenizer.json")
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;

        self.clip_tokenizer = Some(
            Tokenizer::from_file(&clip_tokenizer_path)
                .map_err(|e| ImageGenError::Tokenization(format!("Failed to load CLIP tokenizer: {}", e)))?,
        );

        // Load T5 tokenizer from Google's T5 repo
        // The Flux model uses the same T5 tokenizer
        tracing::info!("Loading T5 tokenizer from Google T5 repo");
        let t5_api = hf_hub::api::sync::Api::new()
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;
        let t5_tokenizer_path = t5_api
            .model("google-t5/t5-large".to_string())
            .get("tokenizer.json")
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;

        self.t5_tokenizer = Some(
            Tokenizer::from_file(&t5_tokenizer_path)
                .map_err(|e| ImageGenError::Tokenization(format!("Failed to load T5 tokenizer: {}", e)))?,
        );

        Ok(())
    }

    fn load_t5_encoder(&mut self) -> Result<()> {
        tracing::info!("Loading T5 encoder");

        // Create T5-v1.1-XXL config (matches text_encoder_2 in Flux models)
        // Based on https://huggingface.co/google/t5-v1_1-xxl/blob/main/config.json
        let config = t5::Config {
            vocab_size: 32128,
            d_model: 4096,
            d_kv: 64,
            d_ff: 10240,
            num_layers: 24,
            num_decoder_layers: Some(24),
            num_heads: 64,
            relative_attention_num_buckets: 32,
            relative_attention_max_distance: 128,
            dropout_rate: 0.1,
            layer_norm_epsilon: 1e-6,
            initializer_factor: 1.0,
            feed_forward_proj: t5::ActivationWithOptionalGating {
                gated: true,
                activation: candle_nn::Activation::Relu,
            },
            is_encoder_decoder: true,
            tie_word_embeddings: false,
            is_decoder: false,
            use_cache: false,
            pad_token_id: 0,
            eos_token_id: 1,
            decoder_start_token_id: Some(0),
        };

        // Load T5 weights from the Flux model repo (sharded across 2 files)
        tracing::info!("Loading T5 weights (sharded across 2 files)");
        let weights_path_1 = self.download_file(
            self.model.repo(),
            "text_encoder_2/model-00001-of-00002.safetensors",
        )?;
        let weights_path_2 = self.download_file(
            self.model.repo(),
            "text_encoder_2/model-00002-of-00002.safetensors",
        )?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path_1, weights_path_2], self.dtype, &self.device)?
        };

        self.t5_encoder = Some(t5::T5EncoderModel::load(vb, &config)?);

        Ok(())
    }

    fn load_clip_encoder(&mut self) -> Result<()> {
        tracing::info!("Loading CLIP encoder");

        // Use CLIP-L config (matches text_encoder in Flux models)
        // This is the standard CLIP vit-large-patch14 configuration
        let config = clip::text_model::ClipTextConfig {
            vocab_size: 49408,
            embed_dim: 768,
            intermediate_size: 3072,
            max_position_embeddings: 77,
            pad_with: Some("!".to_string()),
            num_hidden_layers: 12,
            num_attention_heads: 12,
            projection_dim: 768,
            activation: clip::text_model::Activation::QuickGelu,
        };

        // Load CLIP weights from the Flux model repo
        tracing::info!("Loading CLIP weights from text_encoder/model.safetensors");
        let weights_path = self.download_file(
            self.model.repo(),
            "text_encoder/model.safetensors",
        )?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], self.dtype, &self.device)?
        };

        self.clip_encoder = Some(clip::text_model::ClipTextTransformer::new(vb, &config)?);

        Ok(())
    }

    fn load_flux_model(&mut self, quantized: bool) -> Result<()> {
        tracing::info!("Loading Flux model");

        if quantized {
            return Err(ImageGenError::ModelLoading(
                "Quantized models not yet implemented".into(),
            ));
        }

        // Load the main Flux transformer weights
        let filename = match self.model {
            FluxModel::Schnell => "transformer/diffusion_pytorch_model.safetensors",
            FluxModel::Dev => "transformer/diffusion_pytorch_model.safetensors",
        };

        let weights_path = self.download_file(self.model.repo(), filename)?;

        // Use the appropriate config for the model variant
        let config = match self.model {
            FluxModel::Schnell => flux::model::Config::schnell(),
            FluxModel::Dev => flux::model::Config::dev(),
        };

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], self.dtype, &self.device)?
        };

        self.flux_model = Some(flux::model::Flux::new(&config, vb)?);

        Ok(())
    }

    fn load_autoencoder(&mut self, _quantized: bool) -> Result<()> {
        tracing::info!("Loading AutoEncoder");

        let weights_path = self.download_file(
            self.model.repo(),
            "vae/diffusion_pytorch_model.safetensors",
        )?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], self.dtype, &self.device)?
        };

        // Create AutoEncoder config
        let config = flux::autoencoder::Config {
            resolution: 256,
            in_channels: 3,
            ch: 128,
            out_ch: 3,
            ch_mult: vec![1, 2, 4, 4],
            num_res_blocks: 2,
            z_channels: 16,
            scale_factor: 0.3611,
            shift_factor: 0.1159,
        };

        self.ae = Some(flux::autoencoder::AutoEncoder::new(&config, vb)?);

        Ok(())
    }

    fn download_file(&self, repo: &str, filename: &str) -> Result<PathBuf> {
        let api = hf_hub::api::sync::Api::new()
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;

        let repo = api.model(repo.to_string());

        let path = repo
            .get(filename)
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;

        Ok(path)
    }

    fn encode_prompt(&mut self, prompt: &str) -> Result<(Tensor, Tensor)> {
        // Encode with T5
        let t5_tokenizer = self
            .t5_tokenizer
            .as_ref()
            .ok_or_else(|| ImageGenError::ModelLoading("T5 tokenizer not loaded".into()))?;

        let t5_tokens = t5_tokenizer
            .encode(prompt, true)
            .map_err(|e| ImageGenError::Tokenization(e.to_string()))?
            .get_ids()
            .to_vec();

        let t5_token_ids = Tensor::new(&t5_tokens[..], &self.device)?
            .unsqueeze(0)?
            .to_dtype(DType::U32)?;

        let t5_encoder = self
            .t5_encoder
            .as_mut()
            .ok_or_else(|| ImageGenError::ModelLoading("T5 encoder not loaded".into()))?;

        let t5_embeddings = t5_encoder.forward(&t5_token_ids)?;

        // Encode with CLIP
        let clip_tokenizer = self
            .clip_tokenizer
            .as_ref()
            .ok_or_else(|| ImageGenError::ModelLoading("CLIP tokenizer not loaded".into()))?;

        let clip_tokens = clip_tokenizer
            .encode(prompt, true)
            .map_err(|e| ImageGenError::Tokenization(e.to_string()))?
            .get_ids()
            .to_vec();

        let clip_token_ids = Tensor::new(&clip_tokens[..], &self.device)?
            .unsqueeze(0)?
            .to_dtype(DType::U32)?;

        let clip_encoder = self
            .clip_encoder
            .as_mut()
            .ok_or_else(|| ImageGenError::ModelLoading("CLIP encoder not loaded".into()))?;

        let clip_embeddings = clip_encoder.forward(&clip_token_ids)?;

        Ok((t5_embeddings, clip_embeddings))
    }
}

impl ImageGenerator for FluxGenerator {
    fn generate(&mut self, config: &ImageGenConfig) -> Result<GeneratedImage> {
        tracing::info!("Generating image: {}", config.prompt);

        // Validate dimensions
        if config.width % 8 != 0 || config.height % 8 != 0 {
            return Err(ImageGenError::InvalidConfig(
                "Width and height must be multiples of 8".into(),
            ));
        }

        // Load models if not already loaded
        if self.flux_model.is_none() {
            self.load_models(config.quantized)?;
        }

        // Generate seed
        let seed = config.seed.unwrap_or_else(|| rand::random());
        tracing::info!("Using seed: {}", seed);

        // Encode prompt
        let (t5_embeddings, clip_embeddings) = self.encode_prompt(&config.prompt)?;

        // Initialize latents
        let latent_height = config.height / 8;
        let latent_width = config.width / 8;

        let _rng = rand::rngs::StdRng::from_seed({
            let mut seed_bytes = [0u8; 32];
            seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
            seed_bytes
        });

        let latents = Tensor::randn(
            0f32,
            1f32,
            (1, 16, latent_height, latent_width),
            &self.device,
        )?
        .to_dtype(self.dtype)?;

        // Run diffusion
        let flux_model = self
            .flux_model
            .as_ref()
            .ok_or_else(|| ImageGenError::ModelLoading("Flux model not loaded".into()))?;

        let timesteps = sampling::get_schedule(config.num_steps);

        // Create img_ids and txt_ids for positional embeddings
        let img_ids = Tensor::zeros((1, latent_height * latent_width, 3), self.dtype, &self.device)?;
        let txt_seq_len = t5_embeddings.dim(1)?;
        let txt_ids = Tensor::zeros((1, txt_seq_len, 3), self.dtype, &self.device)?;

        // Pooled text embeddings (CLIP)
        let y = clip_embeddings.mean(1)?;

        // Prepare latents as flattened sequence
        let mut img = latents.flatten(2, 3)?.transpose(1, 2)?; // [B, H*W, C]

        for (step, &t) in timesteps.iter().enumerate() {
            tracing::debug!("Step {}/{}", step + 1, config.num_steps);

            let guidance_value = 3.5f32;
            let guidance = Tensor::new(&[guidance_value], &self.device)?
                .to_dtype(self.dtype)?;

            let timestep_tensor = Tensor::new(&[t], &self.device)?
                .to_dtype(self.dtype)?;

            // Run model with WithForward trait
            let noise_pred = flux_model.forward(
                &img,
                &img_ids,
                &t5_embeddings,
                &txt_ids,
                &timestep_tensor,
                &y,
                Some(&guidance)
            )?;

            // Update latents using Euler method
            let dt = if step < timesteps.len() - 1 {
                timesteps[step + 1] - t
            } else {
                -t
            };

            let dt_tensor = Tensor::new(&[dt], &self.device)?
                .to_dtype(self.dtype)?;

            img = (&img + &noise_pred.broadcast_mul(&dt_tensor)?)?;
        }

        // Reshape back to image format
        let img = img.transpose(1, 2)?.reshape((1, 16, latent_height, latent_width))?;

        // Decode latents
        tracing::info!("Decoding image");
        let ae = self
            .ae
            .as_ref()
            .ok_or_else(|| ImageGenError::ModelLoading("AutoEncoder not loaded".into()))?;

        let decoded = ae.decode(&img)?;

        // Convert to image
        let decoded = ((decoded + 1.0)? * 127.5)?
            .clamp(0f32, 255f32)?
            .to_dtype(DType::U8)?;

        let (_, _, height, width) = decoded.dims4()?;
        let decoded = decoded.i((0, .., .., ..))?;

        let data = decoded.permute((1, 2, 0))?.to_vec3::<u8>()?;
        let data: Vec<u8> = data.into_iter().flatten().flatten().collect();

        Ok(GeneratedImage {
            data,
            width: width as u32,
            height: height as u32,
            prompt: config.prompt.clone(),
            seed,
        })
    }
}
