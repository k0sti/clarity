# imagent

AI Image Generation Library and CLI for Rust using Stable Diffusion with Candle.

## Overview

`imagent` is a Rust library and command-line tool for generating images using Stable Diffusion models, powered by the Candle ML framework from HuggingFace. It provides a clean, trait-based interface for image generation with support for both library integration and standalone CLI usage.

## Current Implementation Status

**Implemented Models**:
- ✅ **Stable Diffusion v1.5** - Classic SD, fast and reliable
- ✅ **Stable Diffusion v2.1** - Improved quality
- ⏳ **Stable Diffusion XL** - Requires dual text encoders (not yet implemented)
- ⏳ **SD-Turbo** - Requires dual text encoders (not yet implemented)
- 🔨 **Flux Models** - Stub implementation only

**Working Features**:
- ✅ Complete diffusion pipeline (CLIP → UNet → VAE)
- ✅ Classifier-free guidance
- ✅ HuggingFace Hub integration with automatic model downloads
- ✅ CPU mode (tested and working)
- ⏳ GPU/CUDA support (requires CUDA toolkit with nvcc)
- ✅ Configurable image size and inference steps
- ✅ Reproducible generation with seeds

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
imagent = { path = "../crates/imagent" }
```

Or build the CLI tool:

```bash
# CPU-only (works out of the box)
cargo build --release --bin imagent-bin

# With CUDA support (requires CUDA toolkit installed)
cargo build --release --features cuda --bin imagent-bin
```

## Quick Start

### CLI Usage

```bash
# Basic usage (256x256, 10 steps, CPU, SD v1.5)
cargo run --bin imagent-bin -- --prompt "A robot in a forest"

# Custom size and steps
cargo run --bin imagent-bin -- --prompt "Sunset over mountains" \
    --width 512 --height 512 -n 20

# Use SD v2.1
cargo run --bin imagent-bin -- --prompt "Abstract art" -m sd-v21

# Specify output and seed
cargo run --bin imagent-bin -- --prompt "Cat wearing a hat" \
    -o my_cat.png --seed 42
```

### Library Usage

```rust
use imagent::{ImageGenConfig, ImageGenerator, StableDiffusionGenerator, StableDiffusionVersion};

fn main() -> imagent::Result<()> {
    // Create generator
    let mut generator = StableDiffusionGenerator::new(
        StableDiffusionVersion::V1_5,
        true, // use_cpu
    )?;

    // Configure generation
    let config = ImageGenConfig {
        prompt: "A beautiful landscape".to_string(),
        width: 512,
        height: 512,
        num_steps: 20,
        seed: None,
        quantized: false,
        use_cpu: true,
    };

    // Generate and save
    let image = generator.generate(&config)?;
    image.save("output.png")?;

    println!("Generated with seed: {}", image.seed);
    Ok(())
}
```

## CLI Options

```
Options:
  -p, --prompt <PROMPT>          Text prompt describing the image
  -o, --output <OUTPUT>          Output file path [default: output.png]
  -w, --width <WIDTH>            Image width (multiple of 8) [default: 256]
      --height <HEIGHT>          Image height (multiple of 8) [default: 256]
  -n, --num-steps <NUM_STEPS>    Inference steps [default: 10 for SD v1.5/v2.1]
  -s, --seed <SEED>              Random seed for reproducibility
  -m, --model <MODEL>            Model variant [default: sd-v15]
                                 Options: sd-v15, sd-v21, sd-xl*, sd-turbo*
                                 (* = not yet supported, requires dual encoders)
  -q, --quantized                Use quantized models (not yet supported)
      --cpu                      Force CPU usage
  -v, --verbose                  Verbose logging
```

## Performance Tips

### Fast Testing (Quick Results)
```bash
# Smallest/fastest settings
cargo run --bin imagent-bin -- --prompt "Test" --width 256 --height 256 -n 5
```

### Quality Generation
```bash
# Higher resolution and more steps
cargo run --bin imagent-bin -- --prompt "Detailed scene" \
    --width 512 --height 512 -n 30
```

### First Run Notes
- First run downloads model weights (~3.4GB for SD v1.5)
- Models are cached in `~/.cache/huggingface/hub/`
- Subsequent runs are much faster (no downloads)
- Generation time on CPU:
  - 256x256, 10 steps: ~30-60 seconds
  - 512x512, 20 steps: ~2-4 minutes

## GPU/CUDA Support

### Requirements
- NVIDIA GPU with CUDA support
- CUDA Toolkit 12.x with `nvcc` compiler
- For Nix users: CUDA packages are split and may need manual configuration

### Building with CUDA
```bash
cargo build --release --features cuda --bin imagent-bin
```

### Known Issues with Nix
The Nix CUDA packages are split across multiple store paths, which can cause build issues with `candle-kernels`. CPU mode works reliably on all systems.

## Model Information

### Stable Diffusion v1.5
- **Size**: ~3.4GB
- **Default steps**: 10
- **Recommended resolution**: 512x512
- **Guidance scale**: 7.5
- **Status**: ✅ Fully working

### Stable Diffusion v2.1
- **Size**: ~3.4GB
- **Default steps**: 10
- **Recommended resolution**: 768x768
- **Guidance scale**: 7.5
- **Status**: ✅ Fully working

### Stable Diffusion XL
- **Status**: ❌ Not yet supported
- **Reason**: Requires dual text encoders (CLIP-L + OpenCLIP-G)
- **Error**: Shape mismatch in attention layers
- **Future**: Will be implemented with dual encoder support

### SD-Turbo
- **Status**: ❌ Not yet supported (same reason as SDXL)

## Architecture

The library uses a trait-based design:

```rust
pub trait ImageGenerator {
    fn generate(&mut self, config: &ImageGenConfig) -> Result<GeneratedImage>;
}
```

Implementations:
- `StableDiffusionGenerator` - Full SD pipeline
- `FluxGenerator` - Stub implementation (placeholder)

## Troubleshooting

### Error: "SDXL and SD-Turbo are not yet supported"
Use `-m sd-v15` or `-m sd-v21` instead. SDXL requires dual text encoders which aren't implemented yet.

### Slow generation on CPU
This is expected. Try:
- Reduce image size: `--width 256 --height 256`
- Reduce steps: `-n 5` or `-n 10`
- Use GPU if available (requires CUDA build)

### Out of memory
- Reduce image size
- Use CPU mode instead of GPU: `--cpu`
- Close other applications

### Models not downloading
- Check internet connection
- Ensure `~/.cache/huggingface/hub/` is writable
- Some models may require HuggingFace authentication (not currently implemented)

## Development Status

- [x] Basic Stable Diffusion pipeline
- [x] SD v1.5 and v2.1 support
- [x] HuggingFace Hub integration
- [x] CPU mode
- [ ] SDXL dual encoder support
- [ ] GPU/CUDA optimization
- [ ] Flux model implementation
- [ ] Quantized model support
- [ ] Negative prompts
- [ ] Image-to-image generation
- [ ] Inpainting

## License

See workspace license.

## Credits

- Powered by [Candle](https://github.com/huggingface/candle) ML framework
- Uses models from [Stability AI](https://stability.ai/) and [HuggingFace](https://huggingface.co/)
