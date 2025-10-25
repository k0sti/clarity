# Clarity - Ollama Chat Client
# Run `just --list` to see all available commands

# Default recipe - show help
default:
    @just --list

# Run the interactive chat
chat:
    cargo run --bin clarity-chat

# Run chat with a specific model
chat-model MODEL:
    OLLAMA_MODEL={{MODEL}} cargo run --bin clarity-chat

# Watch for file changes
watch-files:
    cargo run --bin clarity-watch

# Initialize .clarity.json config
watch-init:
    cargo run --bin clarity-watch -- --init

# Find and dispatch last modified file to orchestrator
watch-last:
    cargo run --bin clarity-watch -- --last

# Build all binaries and examples
build:
    cargo build --bins --examples

# Build release version
build-release:
    cargo build --release --bins --examples

# Run tests
test:
    cargo test

# Check code without building
check:
    cargo check --bins --examples

# Format code
fmt:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy --bins --examples

# Clean build artifacts
clean:
    cargo clean

# Examples
# --------

# Run streaming chat example
example-streaming:
    cargo run --example streaming_chat

# Run generation example
example-generate:
    cargo run --example generate

# Run embeddings example (requires nomic-embed-text model)
example-embeddings:
    cargo run --example embeddings

# Run tool calling example
example-tools:
    cargo run --example tool_calling

# Run tool calling with specific model
example-tools-model MODEL:
    OLLAMA_MODEL={{MODEL}} cargo run --example tool_calling

# Run model management example
example-models:
    cargo run --example model_management

# Run vision example with image
example-vision IMAGE:
    cargo run --example vision {{IMAGE}}

# Run structured output example
example-structured:
    cargo run --example structured_output

# Run all examples (non-interactive ones)
examples-all:
    @echo "Running generate example..."
    @cargo run --example generate
    @echo "\nRunning model management example..."
    @cargo run --example model_management
    @echo "\nRunning structured output example..."
    @cargo run --example structured_output

# Development
# -----------

# Watch and rebuild on changes
watch:
    cargo watch -x 'build --bins'

# Install the binary
install:
    cargo install --path .

# Show binary info
info:
    @echo "Available binaries:"
    @ls -lh target/release/clarity-* 2>/dev/null || echo "  (run 'just build-release' first)"
    @echo "\nAvailable examples:"
    @ls -1 examples/*.rs | sed 's/examples\//  - /g' | sed 's/\.rs//g'
