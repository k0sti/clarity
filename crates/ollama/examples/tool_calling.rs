// Tool calling example - function calling with tools
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    tools: Vec<Tool>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Clone)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDef,
}

#[derive(Serialize, Clone)]
struct FunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
struct ToolCall {
    function: FunctionCall,
}

#[derive(Serialize, Deserialize, Clone)]
struct FunctionCall {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: Message,
}

fn get_weather(location: &str) -> String {
    format!("The weather in {} is sunny and 72°F", location)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());

    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: FunctionDef {
            name: "get_weather".to_string(),
            description: "Get the current weather for a location".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name"
                    }
                },
                "required": ["location"]
            }),
        },
    }];

    let mut messages = vec![Message {
        role: "user".to_string(),
        content: "What's the weather in San Francisco?".to_string(),
        tool_calls: None,
    }];

    println!("User: {}\n", messages[0].content);

    // First call - model decides to use tool
    let req = ChatRequest {
        model: model.clone(),
        messages: messages.clone(),
        tools: tools.clone(),
        stream: false,
    };

    let resp = client
        .post("http://localhost:11434/api/chat")
        .json(&req)
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    messages.push(resp.message.clone());

    // Check if model called a tool
    if let Some(tool_calls) = &resp.message.tool_calls {
        for call in tool_calls {
            println!("Assistant called tool: {}", call.function.name);
            println!("Arguments: {}\n", call.function.arguments);

            // Execute the tool
            let location = call.function.arguments["location"].as_str().unwrap();
            let result = get_weather(location);

            println!("Tool result: {}\n", result);

            // Add tool result to messages
            messages.push(Message {
                role: "tool".to_string(),
                content: result,
                tool_calls: None,
            });

            // Second call - model uses tool result
            let req2 = ChatRequest {
                model: model.clone(),
                messages: messages.clone(),
                tools: tools.clone(),
                stream: false,
            };

            let resp2 = client
                .post("http://localhost:11434/api/chat")
                .json(&req2)
                .send()
                .await?
                .json::<ChatResponse>()
                .await?;

            println!("Assistant: {}", resp2.message.content);
        }
    } else {
        println!("Assistant: {}", resp.message.content);
    }

    Ok(())
}
