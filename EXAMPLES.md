# Clarity Examples

Complete examples demonstrating all Ollama API 2025 features.

## Quick Reference

| Example | API Endpoint | Features | Model Required |
|---------|-------------|----------|----------------|
| `streaming_chat.rs` | `/api/chat` | Real-time streaming | Any chat model |
| `generate.rs` | `/api/generate` | Single completion, stats | Any model |
| `embeddings.rs` | `/api/embed` | Vector embeddings, similarity | `nomic-embed-text` |
| `tool_calling.rs` | `/api/chat` | Function calling | Tool-enabled model |
| `model_management.rs` | `/api/tags`, `/api/show`, `/api/ps` | Model info & monitoring | None |
| `vision.rs` | `/api/chat` | Image analysis | `llava` or vision model |
| `structured_output.rs` | `/api/chat` | JSON schema validation | Any chat model |

## Running Examples

```bash
# Basic examples
cargo run --example streaming_chat
cargo run --example generate
cargo run --example model_management
cargo run --example structured_output

# Embeddings (requires embedding model)
ollama pull nomic-embed-text
cargo run --example embeddings

# Tool calling (requires tool-enabled model)
OLLAMA_MODEL=llama3.1 cargo run --example tool_calling

# Vision (requires vision model + image)
ollama pull llava
cargo run --example vision /path/to/image.jpg
```

## Example Details

### 1. streaming_chat.rs
**Demonstrates:** Real-time streaming responses

Shows how to:
- Enable streaming with `stream: true`
- Process streaming responses line-by-line
- Display tokens as they arrive
- Handle newline-delimited JSON

**Key Code:**
```rust
let mut stream = resp.bytes_stream();
while let Some(chunk) = stream.next().await {
    // Process each chunk as it arrives
}
```

### 2. generate.rs
**Demonstrates:** Single-shot text generation

Shows how to:
- Use `/api/generate` endpoint
- Configure generation parameters (temperature, top_p)
- Limit output tokens
- Access performance statistics

**Parameters:**
- `temperature`: 0.7 - Controls randomness
- `top_p`: 0.9 - Nucleus sampling
- `num_predict`: 100 - Max tokens

**Stats Returned:**
- Prompt tokens, response tokens
- Total duration, load duration

### 3. embeddings.rs
**Demonstrates:** Vector embeddings and semantic search

Shows how to:
- Generate embeddings for multiple texts
- Calculate cosine similarity
- Compare semantic similarity between texts
- Batch process multiple inputs

**Output:**
- Embedding dimensions
- Similarity scores between text pairs
- Demonstrates semantic understanding

**Required Model:**
```bash
ollama pull nomic-embed-text
```

### 4. tool_calling.rs
**Demonstrates:** Function calling / tool use

Shows how to:
- Define tool schemas with JSON Schema
- Let model decide when to call tools
- Execute tools and return results
- Continue conversation with tool results

**Features:**
- Multi-turn tool conversations
- Tool parameter extraction
- Function execution integration

**Compatible Models:**
- llama3.1, llama3.2
- gpt-oss:20b
- mistral, mixtral
- qwen2.5

### 5. model_management.rs
**Demonstrates:** Model introspection and monitoring

Shows how to:
- List all available models (`/api/tags`)
- Get detailed model info (`/api/show`)
- Check loaded models (`/api/ps`)
- Display memory usage (RAM/VRAM)

**Information Retrieved:**
- Model family, parameters, format
- Quantization level
- Capabilities (completion, tools, vision)
- Memory footprint

### 6. vision.rs
**Demonstrates:** Image understanding with vision models

Shows how to:
- Encode images as base64
- Send images in chat messages
- Analyze image content
- Use multimodal models

**Usage:**
```bash
cargo run --example vision /path/to/image.jpg
```

**Vision Models:**
- llava (general vision)
- llava-phi3 (faster)
- bakllava (detailed)

### 7. structured_output.rs
**Demonstrates:** Type-safe JSON responses

Shows how to:
- Define JSON schemas
- Validate response structure
- Parse type-safe outputs
- Ensure consistent formatting

**Schema Features:**
- Object properties
- Required fields
- Type validation
- Deserialize to Rust structs

## API Endpoint Coverage

### Chat API (`/api/chat`)
- ✅ Basic chat (main app)
- ✅ Streaming (streaming_chat.rs)
- ✅ Tool calling (tool_calling.rs)
- ✅ Vision/multimodal (vision.rs)
- ✅ Structured output (structured_output.rs)

### Generation API (`/api/generate`)
- ✅ Single completion (generate.rs)
- ✅ Parameter control (generate.rs)
- ✅ Performance stats (generate.rs)

### Embeddings API (`/api/embed`)
- ✅ Vector generation (embeddings.rs)
- ✅ Batch processing (embeddings.rs)
- ✅ Similarity calculation (embeddings.rs)

### Management APIs
- ✅ List models - `/api/tags` (model_management.rs)
- ✅ Model info - `/api/show` (model_management.rs, main app)
- ✅ Loaded models - `/api/ps` (model_management.rs)

## Features by Model Type

### Standard Chat Models
- Chat conversations
- Text generation
- Structured output
- Streaming

### Tool-Enabled Models
All standard features plus:
- Function calling
- Tool definitions
- Multi-turn tool use

### Vision Models
All standard features plus:
- Image analysis
- Visual Q&A
- Multimodal understanding

### Embedding Models
- Vector embeddings
- Semantic search
- Similarity comparison

## Advanced Features

### Streaming
Real-time token generation for responsive UIs.
**Example:** `streaming_chat.rs`

### Tool Calling
Enable models to use external functions and APIs.
**Example:** `tool_calling.rs`

### Structured Output
Type-safe JSON responses with schema validation.
**Example:** `structured_output.rs`

### Vision
Understand and analyze image content.
**Example:** `vision.rs`

### Performance Metrics
Track token counts, timing, and resource usage.
**Example:** `generate.rs`, `model_management.rs`

## Next Steps

1. **Extend tool_calling.rs** - Add more tools (web search, calculator, database)
2. **Enhance vision.rs** - Multi-image analysis, OCR, object detection
3. **Build RAG system** - Combine embeddings with vector DB
4. **Create agents** - Multi-turn autonomous task execution
5. **Add caching** - Implement prompt caching for faster responses

## Resources

- [Ollama API Docs](https://docs.ollama.com/api)
- [Model Library](https://ollama.com/library)
- [Tool-Enabled Models](https://ollama.com/search?c=tools)
- [Vision Models](https://ollama.com/search?c=vision)
