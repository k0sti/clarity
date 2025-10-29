# Clarity - AI orchestration and Ollama examples
# Usage: just <recipe>

# Default recipe - show available commands
default:
    @just --list

# Build all workspace crates
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Run tests for all crates
test:
    cargo test

# Check all crates compile
check:
    cargo check

# Format all code
fmt:
    cargo fmt --all

# Run clippy linter
lint:
    cargo clippy --all-targets --all-features

# Clean build artifacts
clean:
    cargo clean

# ============================================================================
# Orchestrator Commands
# ============================================================================

# Run orchestration CLI on a file
orchestrate FILE:
    cargo run --bin clarity-orchestrate -- {{FILE}}

# Run orchestration CLI with custom model
orchestrate-model FILE MODEL:
    OLLAMA_MODEL={{MODEL}} cargo run --bin clarity-orchestrate -- {{FILE}}

# Run interactive chat
chat:
    cargo run --bin clarity-chat

# Run file watcher
watch:
    cargo run --bin clarity-watch

# ============================================================================
# Ollama Examples
# ============================================================================

# List all available examples
examples:
    @echo "Available examples:"
    @echo "  generate          - Single prompt completion"
    @echo "  streaming_chat    - Streaming chat with markdown"
    @echo "  embeddings        - Generate text embeddings"
    @echo "  orchestration     - AI orchestration with experts"
    @echo "  tool_calling      - Function calling with tools"
    @echo "  vision            - Vision model with images"
    @echo "  structured_output - Structured JSON output"
    @echo "  model_management  - List, pull, and manage models"
    @echo ""
    @echo "Run with: just example <name>"

# Run a specific example
example NAME:
    cargo run --example {{NAME}} -p ollama

# Run example with custom model
example-model NAME MODEL:
    OLLAMA_MODEL={{MODEL}} cargo run --example {{NAME}} -p ollama

# Build all examples
build-examples:
    cargo build --examples -p ollama

# ============================================================================
# CVM (Context VM) Commands
# ============================================================================

# Run CVM binary
cvm *ARGS:
    cargo run --bin cvm -- {{ARGS}}

# Build CVM crate
build-cvm:
    cargo build -p cvm

# ============================================================================
# MCP Agent Commands
# ============================================================================

# Run agent with config file
agent CONFIG:
    cargo run --bin mcp-agent -- --config crates/mcp/agents/{{CONFIG}}.toml

# Run user agent (TUI)
user:
    cargo run --bin mcp-user

# Run gardening agent
gardener:
    just agent gardener

# Run rust expert agent
rust-expert:
    just agent rust-expert

# Run math tutor agent
math-tutor:
    just agent math-tutor

# Build MCP crate
build-mcp:
    cargo build -p mcp

# ============================================================================
# Development Commands
# ============================================================================

# Run all checks (fmt, lint, test, build)
ci: fmt lint test build

# Watch and rebuild on file changes (requires cargo-watch)
dev:
    cargo watch -x check -x test

# Update dependencies
update:
    cargo update

# Show dependency tree
tree:
    cargo tree

# Audit dependencies for security vulnerabilities
audit:
    cargo audit

# Show workspace info
info:
    @echo "Clarity Workspace"
    @echo "================="
    @echo ""
    @echo "Crates:"
    @echo "  - boostrap     : Lua bootstrap experiments"
    @echo "  - orchestrator : AI orchestration system"
    @echo "  - cvm          : Context VM (Nostr DVM/MCP bridge)"
    @echo "  - mcp          : MCP over Nostr (ContextVM agents)"
    @echo "  - ollama       : Ollama API examples"
    @echo ""
    @echo "Binaries:"
    @echo "  - clarity-orchestrate : Orchestrate file processing"
    @echo "  - clarity-chat        : Interactive chat"
    @echo "  - clarity-watch       : File watcher"
    @echo "  - cvm                 : Context VM CLI"
    @echo "  - mcp-agent           : MCP agent with LLM"
    @echo "  - mcp-user            : MCP user agent (TUI)"
    @echo ""
    @echo "Quick MCP commands:"
    @echo "  just gardener         : Run gardening expert agent"
    @echo "  just user             : Run user agent TUI"

# ============================================================================
# Quick Shortcuts
# ============================================================================

# Run generate example (most common)
gen:
    just example generate

# Run streaming chat example
stream:
    just example streaming_chat

# Run orchestration example
orc:
    just example orchestration

# Quick chat with default model
c:
    just chat

# Quick watch current directory
w:
    just watch
