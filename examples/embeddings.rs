// Embeddings example - generate vector representations
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (mag_a * mag_b)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let model = "nomic-embed-text".to_string();

    let texts = vec![
        "The cat sat on the mat".to_string(),
        "A feline rested on the rug".to_string(),
        "The weather is sunny today".to_string(),
    ];

    let req = EmbedRequest {
        model: model.clone(),
        input: texts.clone(),
    };

    let resp = client
        .post("http://localhost:11434/api/embed")
        .json(&req)
        .send()
        .await?
        .json::<EmbedResponse>()
        .await?;

    println!("Model: {}\n", model);
    println!("Generated {} embeddings\n", resp.embeddings.len());
    println!("Embedding dimension: {}\n", resp.embeddings[0].len());

    println!("Similarity scores:");
    println!("  '{}' <-> '{}': {:.4}",
        texts[0], texts[1],
        cosine_similarity(&resp.embeddings[0], &resp.embeddings[1]));
    println!("  '{}' <-> '{}': {:.4}",
        texts[0], texts[2],
        cosine_similarity(&resp.embeddings[0], &resp.embeddings[2]));
    println!("  '{}' <-> '{}': {:.4}",
        texts[1], texts[2],
        cosine_similarity(&resp.embeddings[1], &resp.embeddings[2]));

    Ok(())
}
