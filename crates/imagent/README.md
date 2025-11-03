# imagent

AI Image Generation Library and CLI for Rust using Flux models with Candle.

## Overview

`imagent` is a Rust library and command-line tool for generating images using multiple AI models including Flux (Black Forest Labs) and Stable Diffusion (Stability AI), powered by the Candle ML framework from HuggingFace. It provides a clean, trait-based interface for image generation with support for both library integration and standalone CLI usage.

## Current Implementation Status

**Current Version**: Stub Implementation (v0.1.0)

The crate currently uses **stub implementations** that generate colorful gradient placeholder images. This allows:
- ✅ Testing the complete API surface
- ✅ Validating image saving and file I/O
- ✅ Developing applications that will use imagent
- ✅ CLI interface testing and integration
- ✅ Multi-model architecture testing

**Supported Models** (stub implementations):
- **Flux Models**: Schnell (4 steps), Dev (50 steps)
- **Stable Diffusion**: v1.5, v2.1, XL, Turbo (1 step)

**Full Implementation**: Work in progress. The complete implementation will include:
- T5-XXL and CLIP text encoders for prompt processing
- Flux/Stable Diffusion diffusion models for image generation
- VAE/AutoEncoder for latent-to-pixel decoding
- HuggingFace Hub integration for model downloading
- GPU acceleration with CUDA support
- Quantized model support (GGUF format)
- Classifier-free guidance
- Negative prompts

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
imagent = { path = "../imagent" }
```

Or build the CLI tool:

```bash
cargo build --release --bin imagent-bin
```

## Quick Start

### Command Line Usage

Generate an image with a text prompt:

```bash
# Default: Stable Diffusion Turbo (fastest)
imagent-bin --prompt "A rusty robot walking on a beach"

# Use Flux Schnell for faster generation
imagent-bin \
    --prompt "A beautiful sunset over mountains" \
    --model flux-schnell \
    --output sunset.png

# Use Stable Diffusion XL for high quality
imagent-bin \
    --prompt "Cyberpunk cityscape with neon lights" \
    --model sd-xl \
    --width 1024 \
    --height 768 \
    --output city.png

# Use Stable Diffusion v1.5 (classic)
imagent-bin \
    --prompt "A majestic castle on a mountain peak" \
    --model sd-v15 \
    --output castle.png

# Use specific seed for reproducibility
imagent-bin \
    --prompt "Abstract digital art" \
    --model sd-turbo \
    --seed 42 \
    --output art.png

# Flux Dev for higher quality (more steps)
imagent-bin \
    --prompt "Photorealistic portrait" \
    --model flux-dev \
    --num-steps 50 \
    --output portrait.png

# Custom dimensions (must be multiple of 8)
imagent-bin \
    --prompt "Wide landscape" \
    --width 1280 \
    --height 720 \
    --model sd-xl

# Verbose logging
imagent-bin \
    --prompt "Futuristic cityscape" \
    --model sd-v21 \
    --verbose
```

### Library Usage

#### Using Stable Diffusion

```rust
use imagent::{StableDiffusionGenerator, StableDiffusionVersion, ImageGenConfig, ImageGenerator};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Stable Diffusion Turbo generator (fastest)
    let mut generator = StableDiffusionGenerator::new(
        StableDiffusionVersion::Turbo,
        false  // use GPU
    )?;

    // Configure image generation
    let config = ImageGenConfig {
        prompt: "A majestic castle on a mountain peak at sunset".to_string(),
        width: 512,
        height: 512,
        num_steps: 1,  // Turbo only needs 1 step
        seed: Some(42),
        quantized: false,
        use_cpu: false,
    };

    // Generate the image
    let image = generator.generate(&config)?;

    // Save to file
    image.save(Path::new("castle.png"))?;

    println!("Image saved! Seed: {}", image.seed);
    Ok(())
}
```

#### Using Flux

```rust
use imagent::{FluxGenerator, FluxModel, ImageGenConfig, ImageGenerator};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Flux generator (Schnell variant for speed)
    let mut generator = FluxGenerator::new(FluxModel::Schnell, false)?;

    // Configure image generation
    let config = ImageGenConfig {
        prompt: "A rusty robot walking on a beach".to_string(),
        width: 1024,
        height: 1024,
        num_steps: 4,
        seed: Some(42),
        quantized: false,
        use_cpu: false,
    };

    // Generate the image
    let image = generator.generate(&config)?;

    // Save to file
    image.save(Path::new("robot.png"))?;

    println!("Image saved! Seed: {}", image.seed);
    Ok(())
}
```

## CLI Reference

### Required Arguments

- `--prompt, -p <PROMPT>`: Text description of the image to generate

### Optional Arguments

**Output Configuration:**
- `--output, -o <PATH>`: Output file path (default: `output.png`)
- `--width, -w <WIDTH>`: Image width in pixels, must be multiple of 8 (default: `1024`)
- `--height <HEIGHT>`: Image height in pixels, must be multiple of 8 (default: `1024`)

**Model Selection:**
- `--model, -m <MODEL>`: Model variant to use (default: `sd-turbo`)
  - **Flux Models:**
    - `flux-schnell`: Flux Schnell - 4 steps, faster
    - `flux-dev`: Flux Dev - 50 steps, higher quality
  - **Stable Diffusion Models:**
    - `sd-v15`: Stable Diffusion v1.5 - Classic, 30 steps
    - `sd-v21`: Stable Diffusion v2.1 - Improved, 30 steps
    - `sd-xl`: Stable Diffusion XL - High quality, 30 steps
    - `sd-turbo`: Stable Diffusion Turbo - Fastest, 1 step (default)

**Generation Parameters:**
- `--num-steps, -n <STEPS>`: Override default number of inference steps
- `--seed, -s <SEED>`: Random seed for reproducible generation
- `--quantized, -q`: Use quantized models (lower memory, faster)
- `--cpu`: Force CPU execution instead of GPU

**Logging:**
- `--verbose, -v`: Enable detailed logging output

**Other:**
- `--help, -h`: Display help information
- `--version, -V`: Display version information

## API Documentation

### Core Types

#### `ImageGenConfig`

Configuration structure for image generation:

```rust
pub struct ImageGenConfig {
    pub prompt: String,        // Text prompt
    pub width: usize,          // Image width (multiple of 8)
    pub height: usize,         // Image height (multiple of 8)
    pub num_steps: usize,      // Inference steps
    pub seed: Option<u64>,     // Random seed (None = random)
    pub quantized: bool,       // Use quantized models
    pub use_cpu: bool,         // Force CPU usage
}
```

Default values:
- `width`, `height`: 1024
- `num_steps`: 4 (Schnell default)
- `seed`: Random
- `quantized`: false
- `use_cpu`: false

#### `GeneratedImage`

Result of image generation:

```rust
pub struct GeneratedImage {
    pub data: Vec<u8>,      // RGB pixel data
    pub width: u32,         // Image width
    pub height: u32,        // Image height
    pub prompt: String,     // Generation prompt
    pub seed: u64,          // Seed used
}

impl GeneratedImage {
    pub fn save(&self, path: &Path) -> Result<()>;
}
```

#### `ImageGenerator` Trait

Interface for image generation backends:

```rust
pub trait ImageGenerator {
    fn generate(&mut self, config: &ImageGenConfig) -> Result<GeneratedImage>;
}
```

#### `FluxModel` Enum

Flux model variant selection:

```rust
pub enum FluxModel {
    Schnell,  // 4 steps, faster
    Dev,      // 50 steps, higher quality
}

impl FluxModel {
    pub fn repo(&self) -> &str;            // HuggingFace repo name
    pub fn default_steps(&self) -> usize;  // Default step count
}
```

#### `FluxGenerator`

Flux generator implementation:

```rust
pub struct FluxGenerator { /* ... */ }

impl FluxGenerator {
    pub fn new(model: FluxModel, use_cpu: bool) -> Result<Self>;
}

impl ImageGenerator for FluxGenerator {
    fn generate(&mut self, config: &ImageGenConfig) -> Result<GeneratedImage>;
}
```

#### `StableDiffusionVersion` Enum

Stable Diffusion model variant selection:

```rust
pub enum StableDiffusionVersion {
    V1_5,   // Classic, 30 steps
    V2_1,   // Improved, 30 steps
    Xl,     // High quality, 30 steps
    Turbo,  // Fastest, 1 step
}

impl StableDiffusionVersion {
    pub fn repo(&self) -> &str;            // HuggingFace repo name
    pub fn default_steps(&self) -> usize;  // Default step count
    pub fn default_guidance(&self) -> f64; // Default guidance scale
}
```

#### `StableDiffusionGenerator`

Stable Diffusion generator implementation:

```rust
pub struct StableDiffusionGenerator { /* ... */ }

impl StableDiffusionGenerator {
    pub fn new(version: StableDiffusionVersion, use_cpu: bool) -> Result<Self>;
    pub fn with_guidance_scale(self, scale: f64) -> Self;
}

impl ImageGenerator for StableDiffusionGenerator {
    fn generate(&mut self, config: &ImageGenConfig) -> Result<GeneratedImage>;
}
```

### Error Handling

```rust
pub enum ImageGenError {
    Candle(candle_core::Error),
    ImageProcessing(String),
    ModelLoading(String),
    Tokenization(String),
    InvalidConfig(String),
    Io(std::io::Error),
    HfHub(String),
    Json(serde_json::Error),
    Other(anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ImageGenError>;
```

## Architecture

### Project Structure

```
crates/imagent/
├── Cargo.toml                # Dependencies and binary configuration
├── README.md                 # This file
└── src/
    ├── lib.rs               # Public API and core types
    ├── error.rs             # Error types and Result alias
    ├── flux_stub.rs         # Current stub implementation
    ├── flux_wip/            # Full implementation (WIP)
    │   ├── mod.rs          # Main Flux implementation
    │   ├── model.rs        # Model definitions
    │   └── sampling.rs     # Diffusion sampling schedules
    └── bin/
        └── imagent-bin.rs  # CLI application
```

### Dependencies

**Core ML Framework:**
- `candle-core` (0.9.2-alpha.1): Tensor operations and device management
- `candle-nn` (0.9.2-alpha.1): Neural network layers
- `candle-transformers` (0.9.2-alpha.1): Pre-built transformer models

**Model & Data:**
- `tokenizers` (0.21): Text tokenization for prompts
- `hf-hub` (0.3): HuggingFace Hub API client
- `image` (0.25): Image encoding/decoding

**Runtime & Utilities:**
- `tokio` (1.x): Async runtime
- `clap` (4.x): CLI argument parsing
- `tracing` (0.1): Structured logging
- `tracing-subscriber` (0.3): Log formatting
- `rand` (0.8): Random number generation

**Error Handling:**
- `anyhow` (1.0): Error context
- `thiserror` (2.0): Error derive macros

**Serialization:**
- `serde` (1.0): Serialization framework
- `serde_json` (1.0): JSON support

## Examples

### Example 1: Basic Generation

```rust
use imagent::{FluxGenerator, FluxModel, ImageGenConfig, ImageGenerator};

let mut gen = FluxGenerator::new(FluxModel::Schnell, false)?;
let config = ImageGenConfig {
    prompt: "A serene lake at sunset".into(),
    ..Default::default()
};
let img = gen.generate(&config)?;
img.save("lake.png".as_ref())?;
```

### Example 2: Custom Dimensions

```rust
use imagent::{FluxGenerator, FluxModel, ImageGenConfig, ImageGenerator};

let mut gen = FluxGenerator::new(FluxModel::Schnell, false)?;
let config = ImageGenConfig {
    prompt: "Pixel art character".into(),
    width: 512,
    height: 512,
    ..Default::default()
};
let img = gen.generate(&config)?;
img.save("character.png".as_ref())?;
```

### Example 3: Reproducible Generation

```rust
use imagent::{FluxGenerator, FluxModel, ImageGenConfig, ImageGenerator};

let mut gen = FluxGenerator::new(FluxModel::Schnell, false)?;
let config = ImageGenConfig {
    prompt: "Abstract patterns".into(),
    seed: Some(12345),
    ..Default::default()
};

// Generate the same image multiple times
let img1 = gen.generate(&config)?;
let img2 = gen.generate(&config)?;
// Both images will be identical
```

### Example 4: Error Handling

```rust
use imagent::{FluxGenerator, FluxModel, ImageGenConfig, ImageGenerator, ImageGenError};

let mut gen = FluxGenerator::new(FluxModel::Schnell, false)?;

let config = ImageGenConfig {
    prompt: "Test image".into(),
    width: 1023,  // Invalid: not multiple of 8
    height: 1024,
    ..Default::default()
};

match gen.generate(&config) {
    Ok(img) => println!("Generated: {}x{}", img.width, img.height),
    Err(ImageGenError::InvalidConfig(msg)) => {
        eprintln!("Configuration error: {}", msg);
    }
    Err(e) => eprintln!("Generation failed: {}", e),
}
```

## Development

### Building

```bash
# Build library only
cargo build -p imagent

# Build CLI tool
cargo build -p imagent --bin imagent-bin

# Build with optimizations
cargo build -p imagent --release
```

### Testing

```bash
# Run tests
cargo test -p imagent

# Test CLI
cargo run -p imagent --bin imagent-bin -- \
    --prompt "Test image" \
    --output /tmp/test.png \
    --width 512 \
    --height 512
```

### Completing the Full Implementation

The work-in-progress full Flux implementation is in `src/flux_wip/`. To complete it:

1. **Fix Candle API Compatibility**: Resolve type mismatches with the latest candle-transformers API
2. **Model Configuration**: Create proper configs for T5-XXL, CLIP-L, and Flux models
3. **HuggingFace Integration**: Test model downloading and caching
4. **Diffusion Loop**: Implement the complete denoising process
5. **GPU Optimization**: Ensure efficient CUDA kernel usage
6. **Quantization**: Add GGUF quantized model support
7. **Testing**: Validate with actual Flux model weights
8. **Documentation**: Document model-specific parameters

To switch to the full implementation once complete:
1. Uncomment `mod flux;` in `src/lib.rs`
2. Update exports to use `flux::{FluxGenerator, FluxModel}`
3. Remove or keep stub for fallback/testing

## Troubleshooting

### "Width and height must be multiples of 8"

Image dimensions must be divisible by 8 due to the model architecture. Use dimensions like 512, 768, 1024, etc.

```bash
# ❌ Invalid
--width 1000 --height 1000

# ✅ Valid
--width 1024 --height 1024
```

### Logging Output

Enable verbose logging to see generation progress:

```bash
imagent-bin --prompt "..." --verbose
```

Or set the environment variable:

```bash
RUST_LOG=imagent=debug imagent-bin --prompt "..."
```

## Roadmap

- [ ] Complete full Flux model implementation
- [ ] Add support for Flux Dev model
- [ ] Implement quantized model loading (GGUF)
- [ ] Add batch generation support
- [ ] Implement negative prompts
- [ ] Add ControlNet support
- [ ] Create async API
- [ ] Add image-to-image generation
- [ ] Support LoRA adapters
- [ ] Add web API server mode

## License

See the workspace LICENSE file for licensing information.

## Contributing

Contributions welcome! The main areas needing work:

1. Completing the full Flux implementation in `src/flux_wip/`
2. Testing with real model weights
3. Performance optimization
4. Documentation improvements
5. Example applications

## Acknowledgments

- **Black Forest Labs**: For the Flux models
- **HuggingFace**: For the Candle ML framework
- **Rust Community**: For the excellent ecosystem

## Links

- [Flux Models](https://github.com/black-forest-labs/flux)
- [Candle ML Framework](https://github.com/huggingface/candle)
- [HuggingFace Hub](https://huggingface.co/black-forest-labs)
