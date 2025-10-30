// Model management example - list, show, pull models
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name: String,
    modified_at: String,
    size: u64,
    _digest: String,
}

#[derive(Serialize)]
struct ShowRequest {
    name: String,
}

#[derive(Deserialize)]
struct ShowResponse {
    #[allow(dead_code)]
    modelfile: String,
    #[serde(default)]
    #[allow(dead_code)]
    parameters: String,
    #[serde(default)]
    #[allow(dead_code)]
    template: String,
    #[serde(default)]
    details: ModelDetails,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Deserialize, Default)]
struct ModelDetails {
    #[serde(default)]
    format: String,
    #[serde(default)]
    family: String,
    #[serde(default)]
    parameter_size: String,
    #[serde(default)]
    quantization_level: String,
}

#[derive(Deserialize)]
struct PsResponse {
    models: Vec<LoadedModel>,
}

#[derive(Deserialize)]
struct LoadedModel {
    name: String,
    size: u64,
    #[serde(default)]
    size_vram: u64,
}

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1_073_741_824;
    const MB: u64 = 1_048_576;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // List all models
    println!("=== Available Models ===\n");
    let tags = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await?
        .json::<TagsResponse>()
        .await?;

    for model in &tags.models {
        println!("• {}", model.name);
        println!("  Size: {}", format_bytes(model.size));
        println!("  Modified: {}", model.modified_at);
        println!();
    }

    // Show details for first model
    if let Some(first) = tags.models.first() {
        println!("\n=== Model Details: {} ===\n", first.name);

        let show = client
            .post("http://localhost:11434/api/show")
            .json(&ShowRequest { name: first.name.clone() })
            .send()
            .await?
            .json::<ShowResponse>()
            .await?;

        if !show.details.family.is_empty() {
            println!("Family: {}", show.details.family);
        }
        if !show.details.parameter_size.is_empty() {
            println!("Parameters: {}", show.details.parameter_size);
        }
        if !show.details.format.is_empty() {
            println!("Format: {}", show.details.format);
        }
        if !show.details.quantization_level.is_empty() {
            println!("Quantization: {}", show.details.quantization_level);
        }
        if !show.capabilities.is_empty() {
            println!("Capabilities: {}", show.capabilities.join(", "));
        }
    }

    // List loaded models
    println!("\n=== Currently Loaded ===\n");
    let ps = client
        .get("http://localhost:11434/api/ps")
        .send()
        .await?
        .json::<PsResponse>()
        .await?;

    if ps.models.is_empty() {
        println!("No models currently loaded in memory");
    } else {
        for model in ps.models {
            println!("• {}", model.name);
            println!("  RAM: {}", format_bytes(model.size));
            if model.size_vram > 0 {
                println!("  VRAM: {}", format_bytes(model.size_vram));
            }
            println!();
        }
    }

    Ok(())
}
