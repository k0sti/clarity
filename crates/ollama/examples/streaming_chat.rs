// Streaming chat example - shows responses token by token
use serde::{Deserialize, Serialize};
use termimad::{MadSkin, crossterm::style::Color};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct StreamResponse {
    message: Message,
    #[allow(dead_code)]
    done: bool,
}

fn create_markdown_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.headers[0].set_fg(Color::Cyan);
    skin.headers[1].set_fg(Color::Blue);
    skin.headers[2].set_fg(Color::Green);
    skin.code_block.set_fg(Color::Yellow);
    skin.inline_code.set_fg(Color::Yellow);
    skin.bold.set_fg(Color::White);
    skin.italic.set_fg(Color::Magenta);
    skin
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());

    let req = ChatRequest {
        model,
        messages: vec![Message {
            role: "user".to_string(),
            content: "Explain streaming in 2 sentences.".to_string(),
        }],
        stream: true,
    };

    use futures_util::StreamExt;

    let resp = client
        .post("http://localhost:11434/api/chat")
        .json(&req)
        .send()
        .await?;

    let mut stream = resp.bytes_stream();
    let mut buffer = Vec::new();
    let mut full_response = String::new();
    let skin = create_markdown_skin();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        buffer.extend_from_slice(&bytes);

        while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<_>>();
            if let Ok(line) = std::str::from_utf8(&line_bytes) {
                let line = line.trim();
                if !line.is_empty() {
                    if let Ok(parsed) = serde_json::from_str::<StreamResponse>(line) {
                        full_response.push_str(&parsed.message.content);
                        print!("{}", parsed.message.content);
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                }
            }
        }
    }

    // Re-render with markdown formatting
    print!("\r");  // Return to start
    print!("\x1B[2K"); // Clear line
    println!();
    skin.print_text(&full_response);
    println!();
    Ok(())
}
