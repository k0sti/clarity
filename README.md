# Clarity

Interactive terminal AI chat client for Ollama with comprehensive API examples.

## Features

- 💬 Interactive chat with conversation history
- 🎨 Beautiful markdown rendering with syntax highlighting
- 🔍 File change watcher with configurable patterns
- 🤖 **AI Orchestration System** - intelligent routing to specialized experts
- 🔧 Model information and capabilities detection
- 🛠️ Tool calling support indicator
- 📝 Comprehensive examples for all Ollama API endpoints

## Quick Start

```bash
# Using just (recommended)
just chat

# With custom model
just chat-model llama3.1

# Or using cargo directly
cargo run --bin clarity-chat

# With custom model via cargo
OLLAMA_MODEL=gpt-oss:20b cargo run --bin clarity-chat
```

> **Tip:** Run `just --list` to see all available commands

### Interactive Commands

- `/models` - List available models
- `/clear` - Clear conversation history
- `exit` or `quit` - Exit the application

### Markdown Formatting

Clarity renders chat responses with beautiful colored markdown formatting:

- **Headers** - Cyan, Blue, and Green for H1-H3
- **Code blocks** - Yellow syntax highlighting
- **Inline code** - Yellow highlighting
- **Bold text** - White emphasis
- **Italic text** - Magenta styling

Try asking for code examples or formatted responses to see the markdown rendering in action!

### File Watcher

Monitor file changes in your project directories with `clarity-watch`:

```bash
# Initialize .clarity.json configuration
just watch-init

# Start watching for file changes
just watch-files

# Find and dispatch last modified file to AI orchestrator
just watch-last
```

**Configuration** (`.clarity.json`):
```json
{
  "watch_dirs": ["src", "examples"],
  "ignore_patterns": [
    "**/.git/**",
    "**/target/**",
    "**/*.swp"
  ],
  "debounce_ms": 200,
  "watch_hidden": false,
  "ollama_endpoint": "http://localhost:11434",
  "ollama_model": "gpt-oss:20b"
}
```

**Configuration Options:**
- `watch_dirs` - Array of directories to monitor recursively
- `ignore_patterns` - Glob patterns for files/dirs to ignore
- `debounce_ms` - Milliseconds to wait before reporting changes (reduces duplicate events)
- `watch_hidden` - Whether to monitor hidden files/directories
- `ollama_endpoint` - Ollama API endpoint for orchestrator (default: http://localhost:11434)
- `ollama_model` - Model to use for file analysis (default: gpt-oss:20b)

The watcher looks for `.clarity.json` in the current directory first, then falls back to your home directory. This allows you to have project-specific and global configurations.

**Events Tracked:**
- 📄 File creation
- ✏️  File modifications
- 🗑️  File deletions
- 👁️  File access

**Orchestrator Integration:**

The `--last` flag finds the most recently modified file in your watched directories and dispatches it to an AI orchestrator (Ollama) for analysis. The orchestrator receives:

- File path (relative to watched directory)
- File type (automatically detected from extension)
- File size
- Modification timestamp

Supported file types include:
- **Documents**: txt, md, pdf, json, yaml, xml
- **Audio**: mp3, wav, flac, ogg, m4a, aac
- **Video**: mp4, avi, mkv, mov, webm
- **Images**: jpg, png, gif, svg, webp
- **Code**: rs, py, js, ts, go, c, cpp, java
- **Archives**: zip, tar, gz, 7z

**Features:**
- ✅ Streaming responses - see LLM output token-by-token (including reasoning)
- ✅ Connection checking - verifies Ollama is running before sending
- ✅ Long timeout (10 minutes) to handle large model responses
- ✅ System prompt guides the AI to recommend specific operations/tools

**Note:** Larger models like `gpt-oss:20b` may take 1-2 minutes to generate complete responses. You'll see the model's reasoning process (in gray) as it thinks through the problem before providing the final answer.

The AI model analyzes the file metadata and recommends processing steps (e.g., "transcribe this audio file", "OCR this image", etc.).

## AI Orchestration System

Clarity includes a sophisticated orchestration system that intelligently routes content to specialized AI experts based on intent and content type.

### Architecture

```
Content → Translator → Orchestrator (LLM) → Experts → Results
```

**Components:**
- **Translator**: Converts any content (files, text, media) into structured form
- **Orchestrator**: Uses local LLM to analyze content and route to appropriate experts
- **Experts**: Specialized agents for different tasks:
  - **Producer** - Creates files, artifacts, and structured outputs
  - **Artist** - Generates creative content (stories, poems, designs, ASCII art)
  - **Scribe** - Documents information in Obsidian markdown vault
  - **Agent** - Executes actions with available tools (bash, HTTP, file ops)
  - **Analyst** - Provides research and in-depth analysis

### Usage

```bash
# Orchestrate a file through the expert system
just orchestrate path/to/file.txt

# Use specific model for routing
just orchestrate-model llama3.1 path/to/code.rs

# Or directly with cargo
cargo run --bin clarity-orchestrate document.md
```

### How It Works

1. **Translation**: File content is decoded into structured textual form
2. **Routing Decision**: LLM analyzes content and determines which expert(s) should handle it
3. **Expert Processing**: Selected experts process the content (parallel or sequential)
4. **Results**: Each expert returns output and any artifacts created

**Example Flow:**
```
Code file → Translator → Orchestrator decides: "Analyst + Producer"
         → Analyst provides code analysis
         → Producer creates formatted documentation
```

### Configuration

Experts can be configured via `.clarity.json`:

```json
{
  "orchestrator": {
    "model": "gpt-oss:20b",
    "temperature": 0.7
  },
  "experts": {
    "producer": {
      "output_dir": "./artifacts"
    },
    "scribe": {
      "vault_path": "~/obsidian/vault",
      "default_location": "Clarity"
    },
    "agent": {
      "confirm_destructive": true,
      "allowed_tools": ["bash", "http", "file"]
    }
  }
}
```

See [doc/ORCHESTRATION_ARCHITECTURE.md](doc/ORCHESTRATION_ARCHITECTURE.md) for detailed architecture documentation.

## Examples

Explore all Ollama API features with these examples:

### 1. Streaming Chat
Real-time streaming responses, token by token.

```bash
just example-streaming
# or: cargo run --example streaming_chat
```

**API:** `/api/chat` with `stream: true`

### 2. Generate
Single-shot text completion with configurable parameters.

```bash
just example-generate
# or: cargo run --example generate
```

**API:** `/api/generate`
**Features:** Temperature, top_p, token limits, performance stats

### 3. Embeddings
Generate vector embeddings and calculate semantic similarity.

```bash
just example-embeddings
# or: cargo run --example embeddings
```

**API:** `/api/embed`
**Features:** Cosine similarity calculation, batch processing

**Note:** Requires an embedding model like `nomic-embed-text`:
```bash
ollama pull nomic-embed-text
```

### 4. Tool Calling
Function calling / tool use with structured responses.

```bash
just example-tools
# or: cargo run --example tool_calling
```

**API:** `/api/chat` with `tools` field
**Features:** Function definitions, tool execution, multi-turn conversations

**Requires:** A model with tool support (llama3.1, gpt-oss, mistral, etc.)

### 5. Model Management
List, inspect, and monitor models.

```bash
just example-models
# or: cargo run --example model_management
```

**APIs:**
- `/api/tags` - List available models
- `/api/show` - Get model details and capabilities
- `/api/ps` - List loaded models with memory usage

### 6. Vision/Multimodal
Analyze images with vision-capable models.

```bash
just example-vision /path/to/image.jpg
# or: cargo run --example vision /path/to/image.jpg
```

**API:** `/api/chat` with base64-encoded images
**Features:** Image analysis, multi-modal understanding

**Requires:** A vision model like `llava`:
```bash
ollama pull llava
```

### 7. Structured Output
JSON schema-validated responses.

```bash
just example-structured
# or: cargo run --example structured_output
```

**API:** `/api/chat` with `format` field
**Features:** Type-safe JSON responses, schema validation

### 8. Orchestration
AI orchestration with specialized experts.

```bash
just example-orchestration
# or: cargo run --example orchestration
```

**Features:** Content translation, LLM-based routing, multi-expert processing
**Demonstrates:** All 5 experts (Producer, Artist, Scribe, Agent, Analyst)

## Ollama API Reference

### Core Endpoints

| Endpoint | Purpose | Example |
|----------|---------|---------|
| `/api/chat` | Multi-turn conversations | `streaming_chat.rs` |
| `/api/generate` | Single-shot completion | `generate.rs` |
| `/api/embed` | Vector embeddings | `embeddings.rs` |
| `/api/show` | Model information | `model_management.rs` |
| `/api/tags` | List models | `model_management.rs` |
| `/api/ps` | Loaded models | `model_management.rs` |

### Capabilities Detection

Models can advertise capabilities in the `/api/show` response:

- `completion` - Text generation
- `tools` - Function/tool calling
- `vision` - Image understanding

The main app automatically detects and displays tool calling support.

## Requirements

- Rust 1.90+ (Edition 2024)
- Ollama running on `localhost:11434`

## Installation

```bash
# Clone and build
git clone <repo>
cd clarity
cargo build --release

# Run
./target/release/clarity-chat
```

## Development

### Just Commands

This project uses [just](https://github.com/casey/just) for task automation. Run `just --list` to see all available commands:

```bash
# Build and run
just build              # Build all binaries and examples
just build-release      # Build optimized release version
just chat               # Run interactive chat
just chat-model MODEL   # Run with specific model

# File watching
just watch-files        # Watch for file changes
just watch-init         # Create .clarity.json config
just watch-last         # Dispatch last modified file to AI

# Examples
just example-streaming  # Streaming chat
just example-generate   # Text generation
just example-tools      # Tool calling
just examples-all       # Run all non-interactive examples

# Development
just check              # Quick syntax check
just lint               # Run clippy linter
just fmt                # Format code
just test               # Run tests
just watch              # Auto-rebuild on changes
```

### Binaries

Clarity includes multiple binaries:

- **`clarity-chat`** - Interactive chat with markdown rendering
- **`clarity-watch`** - File change monitor with AI orchestrator integration
  - Real-time file monitoring
  - Automatic file type detection
  - AI-powered file analysis via `--last` flag
- **`clarity-orchestrate`** - AI orchestration system with specialized experts
  - Intelligent content translation
  - LLM-based expert routing
  - Multi-expert parallel/sequential execution

### Tech Stack

Built with:
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde` - Serialization
- `futures-util` - Streaming support
- `termimad` - Terminal markdown rendering
- `crossterm` - Terminal styling
- `notify` - File system event monitoring
- `chrono` - Date/time handling

## Nix Support

```bash
# Enter dev shell
nix develop

# Build with Nix
nix build
```

## License

MIT
