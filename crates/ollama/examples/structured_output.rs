// Structured output example - JSON schema validation
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    format: serde_json::Value,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize, Debug)]
struct PersonInfo {
    name: String,
    age: u32,
    city: String,
    occupation: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());

    // Define JSON schema for structured output
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" },
            "city": { "type": "string" },
            "occupation": { "type": "string" }
        },
        "required": ["name", "age", "city", "occupation"]
    });

    let req = ChatRequest {
        model: model.clone(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Generate a random person's information.".to_string(),
        }],
        stream: false,
        format: schema,
    };

    println!("Requesting structured JSON output...\n");

    let resp = client
        .post("http://localhost:11434/api/chat")
        .json(&req)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    // Parse structured response
    let person: PersonInfo = serde_json::from_str(&resp.message.content)?;

    println!("Generated Person:");
    println!("  Name: {}", person.name);
    println!("  Age: {}", person.age);
    println!("  City: {}", person.city);
    println!("  Occupation: {}", person.occupation);

    Ok(())
}
