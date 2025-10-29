# Ollama Examples

Examples demonstrating how to use the Ollama API with Rust.

## Prerequisites

Make sure Ollama is running:
```bash
ollama serve
```

## Examples

### Basic Examples

- **`generate`** - Single prompt completion
  ```bash
  cargo run --example generate -p ollama
  ```

- **`streaming_chat`** - Streaming chat with markdown rendering
  ```bash
  cargo run --example streaming_chat -p ollama
  ```

- **`embeddings`** - Generate text embeddings
  ```bash
  cargo run --example embeddings -p ollama
  ```

### Advanced Examples

- **`orchestration`** - AI orchestration with specialized experts
  ```bash
  cargo run --example orchestration -p ollama
  ```

- **`tool_calling`** - Function calling with tools
  ```bash
  cargo run --example tool_calling -p ollama
  ```

- **`vision`** - Vision model example with image input
  ```bash
  cargo run --example vision -p ollama
  ```

- **`structured_output`** - Structured JSON output
  ```bash
  cargo run --example structured_output -p ollama
  ```

- **`model_management`** - List, pull, and manage models
  ```bash
  cargo run --example model_management -p ollama
  ```

## Configuration

Set the model to use via environment variable:
```bash
export OLLAMA_MODEL=llama3.1
cargo run --example generate -p ollama
```

Default model: `gpt-oss:20b`

## Building All Examples

```bash
cargo build --examples -p ollama
```
