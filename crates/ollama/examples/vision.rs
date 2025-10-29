// Vision/multimodal example - analyze images with vision models
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Use a vision model like llava
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llava".to_string());

    // Get image path from args or use example
    let image_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/path/to/image.jpg".to_string());

    println!("Model: {}", model);
    println!("Image: {}\n", image_path);

    // Read image and encode to base64
    use base64::Engine;
    let image_data = std::fs::read(&image_path)?;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

    let req = ChatRequest {
        model: model.clone(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "What do you see in this image? Describe it in detail.".to_string(),
            images: Some(vec![base64_image]),
        }],
        stream: false,
    };

    println!("Analyzing image...\n");

    let resp = client
        .post("http://localhost:11434/api/chat")
        .json(&req)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    println!("Analysis:\n{}", resp.message.content);

    Ok(())
}
