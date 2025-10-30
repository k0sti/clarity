use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use termimad::{MadSkin, crossterm::style::Color};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: Message,
    #[serde(default)]
    _done: bool,
    #[serde(default)]
    _total_duration: u64,
    #[serde(default)]
    _prompt_eval_count: i32,
    #[serde(default)]
    _eval_count: i32,
}

#[derive(Serialize)]
struct ShowRequest {
    name: String,
}

#[derive(Deserialize)]
struct ShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    details: ModelDetails,
}

#[derive(Deserialize, Default)]
struct ModelDetails {
    #[serde(default)]
    parameter_size: String,
    #[serde(default)]
    _quantization_level: String,
    #[serde(default)]
    family: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name: String,
    #[serde(default)]
    _size: u64,
}

async fn get_model_info(client: &reqwest::Client, model: &str) -> Result<ShowResponse, Box<dyn std::error::Error>> {
    let resp = client
        .post("http://localhost:11434/api/show")
        .json(&ShowRequest { name: model.to_string() })
        .send()
        .await?
        .json::<ShowResponse>()
        .await?;
    Ok(resp)
}

async fn list_models(client: &reqwest::Client) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let resp = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await?
        .json::<TagsResponse>()
        .await?;
    Ok(resp.models.into_iter().map(|m| m.name).collect())
}

fn create_markdown_skin() -> MadSkin {
    let mut skin = MadSkin::default();

    // Headers
    skin.headers[0].set_fg(Color::Cyan);
    skin.headers[1].set_fg(Color::Blue);
    skin.headers[2].set_fg(Color::Green);

    // Code blocks
    skin.code_block.set_fg(Color::Yellow);
    skin.inline_code.set_fg(Color::Yellow);

    // Bold and italic
    skin.bold.set_fg(Color::White);
    skin.italic.set_fg(Color::Magenta);

    skin
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Clarity v{} - Ollama Chat\n", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());

    // Show model info
    if let Ok(info) = get_model_info(&client, &model).await {
        println!("Model: {}", model);
        if !info.details.parameter_size.is_empty() {
            println!("  Size: {}", info.details.parameter_size);
        }
        if !info.details.family.is_empty() {
            println!("  Family: {}", info.details.family);
        }
        if !info.capabilities.is_empty() {
            println!("  Capabilities: {}", info.capabilities.join(", "));
            if info.capabilities.contains(&"tools".to_string()) {
                println!("  ✓ Tool calling supported");
            }
        }
        println!();
    }

    let mut history = Vec::new();
    let skin = create_markdown_skin();
    println!("Commands: /models, /clear, exit\n");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() || input == "exit" || input == "quit" {
            break;
        }

        match input {
            "/models" => {
                if let Ok(models) = list_models(&client).await {
                    println!("\nAvailable models:");
                    for m in models {
                        println!("  • {}", m);
                    }
                    println!();
                }
                continue;
            }
            "/clear" => {
                history.clear();
                println!("History cleared.\n");
                continue;
            }
            _ => {}
        }

        history.push(Message {
            role: "user".to_string(),
            content: input.to_string(),
        });

        let req = ChatRequest {
            model: model.clone(),
            messages: history.clone(),
            stream: false,
        };

        let resp = client
            .post("http://localhost:11434/api/chat")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            eprintln!("Error: API returned status {}", status);
            eprintln!("Response: {}", text);
            continue;
        }

        let chat_resp: ChatResponse = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to parse response: {}", e);
                eprintln!("Raw response: {}", text);
                continue;
            }
        };

        println!(); // Empty line before response
        skin.print_text(&chat_resp.message.content);
        println!(); // Empty line after response
        history.push(chat_resp.message);
    }

    println!("Goodbye!");
    Ok(())
}
