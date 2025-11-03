# Assistant Crate Design Document

**Date**: 2025-11-02
**Status**: Deprecated - Prototype Discontinued
**Version**: 0.1.0

---

## Project Status: DISCONTINUED

The `crates/assistant` prototype has been **discontinued** as of 2025-11-03. The Dioxus framework proved unsuitable for rapid UI and AI development needs.

### Prototype Report

**Implementation Summary:**
- **Framework**: Dioxus 0.7.0-rc.3 (desktop features)
- **Architecture**: Simple desktop application with basic chat interface
- **Code Size**: ~60 lines of Rust code
- **Features Implemented**:
  - Basic chat UI with message display
  - Text input with Enter key and button send
  - Message state management using Dioxus signals
  - CSS styling for dark theme chat interface

**What Was Built:**
The prototype implemented only the most basic chat interface:
- Header with "AI Assistant" title
- Scrollable message area displaying user messages
- Input area with text field and send button
- Message state tracked locally (user messages only)
- No AI integration (commented TODOs for MCP server connection)

**Limitations Discovered:**

1. **Development Velocity**: The reactive UI paradigm in Dioxus required significant boilerplate for simple interactions
2. **AI Integration Gap**: No clear path to integrate streaming LLM responses with Dioxus's reactive model
3. **Rapid Iteration**: Making UI changes required full recompilation, slowing down design iteration
4. **Ecosystem Maturity**: Dioxus 0.7.0-rc.3 lacked mature components for complex AI assistant features
5. **Async Complexity**: Integrating tokio-based async AI operations with Dioxus event system was unclear

**Technical Debt Avoided:**
By discontinuing early, we avoided:
- Complex state synchronization between UI and AI streams
- Custom component library development
- Desktop-specific packaging and distribution challenges
- Multi-platform compatibility issues

**Files Removed:**
- `crates/assistant/src/main.rs` - Main application entry point
- `crates/assistant/assets/style.css` - Chat interface styling
- `crates/assistant/Cargo.toml` - Dependency manifest
- `crates/assistant/flake.nix` - Nix development environment
- `crates/assistant/flake.lock` - Nix lock file

---

## Original Vision (Not Implemented)

The `crates/assistant` was designed to be the **unified desktop interface** for the entire Clarity project, combining local LLMs (Ollama), intelligent content routing (Orchestrator), and decentralized agent communication (MCP over Nostr) into a single, cohesive application.

**Vision**: A local-first, privacy-focused AI assistant that intelligently routes tasks to specialized experts, discovers and collaborates with decentralized agents, and provides a beautiful desktop experience.

---

## Features

- Multiple assistant agents in one interface. initially just tabbed
 - Trigger configuration panel. Agent may have triggers enabled that start and feed input to llm
- Visual indicators in tabs showing which agent is active
- Progress bars for long-running tasks

- **Streaming Responses**: Real-time LLM output rendering (using ollama crate's streaming)
- **Message Types**: Support text, code, images, files
- **History**: Persistent conversation history with SQLite
- **Tool Call Visualization**: Show when/how tools are invoked
- **Markdown Rendering**: Rich text display with syntax highlighting

**User Experience:**
- Clean, distraction-free interface
- Keyboard shortcuts (Enter to send, Ctrl+N for new session)
- Copy/paste code blocks

---

```
Agent System
├─ Agent Directory (browse Nostr-published agents)
├─ Agent Cards (name, subject, capabilities, tools)
```

**Agent Discovery:**
- Automatically discovers agents on configured Nostr relays
- Displays agent capabilities, subjects, and available tools
- Shows agent status (online/offline)
- Filter agents by capability or subject

---

### 4. Local LLM Management 🤖

**Features:**

```
LLM Controls
├─ Model Selector (dropdown of available Ollama models)
├─ Model Info Display (size, parameters, capabilities)
├─ Parameter Controls (temperature, top_p, seed, etc.)
├─ Vision Model Support (image upload for llava/bakllava)
```

**Model Management:**
- List all installed Ollama models
- Show model details (size, parameter count, quantization)
- Quick model switching mid-conversation
- Model download/update status
- Model performance metrics (tokens/sec)

**Advanced Controls:**
- Temperature`^
- Top-p, top-`^ for reproducibility
- System prompt editor

**Vision Support:**
- Drag-and-drop images
- Image preview in chat
- Support for llava, bakllava models
- Multiple images per message

---

### 5. Advanced Capabilities 🔧

#### Tool System
- **Visual Tool Library Browser**: See all available tools across agents

#### File Operations
- **Integrated File Browser**: Navigate local filesystem


---

### 6. Configuration & Settings ⚙️

**Settings Tabs:**

```
Settings
├─ Ollama
│  ├─ Host URL (default: http://localhost:11434)
│  ├─ Default Model
│  ├─ Streaming (on/off)
│  ├─ Timeout settings
│  └─ Connection test
├─ Nostr
│  ├─ Relay List (add/remove relays)
│  ├─ Key Management (view/generate keys)
│  ├─ Encryption Mode (optional/required/disabled)
│  └─ Relay status indicators
├─ Triggers
│  ├─ Vault Location (for Scribe)
│  ├─ Artifacts Directory
│  ├─ Auto-routing (on/off)
│  └─ File Watcher settings
│  └─ Accent Color
```

