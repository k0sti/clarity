// Stable Diffusion implementation using Candle

use crate::{GeneratedImage, ImageGenConfig, ImageGenError, ImageGenerator, Result};
use candle_core::{DType, Device, IndexOp, Module, Tensor};
use candle_transformers::models::stable_diffusion::{self, StableDiffusionConfig};
use tokenizers::Tokenizer;

/// Stable Diffusion model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StableDiffusionVersion {
    /// Stable Diffusion v1.5
    V1_5,
    /// Stable Diffusion v2.1
    V2_1,
    /// Stable Diffusion XL 1.0
    Xl,
    /// Stable Diffusion Turbo (fastest)
    Turbo,
}

impl StableDiffusionVersion {
    pub fn repo(&self) -> &str {
        match self {
            Self::V1_5 => "runwayml/stable-diffusion-v1-5",
            Self::V2_1 => "stabilityai/stable-diffusion-2-1",
            Self::Xl => "stabilityai/stable-diffusion-xl-base-1.0",
            Self::Turbo => "stabilityai/sdxl-turbo",
        }
    }

    pub fn default_steps(&self) -> usize {
        match self {
            Self::V1_5 | Self::V2_1 => 10,  // Reduced for faster generation
            Self::Xl => 30,
            Self::Turbo => 1,
        }
    }

    pub fn default_guidance(&self) -> f64 {
        match self {
            Self::Turbo => 0.0,
            _ => 7.5,
        }
    }
}

/// Stable Diffusion image generator
pub struct StableDiffusionGenerator {
    version: StableDiffusionVersion,
    device: Device,
    dtype: DType,
    guidance_scale: f64,
    vae_scale: f64,
}

impl StableDiffusionGenerator {
    /// Create a new Stable Diffusion generator
    pub fn new(version: StableDiffusionVersion, use_cpu: bool) -> Result<Self> {
        // SDXL and Turbo require dual text encoders (CLIP-L + OpenCLIP-G)
        // which is not yet implemented
        if matches!(version, StableDiffusionVersion::Xl | StableDiffusionVersion::Turbo) {
            return Err(ImageGenError::InvalidConfig(
                "SDXL and SD-Turbo are not yet supported. These models require dual text encoders (CLIP + CLIP2) which is not yet implemented. Please use SD v1.5 or v2.1 instead.".into()
            ));
        }

        let device = if use_cpu {
            Device::Cpu
        } else {
            Device::cuda_if_available(0)?
        };

        let dtype = if device.is_cuda() {
            DType::F16
        } else {
            DType::F32
        };

        // VAE scale factor: standard models use 0.18215, Turbo uses 0.13025
        let vae_scale = match version {
            StableDiffusionVersion::Turbo => 0.13025,
            _ => 0.18215,
        };

        Ok(Self {
            version,
            device,
            dtype,
            guidance_scale: version.default_guidance(),
            vae_scale,
        })
    }

    /// Set guidance scale (classifier-free guidance)
    pub fn with_guidance_scale(mut self, scale: f64) -> Self {
        self.guidance_scale = scale;
        self
    }

    fn get_sd_config(&self) -> StableDiffusionConfig {
        match self.version {
            StableDiffusionVersion::V1_5 => StableDiffusionConfig::v1_5(None, None, None),
            StableDiffusionVersion::V2_1 => StableDiffusionConfig::v2_1(None, None, None),
            StableDiffusionVersion::Xl => StableDiffusionConfig::sdxl(None, None, None),
            StableDiffusionVersion::Turbo => StableDiffusionConfig::sdxl_turbo(None, None, None),
        }
    }

    fn download_file(&self, filename: &str) -> Result<std::path::PathBuf> {
        let api = hf_hub::api::sync::Api::new()
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;

        let path = api
            .model(self.version.repo().to_string())
            .get(filename)
            .map_err(|e| ImageGenError::HfHub(e.to_string()))?;

        Ok(path)
    }

    fn text_embeddings(&self, prompt: &str, sd_config: &StableDiffusionConfig) -> Result<Tensor> {
        tracing::info!("Loading CLIP tokenizer and encoder");

        // Download tokenizer from OpenAI CLIP repo (different for each SD version)
        let tokenizer_repo = match self.version {
            StableDiffusionVersion::V1_5 | StableDiffusionVersion::V2_1 => {
                "openai/clip-vit-base-patch32"
            }
            StableDiffusionVersion::Xl | StableDiffusionVersion::Turbo => {
                "openai/clip-vit-large-patch14"
            }
        };

        tracing::debug!("Downloading tokenizer from: {}", tokenizer_repo);
        let api = hf_hub::api::sync::Api::new()
            .map_err(|e| ImageGenError::HfHub(format!("Failed to create API: {}", e)))?;
        let tokenizer_path = api
            .model(tokenizer_repo.to_string())
            .get("tokenizer.json")
            .map_err(|e| ImageGenError::HfHub(format!("Failed to download tokenizer.json: {}", e)))?;
        tracing::debug!("Tokenizer downloaded to: {:?}", tokenizer_path);

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| ImageGenError::Tokenization(e.to_string()))?;

        // Get padding token
        let pad_id = match &sd_config.clip.pad_with {
            Some(padding) => *tokenizer
                .get_vocab(true)
                .get(padding.as_str())
                .ok_or_else(|| ImageGenError::Tokenization("Pad token not found".into()))?,
            None => *tokenizer
                .get_vocab(true)
                .get("<|endoftext|>")
                .ok_or_else(|| ImageGenError::Tokenization("End token not found".into()))?,
        };

        // Tokenize and pad prompt
        let mut tokens = tokenizer
            .encode(prompt, true)
            .map_err(|e| ImageGenError::Tokenization(e.to_string()))?
            .get_ids()
            .to_vec();

        while tokens.len() < sd_config.clip.max_position_embeddings {
            tokens.push(pad_id);
        }
        let tokens = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?;

        // Load CLIP weights from the SD model repo
        // Try fp16 first if using F16 dtype, fallback to fp32 if not available
        let clip_weights = if self.dtype == DType::F16 {
            tracing::info!("Attempting to load fp16 CLIP weights");
            match self.download_file("text_encoder/model.fp16.safetensors") {
                Ok(path) => path,
                Err(_) => {
                    tracing::warn!("fp16 CLIP weights not found, falling back to fp32");
                    self.download_file("text_encoder/model.safetensors")?
                }
            }
        } else {
            self.download_file("text_encoder/model.safetensors")?
        };

        // Build text model
        let text_model = stable_diffusion::build_clip_transformer(
            &sd_config.clip,
            clip_weights,
            &self.device,
            self.dtype,
        )?;

        tracing::info!("Encoding prompt");
        let text_embeddings = text_model.forward(&tokens)?;

        Ok(text_embeddings)
    }
}

impl ImageGenerator for StableDiffusionGenerator {
    fn generate(&mut self, config: &ImageGenConfig) -> Result<GeneratedImage> {
        tracing::info!("Generating image with Stable Diffusion {:?}", self.version);
        tracing::info!("Prompt: {}", config.prompt);

        // Validate dimensions
        if config.width % 8 != 0 || config.height % 8 != 0 {
            return Err(ImageGenError::InvalidConfig(
                "Width and height must be multiples of 8".into(),
            ));
        }

        let seed = config.seed.unwrap_or_else(|| rand::random());
        tracing::info!("Using seed: {}", seed);

        // Get SD config
        let sd_config = self.get_sd_config();

        // 1. Encode text prompt
        let text_embeddings = self.text_embeddings(&config.prompt, &sd_config)?;

        // 2. Create uncond embeddings if using guidance
        let text_embeddings = if self.guidance_scale > 1.0 {
            tracing::info!("Creating unconditional embeddings for guidance");
            let uncond_embeddings = self.text_embeddings("", &sd_config)?;
            Tensor::cat(&[uncond_embeddings, text_embeddings], 0)?
        } else {
            text_embeddings
        };

        // 3. Load VAE
        tracing::info!("Loading VAE");
        let vae_weights_file = if self.dtype == DType::F16 {
            // For SDXL fp16, use the fixed VAE from madebyollin
            if matches!(self.version, StableDiffusionVersion::Xl | StableDiffusionVersion::Turbo) {
                tracing::info!("Using SDXL VAE fp16 fix from madebyollin");
                let api = hf_hub::api::sync::Api::new()
                    .map_err(|e| ImageGenError::HfHub(e.to_string()))?;
                api
                    .model("madebyollin/sdxl-vae-fp16-fix".to_string())
                    .get("diffusion_pytorch_model.safetensors")
                    .map_err(|e| ImageGenError::HfHub(e.to_string()))?
            } else {
                // Try fp16, fallback to fp32
                match self.download_file("vae/diffusion_pytorch_model.fp16.safetensors") {
                    Ok(path) => path,
                    Err(_) => {
                        tracing::warn!("fp16 VAE weights not found, falling back to fp32");
                        self.download_file("vae/diffusion_pytorch_model.safetensors")?
                    }
                }
            }
        } else {
            self.download_file("vae/diffusion_pytorch_model.safetensors")?
        };
        let vae = sd_config.build_vae(vae_weights_file, &self.device, self.dtype)?;

        // 4. Load UNet
        tracing::info!("Loading UNet");
        let unet_weights = if self.dtype == DType::F16 {
            tracing::info!("Attempting to load fp16 UNet weights");
            match self.download_file("unet/diffusion_pytorch_model.fp16.safetensors") {
                Ok(path) => path,
                Err(_) => {
                    tracing::warn!("fp16 UNet weights not found, falling back to fp32");
                    self.download_file("unet/diffusion_pytorch_model.safetensors")?
                }
            }
        } else {
            self.download_file("unet/diffusion_pytorch_model.safetensors")?
        };
        let unet = sd_config.build_unet(
            unet_weights,
            &self.device,
            4, // standard latent channels
            false, // no flash attention
            self.dtype,
        )?;

        // 5. Initialize latents
        tracing::info!("Initializing latents");
        let latent_height = config.height / 8;
        let latent_width = config.width / 8;

        use rand::SeedableRng;
        let _rng = rand::rngs::StdRng::from_seed({
            let mut seed_bytes = [0u8; 32];
            seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
            seed_bytes
        });

        let mut latents = Tensor::randn(
            0f32,
            1f32,
            (1, 4, latent_height, latent_width),
            &self.device,
        )?
        .to_dtype(self.dtype)?;

        // 6. Create scheduler
        let mut scheduler = sd_config.build_scheduler(config.num_steps)?;
        let timesteps = scheduler.timesteps();
        let timesteps = timesteps.to_vec(); // Convert to owned vec

        // 7. Diffusion loop
        tracing::info!("Running diffusion for {} steps", config.num_steps);
        for (step_idx, &timestep) in timesteps.iter().enumerate() {
            tracing::debug!("Step {}/{}", step_idx + 1, config.num_steps);

            let latent_model_input = if self.guidance_scale > 1.0 {
                Tensor::cat(&[&latents, &latents], 0)?
            } else {
                latents.clone()
            };

            let latent_model_input = scheduler.scale_model_input(latent_model_input, timestep)?;

            let noise_pred = unet.forward(&latent_model_input, timestep as f64, &text_embeddings)?;

            let noise_pred = if self.guidance_scale > 1.0 {
                let noise_pred = noise_pred.chunk(2, 0)?;
                let (uncond, text) = (&noise_pred[0], &noise_pred[1]);
                (uncond + ((text - uncond)? * self.guidance_scale)?)?
            } else {
                noise_pred
            };

            latents = scheduler.step(&noise_pred, timestep, &latents)?;
        }

        // 8. Decode latents
        tracing::info!("Decoding latents to image");
        let image = vae.decode(&(&latents / self.vae_scale)?)?;
        let image = ((image / 2.)? + 0.5)?.to_device(&Device::Cpu)?;
        let image = (image.clamp(0f32, 1.)? * 255.)?.to_dtype(DType::U8)?;

        // 9. Convert to RGB bytes
        let (_, _, height, width) = image.dims4()?;
        let image = image.i(0)?;
        let data = image.permute((1, 2, 0))?.to_vec3::<u8>()?;
        let data: Vec<u8> = data.into_iter().flatten().flatten().collect();

        tracing::info!("Image generation complete!");

        Ok(GeneratedImage {
            data,
            width: width as u32,
            height: height as u32,
            prompt: config.prompt.clone(),
            seed,
        })
    }
}
