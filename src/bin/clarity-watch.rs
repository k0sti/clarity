use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct WatchConfig {
    /// Directories to watch for changes
    watch_dirs: Vec<String>,

    /// File patterns to ignore (glob patterns)
    #[serde(default)]
    ignore_patterns: Vec<String>,

    /// Debounce delay in milliseconds
    #[serde(default = "default_debounce")]
    debounce_ms: u64,

    /// Whether to watch hidden files/directories
    #[serde(default)]
    watch_hidden: bool,

    /// Ollama API endpoint
    #[serde(default = "default_ollama_endpoint")]
    ollama_endpoint: String,

    /// Ollama model to use for orchestration
    #[serde(default = "default_ollama_model")]
    ollama_model: String,

    /// Disable thinking tokens if supported by model
    #[serde(default)]
    disable_thinking: bool,
}

#[derive(Debug, Serialize)]
struct FileDispatch {
    file_path: String,
    file_type: String,
    file_size: u64,
    modified_timestamp: String,
    modified_unix: u64,
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    thinking: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Deserialize)]
struct StreamResponse {
    message: OllamaMessage,
    done: bool,
}

fn default_debounce() -> u64 {
    200
}

fn default_ollama_endpoint() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string())
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            watch_dirs: vec![".".to_string()],
            ignore_patterns: vec![
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.clarity.json".to_string(),
            ],
            debounce_ms: 200,
            watch_hidden: false,
            ollama_endpoint: default_ollama_endpoint(),
            ollama_model: default_ollama_model(),
            disable_thinking: false,
        }
    }
}

fn find_config() -> Option<PathBuf> {
    // Try current directory first
    let local_config = PathBuf::from(".clarity.json");
    if local_config.exists() {
        return Some(local_config);
    }

    // Try home directory
    if let Some(home) = dirs::home_dir() {
        let home_config = home.join(".clarity.json");
        if home_config.exists() {
            return Some(home_config);
        }
    }

    None
}

fn load_config() -> Result<WatchConfig, Box<dyn std::error::Error>> {
    if let Some(config_path) = find_config() {
        println!("📝 Loading config from: {}", config_path.display());
        let contents = fs::read_to_string(&config_path)?;
        let config: WatchConfig = serde_json::from_str(&contents)?;
        Ok(config)
    } else {
        println!("⚠️  No .clarity.json found, using default config");
        println!("   Create .clarity.json in your project or home directory to customize");
        Ok(WatchConfig::default())
    }
}

fn create_example_config(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let example = WatchConfig {
        watch_dirs: vec![
            "src".to_string(),
            "examples".to_string(),
        ],
        ignore_patterns: vec![
            "**/.git/**".to_string(),
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/*.swp".to_string(),
            "**/*.tmp".to_string(),
        ],
        debounce_ms: 200,
        watch_hidden: false,
        ollama_endpoint: default_ollama_endpoint(),
        ollama_model: default_ollama_model(),
        disable_thinking: true,
    };

    let json = serde_json::to_string_pretty(&example)?;
    fs::write(path, json)?;
    println!("✅ Created example config at: {}", path.display());
    Ok(())
}

fn should_ignore(path: &Path, patterns: &[String], watch_hidden: bool) -> bool {
    let path_str = path.to_string_lossy();

    // Check if path contains hidden components
    if !watch_hidden {
        if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().starts_with('.') {
                return true;
            }
        }
        // Check parent directories for hidden components
        for component in path.components() {
            if let Some(comp_str) = component.as_os_str().to_str() {
                if comp_str.starts_with('.') && comp_str != "." {
                    return true;
                }
            }
        }
    }

    // Check ignore patterns
    for pattern in patterns {
        if glob_match::glob_match(pattern, &path_str) {
            return true;
        }
    }

    false
}

fn format_event_kind(kind: &notify::EventKind) -> &str {
    use notify::EventKind;
    match kind {
        EventKind::Create(_) => "📄 Created",
        EventKind::Modify(_) => "✏️  Modified",
        EventKind::Remove(_) => "🗑️  Removed",
        EventKind::Access(_) => "👁️  Accessed",
        _ => "📌 Changed",
    }
}

fn determine_file_type(path: &Path) -> String {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            // Documents
            "txt" | "md" | "markdown" => "text/plain",
            "pdf" => "application/pdf",
            "doc" | "docx" => "application/msword",
            "json" => "application/json",
            "xml" => "application/xml",
            "yaml" | "yml" => "application/yaml",

            // Audio
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "flac" => "audio/flac",
            "ogg" => "audio/ogg",
            "m4a" => "audio/m4a",
            "aac" => "audio/aac",

            // Video
            "mp4" => "video/mp4",
            "avi" => "video/avi",
            "mkv" => "video/mkv",
            "mov" => "video/quicktime",
            "webm" => "video/webm",

            // Images
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",

            // Code
            "rs" => "text/rust",
            "py" => "text/python",
            "js" => "text/javascript",
            "ts" => "text/typescript",
            "go" => "text/go",
            "c" | "h" => "text/c",
            "cpp" | "hpp" => "text/cpp",
            "java" => "text/java",

            // Archives
            "zip" => "application/zip",
            "tar" => "application/tar",
            "gz" => "application/gzip",
            "7z" => "application/x-7z-compressed",

            _ => "application/octet-stream",
        }.to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

fn find_last_modified_file(dirs: &[String], ignore_patterns: &[String], watch_hidden: bool) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let mut latest_file: Option<(PathBuf, SystemTime)> = None;

    for dir_str in dirs {
        let dir = PathBuf::from(dir_str);
        if !dir.exists() {
            continue;
        }

        visit_dir(&dir, ignore_patterns, watch_hidden, &mut latest_file)?;
    }

    Ok(latest_file.map(|(path, _)| path))
}

fn visit_dir(
    dir: &Path,
    ignore_patterns: &[String],
    watch_hidden: bool,
    latest_file: &mut Option<(PathBuf, SystemTime)>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if should_ignore(&path, ignore_patterns, watch_hidden) {
            continue;
        }

        if path.is_dir() {
            visit_dir(&path, ignore_patterns, watch_hidden, latest_file)?;
        } else if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Some((_, latest_time)) = latest_file {
                        if modified > *latest_time {
                            *latest_file = Some((path, modified));
                        }
                    } else {
                        *latest_file = Some((path, modified));
                    }
                }
            }
        }
    }

    Ok(())
}

async fn dispatch_to_orchestrator(
    file_dispatch: &FileDispatch,
    config: &WatchConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::StreamExt;
    use std::io::Write;

    println!("\n🚀 Dispatching to orchestrator...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📁 File: {}", file_dispatch.file_path);
    println!("📋 Type: {}", file_dispatch.file_type);
    println!("📏 Size: {} bytes", file_dispatch.file_size);
    println!("🕐 Modified: {}", file_dispatch.modified_timestamp);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Use a long timeout for streaming (10 minutes) but with read timeout for activity
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))  // Total timeout: 10 minutes
        .read_timeout(std::time::Duration::from_secs(60))  // Read timeout: 60s between chunks
        .build()?;

    let endpoint = format!("{}/api/chat", config.ollama_endpoint);

    // First, check if Ollama is responding
    println!("🔌 Checking Ollama connection...");
    match client.get(&format!("{}/api/tags", config.ollama_endpoint)).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("✅ Ollama is responding\n");
            } else {
                println!("⚠️  Ollama returned status: {}\n", resp.status());
            }
        }
        Err(e) => {
            eprintln!("❌ Cannot connect to Ollama: {}", e);
            eprintln!("   Make sure Ollama is running: ollama serve");
            return Err(e.into());
        }
    }

    // System prompt for the orchestrator
    let mut system_prompt = r#"You are given metadata of a file: File's path, type, size, and modification timestamp.
    File contains notes, instructions or tasks to do.
    What tool calls should be performed to extract and process the file contents.
    1. Identify the appropriate tools or methods to read/process the file based on its type
    2. Describe operations needed (e.g., transcription for audio, OCR for images, parsing for documents)
Provide answer as function definitions of the following format:
FunctionDef {
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
}"#.to_string();

    if config.disable_thinking {
        system_prompt.push_str("\nProvide direct, concise answers without showing your reasoning process.");
    }

    // Create user message with file info
    let user_message = format!(
        r#"File Path: {}
File Type: {}
File Size: {} bytes"#,
        file_dispatch.file_path,
        file_dispatch.file_type,
        file_dispatch.file_size
    );

    let request = OllamaRequest {
        model: config.ollama_model.clone(),
        messages: vec![
            OllamaMessage {
                role: "system".to_string(),
                content: system_prompt,
                thinking: String::new(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: user_message,
                thinking: String::new(),
            },
        ],
        stream: true,
        options: Some(serde_json::json!({
            "num_predict": -1,
            "temperature": 0.7,
        })),
    };

    if config.disable_thinking {
        println!("⚙️  Thinking disabled (via prompt)");
    }

    println!("📡 Sending to: {}", endpoint);
    println!("🤖 Model: {}", config.ollama_model);
    println!("⏳ Waiting for response (streaming)...\n");

    let response = client
        .post(&endpoint)
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        eprintln!("❌ Orchestrator error: {}", error_text);
        return Err(format!("API error: {}", error_text).into());
    }

    println!("💬 Orchestrator response:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        buffer.extend_from_slice(&bytes);

        while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<_>>();
            if let Ok(line) = std::str::from_utf8(&line_bytes) {
                let line = line.trim();
                if !line.is_empty() {
                    if let Ok(parsed) = serde_json::from_str::<StreamResponse>(line) {
                        // Print content
                        if !parsed.message.content.is_empty() {
                            print!("{}", parsed.message.content);
                            std::io::stdout().flush()?;
                        }
                        // Print thinking in dim gray
                        if !parsed.message.thinking.is_empty() {
                            print!("\x1b[2m{}\x1b[0m", parsed.message.thinking);
                            std::io::stdout().flush()?;
                        }
                    }
                }
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--init" => {
                let path = PathBuf::from(".clarity.json");
                if path.exists() {
                    eprintln!("❌ .clarity.json already exists");
                    std::process::exit(1);
                }
                create_example_config(&path)?;
                return Ok(());
            }
            "--last" => {
                let config = load_config()?;

                println!("🔍 Searching for last modified file...\n");

                match find_last_modified_file(&config.watch_dirs, &config.ignore_patterns, config.watch_hidden)? {
                    Some(file_path) => {
                        let metadata = fs::metadata(&file_path)?;
                        let modified = metadata.modified()?;
                        let modified_unix = modified.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
                        let modified_timestamp = chrono::DateTime::<chrono::Local>::from(modified)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string();

                        // Calculate relative path from watched directory
                        let relative_path = config.watch_dirs.iter()
                            .find_map(|watch_dir| {
                                let watch_path = PathBuf::from(watch_dir);
                                file_path.strip_prefix(&watch_path).ok()
                                    .map(|p| p.display().to_string())
                            })
                            .unwrap_or_else(|| file_path.display().to_string());

                        let file_dispatch = FileDispatch {
                            file_path: relative_path,
                            file_type: determine_file_type(&file_path),
                            file_size: metadata.len(),
                            modified_timestamp,
                            modified_unix,
                        };

                        dispatch_to_orchestrator(&file_dispatch, &config).await?;
                    }
                    None => {
                        println!("⚠️  No files found in watched directories");
                    }
                }

                return Ok(());
            }
            "--help" | "-h" => {
                println!("clarity-watch - File change watcher for Clarity");
                println!();
                println!("USAGE:");
                println!("    clarity-watch [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    --init     Create example .clarity.json config");
                println!("    --last     Find and dispatch last modified file to orchestrator");
                println!("    --help     Show this help message");
                println!();
                println!("CONFIGURATION:");
                println!("    Place .clarity.json in your project directory or home directory");
                println!();
                println!("Example .clarity.json:");
                println!("{}", serde_json::to_string_pretty(&WatchConfig::default())?);
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    let config = load_config()?;

    println!("\n🔍 Clarity File Watcher");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Watching directories:");
    for dir in &config.watch_dirs {
        let path = PathBuf::from(dir);
        if path.exists() {
            println!("  ✓ {}", dir);
        } else {
            println!("  ✗ {} (not found)", dir);
        }
    }

    if !config.ignore_patterns.is_empty() {
        println!("\nIgnoring patterns:");
        for pattern in &config.ignore_patterns {
            println!("  • {}", pattern);
        }
    }

    println!("\nDebounce: {}ms", config.debounce_ms);
    println!("Watch hidden files: {}", config.watch_hidden);
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Watching for changes... (Press Ctrl+C to stop)\n");

    // Create debouncer
    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(
        Duration::from_millis(config.debounce_ms),
        None,
        move |result: DebounceEventResult| {
            tx.send(result).unwrap();
        },
    )?;

    // Add paths to watcher
    for dir in &config.watch_dirs {
        let path = PathBuf::from(dir);
        if path.exists() {
            debouncer.watch(&path, RecursiveMode::Recursive)?;
        }
    }

    // Process events
    for result in rx {
        match result {
            Ok(events) => {
                for event in events {
                    for path in &event.event.paths {
                        if should_ignore(path, &config.ignore_patterns, config.watch_hidden) {
                            continue;
                        }

                        let timestamp = chrono::Local::now().format("%H:%M:%S");
                        let event_type = format_event_kind(&event.event.kind);

                        println!(
                            "[{}] {} {}",
                            timestamp,
                            event_type,
                            path.display()
                        );
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    eprintln!("⚠️  Watch error: {}", error);
                }
            }
        }
    }

    Ok(())
}
