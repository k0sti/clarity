// imagent-bin - CLI for generating images with AI models

use clap::Parser;
use imagent::{
    FluxGenerator, FluxModel, ImageGenConfig, ImageGenerator, Result, StableDiffusionGenerator,
    StableDiffusionVersion,
};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about = "Generate images using AI models (Flux, Stable Diffusion)", long_about = None)]
struct Args {
    /// Text prompt describing the image to generate
    #[arg(short, long)]
    prompt: String,

    /// Output file path (PNG format)
    #[arg(short, long, default_value = "output.png")]
    output: PathBuf,

    /// Image width (must be multiple of 8)
    #[arg(short, long, default_value = "256")]
    width: usize,

    /// Image height (must be multiple of 8)
    #[arg(long, default_value = "256")]
    height: usize,

    /// Number of inference steps
    #[arg(short, long)]
    num_steps: Option<usize>,

    /// Random seed for reproducibility
    #[arg(short, long)]
    seed: Option<u64>,

    /// Model variant to use
    #[arg(short, long, value_enum, default_value = "sd-v15")]
    model: ModelVariant,

    /// Use quantized models (faster, less memory)
    #[arg(short, long)]
    quantized: bool,

    /// Force CPU usage (default: use GPU if available)
    #[arg(long)]
    cpu: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ModelVariant {
    /// Flux Schnell - 4 steps, faster
    FluxSchnell,
    /// Flux Dev - 50 steps, higher quality
    FluxDev,
    /// Stable Diffusion v1.5 - Classic, 30 steps
    SdV15,
    /// Stable Diffusion v2.1 - Improved, 30 steps
    SdV21,
    /// Stable Diffusion XL - High quality, 30 steps
    SdXl,
    /// Stable Diffusion Turbo - Fastest, 1 step
    SdTurbo,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("imagent={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting imagent-bin");
    tracing::info!("Prompt: {}", args.prompt);
    tracing::info!("Output: {}", args.output.display());
    tracing::info!("Model: {:?}", args.model);

    // Create generator based on model type
    let (mut generator, default_steps): (Box<dyn ImageGenerator>, usize) = match args.model {
        ModelVariant::FluxSchnell => {
            let gen = FluxGenerator::new(FluxModel::Schnell, args.cpu)?;
            (Box::new(gen), FluxModel::Schnell.default_steps())
        }
        ModelVariant::FluxDev => {
            let gen = FluxGenerator::new(FluxModel::Dev, args.cpu)?;
            (Box::new(gen), FluxModel::Dev.default_steps())
        }
        ModelVariant::SdV15 => {
            let gen = StableDiffusionGenerator::new(StableDiffusionVersion::V1_5, args.cpu)?;
            (Box::new(gen), StableDiffusionVersion::V1_5.default_steps())
        }
        ModelVariant::SdV21 => {
            let gen = StableDiffusionGenerator::new(StableDiffusionVersion::V2_1, args.cpu)?;
            (Box::new(gen), StableDiffusionVersion::V2_1.default_steps())
        }
        ModelVariant::SdXl => {
            let gen = StableDiffusionGenerator::new(StableDiffusionVersion::Xl, args.cpu)?;
            (Box::new(gen), StableDiffusionVersion::Xl.default_steps())
        }
        ModelVariant::SdTurbo => {
            let gen = StableDiffusionGenerator::new(StableDiffusionVersion::Turbo, args.cpu)?;
            (Box::new(gen), StableDiffusionVersion::Turbo.default_steps())
        }
    };

    // Determine num_steps
    let num_steps = args.num_steps.unwrap_or(default_steps);

    // Create configuration
    let config = ImageGenConfig {
        prompt: args.prompt,
        width: args.width,
        height: args.height,
        num_steps,
        seed: args.seed,
        quantized: args.quantized,
        use_cpu: args.cpu,
    };

    // Generate image
    tracing::info!("Generating image...");
    let image = generator.generate(&config)?;

    // Save image
    tracing::info!("Saving image to: {}", args.output.display());
    image.save(&args.output)?;

    tracing::info!("Image generated successfully!");
    tracing::info!("Seed used: {}", image.seed);

    println!("Image saved to: {}", args.output.display());
    println!("Seed: {}", image.seed);

    Ok(())
}
