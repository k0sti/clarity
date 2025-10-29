// Generate endpoint example - single prompt completion
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerateOptions>,
}

#[derive(Serialize)]
struct GenerateOptions {
    temperature: f32,
    top_p: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    load_duration: u64,
    #[serde(default)]
    prompt_eval_count: i32,
    #[serde(default)]
    eval_count: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());

    let req = GenerateRequest {
        model: model.clone(),
        prompt: "Why is the sky blue? Answer in one sentence.".to_string(),
        stream: false,
        options: Some(GenerateOptions {
            temperature: 0.7,
            top_p: 0.9,
            num_predict: 100,
        }),
    };

    let resp = client
        .post("http://localhost:11434/api/generate")
        .json(&req)
        .send()
        .await?
        .json::<GenerateResponse>()
        .await?;

    println!("Model: {}\n", model);
    println!("Response: {}\n", resp.response);
    println!("Stats:");
    println!("  Prompt tokens: {}", resp.prompt_eval_count);
    println!("  Response tokens: {}", resp.eval_count);
    println!("  Total time: {}ms", resp.total_duration / 1_000_000);
    println!("  Load time: {}ms", resp.load_duration / 1_000_000);

    Ok(())
}
