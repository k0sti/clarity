use mlua::Lua;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    response: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ToolCall {
    tool: String,
    arguments: serde_json::Value,
}

async fn call_ollama(prompt: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let request = OllamaRequest {
        model: "llama3.1:8b-instruct-q4_K_M".to_string(),
        // model: "qwen2.5:latest".to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send()
        .await?;

    let response_text = response.text().await?;
    let ollama_response: OllamaResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse Ollama response: {}\nResponse: {}", e, response_text))?;

    if let Some(error) = ollama_response.error {
        return Err(format!("Ollama error: {}", error).into());
    }

    ollama_response.response
        .ok_or_else(|| "Missing response field in Ollama response".into())
}

fn write_file(path: &str, content: &str) -> Result<String, Box<dyn Error>> {
    fs::write(path, content)?;
    Ok(format!("Successfully wrote {} bytes to {}", content.len(), path))
}

fn read_file(path: &str) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

fn run_file(path: &str) -> Result<String, Box<dyn Error>> {
    let code = fs::read_to_string(path)?;
    let lua = Lua::new();

    // Override print to show output
    lua.globals().set("print", lua.create_function(|_, args: mlua::Variadic<mlua::Value>| {
        let strings: Vec<String> = args.iter()
            .map(|v| format!("{:?}", v))
            .collect();
        println!("{}", strings.join("\t"));
        Ok(())
    })?)?;

    lua.load(&code).exec()?;

    Ok(format!("Successfully executed Lua script: {}", path))
}

fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut tool_calls = Vec::new();

    // Extract JSON objects that may span multiple lines
    let mut chars = text.chars().peekable();
    let mut current_json = String::new();
    let mut brace_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut capturing = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            if capturing {
                current_json.push(ch);
            }
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            if capturing {
                current_json.push(ch);
            }
            escape_next = true;
            continue;
        }

        if ch == '"' && !escape_next {
            in_string = !in_string;
            if capturing {
                current_json.push(ch);
            }
        } else if ch == '{' && !in_string {
            if brace_depth == 0 {
                capturing = true;
                current_json.clear();
            }
            brace_depth += 1;
            current_json.push(ch);
        } else if ch == '}' && !in_string {
            if capturing {
                current_json.push(ch);
            }
            brace_depth -= 1;

            if brace_depth == 0 && capturing {
                // Try to parse the complete JSON object
                if let Ok(call) = serde_json::from_str::<ToolCall>(&current_json) {
                    tool_calls.push(call);
                }
                capturing = false;
                current_json.clear();
            }
        } else if capturing {
            current_json.push(ch);
        }
    }

    tool_calls
}

fn execute_tool(tool_call: &ToolCall) -> Result<String, Box<dyn Error>> {
    match tool_call.tool.as_str() {
        "write_file" => {
            let path = tool_call.arguments["path"].as_str()
                .ok_or("Missing 'path' argument")?;
            let content = tool_call.arguments["content"].as_str()
                .ok_or("Missing 'content' argument")?;
            write_file(path, content)
        }
        "read_file" => {
            let path = tool_call.arguments["path"].as_str()
                .ok_or("Missing 'path' argument")?;
            read_file(path)
        }
        "run_file" => {
            let path = tool_call.arguments["path"].as_str()
                .ok_or("Missing 'path' argument")?;
            run_file(path)
        }
        _ => Err(format!("Unknown tool: {}", tool_call.tool).into())
    }
}

fn get_system_prompt() -> String {
    r#"You have 3 tools. Output JSON to use them:

{"tool": "write_file", "arguments": {"path": "file.lua", "content": "code here"}}
{"tool": "read_file", "arguments": {"path": "file.lua"}}
{"tool": "run_file", "arguments": {"path": "file.lua"}}

Example: To create and run calculator.lua:
{"tool": "write_file", "arguments": {"path": "calculator.lua", "content": "print(42 + 58)"}}
{"tool": "run_file", "arguments": {"path": "calculator.lua"}}"#.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Rust + Ollama + Lua with Tools ===\n");

    let system_prompt = get_system_prompt();
    let mut conversation = system_prompt.clone();

    // Initial task
    let task = "\n\nCreate calculator.lua that prints 42+58, then run it.";
    conversation.push_str(task);

    println!("System prompt configured with tools.");
    println!("Task: {}\n", task.trim());

    let max_iterations = 10;
    for iteration in 1..=max_iterations {
        println!("=== Iteration {} ===", iteration);

        // Call LLM
        println!("Calling LLM...");
        let response = call_ollama(&conversation).await?;
        println!("LLM Response:\n{}\n", response);

        // Parse tool calls
        let tool_calls = parse_tool_calls(&response);

        if tool_calls.is_empty() {
            println!("No tool calls found. Task complete.");
            break;
        }

        // Execute tools
        let mut tool_results = String::new();
        for (i, tool_call) in tool_calls.iter().enumerate() {
            println!("Executing tool {}: {:?}", i + 1, tool_call);
            match execute_tool(tool_call) {
                Ok(result) => {
                    println!("Tool result: {}\n", result);
                    tool_results.push_str(&format!("Tool '{}' result: {}\n", tool_call.tool, result));
                }
                Err(e) => {
                    println!("Tool error: {}\n", e);
                    tool_results.push_str(&format!("Tool '{}' error: {}\n", tool_call.tool, e));
                }
            }
        }

        // Add to conversation
        conversation.push_str(&format!("\n\nAssistant: {}", response));
        conversation.push_str(&format!("\n\nTool Results:\n{}", tool_results));

        // Check if we should continue
        if iteration == max_iterations {
            println!("Max iterations reached.");
            break;
        }
    }

    println!("\n=== Done! ===");
    Ok(())
}
