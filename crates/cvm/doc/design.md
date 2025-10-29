# CVM (ContextVM) Rust Implementation Design

## Overview

This crate implements a Rust library and binaries for agent-to-agent communication using the ContextVM protocol, which bridges Nostr and the Model Context Protocol (MCP).

## Library Architecture

### Core Module (`crate::core`)

**Protocol Types:**
- `EventKind` - Enum for ContextVM event kinds (25910, 1059, 11316-11320)
- `NostrTag` - Tag constants (p, e, cap, support_encryption)
- `McpMessage` - Wrapper for JSON-RPC MCP messages

**Encryption:**
- `EncryptionMode` - Enum: Optional, Required, Disabled
- `encrypt_message()` - NIP-44 encryption
- `decrypt_message()` - NIP-44 decryption
- `gift_wrap()` - NIP-59 gift wrapping for metadata protection

**Errors:**
- `CvmError` - Protocol, encryption, transport, and relay errors

### Transport Module (`crate::transport`)

**Server Transport (`NostrServerTransport`):**
- Listen for incoming MCP requests via Nostr events
- Manage client sessions with initialization state
- Handle request/response correlation using event IDs
- Support optional encryption per-session
- Send responses back to clients via ephemeral events

**Client Transport (`NostrClientTransport`):**
- Send MCP requests to servers via Nostr
- Subscribe to responses using event correlation
- Handle server discovery and initialization
- Support optional encryption negotiation

### Gateway Module (`crate::gateway`)

**Purpose:** Expose local MCP server capabilities over Nostr network

**Components:**
- `Gateway` - Bridge between local MCP server and Nostr transport
- Forward requests from Nostr to local MCP server
- Publish responses back to Nostr
- Server announcement publishing (kinds 11316-11320)

### Proxy Module (`crate::proxy`)

**Purpose:** Client-side access to remote Nostr-based MCP servers

**Components:**
- `Proxy` - Bridge between local MCP client and Nostr transport
- Forward local requests to remote servers via Nostr
- Relay responses back to local client

### Relay Module (`crate::relay`)

**Nostr Relay Management:**
- `RelayPool` - Connection pool for multiple Nostr relays
- `connect()` - Establish relay connections
- `publish()` - Publish events to relays
- `subscribe()` - Subscribe to event filters
- `disconnect()` - Clean relay disconnection

### Signer Module (`crate::signer`)

**Cryptographic Operations:**
- `NostrSigner` - Trait for signing Nostr events
- `PrivateKeySigner` - Implementation using Nostr private keys
- Sign events with private key
- NIP-44 encryption/decryption support

### Tools Module (`crate::tools`)

**MCP Tool Implementations for Agents:**
- `OllamaTool` - Interface to Ollama LLM
- `SearchTool` - Web/knowledge search
- `MemoryTool` - Agent memory/context storage

## Agent Binary

### Purpose
LLM agent that provides subject-specific expertise over ContextVM.

### Command Line Interface
```
cvm-agent --subject <SUBJECT> [OPTIONS]

Arguments:
  --subject <SUBJECT>          Field of expertise (e.g., "rust", "security", "math")

Options:
  --relay <URL>...             Nostr relay URLs (default: wss://relay.damus.io)
  --private-key <KEY>          Nostr private key (or generate new)
  --ollama-host <HOST>         Ollama API host (default: http://localhost:11434)
  --ollama-model <MODEL>       Ollama model (default: llama3.2)
  --encryption <MODE>          Encryption mode: optional, required, disabled (default: optional)
```

### Agent Architecture

**Initialization:**
1. Load or generate Nostr private key
2. Connect to Nostr relays
3. Initialize Ollama client
4. Configure MCP tools based on subject
5. Publish server announcements (kind 11316-11320)

**Main Loop:**
1. Subscribe to initialization requests (filter by pubkey)
2. Handle MCP protocol handshake
3. Process incoming tool/resource/prompt requests
4. Execute tools using Ollama LLM
5. Send responses via Nostr

**Tool Configuration by Subject:**
- Default tools: `ollama_generate`, `memory_store`, `memory_recall`
- Subject-specific tools loaded from configuration

**Session Management:**
- Track client sessions with initialization state
- Maintain conversation context per client
- Cleanup inactive sessions

### Error Handling
- Log all errors to stderr
- Send error responses to clients via MCP error format
- Graceful shutdown on SIGTERM/SIGINT

## UserAgent Binary

### Purpose
Terminal UI for humans to interact with agents via ContextVM.

### Command Line Interface
```
cvm-user [OPTIONS]

Options:
  --relay <URL>...             Nostr relay URLs (default: wss://relay.damus.io)
  --private-key <KEY>          Nostr private key (or generate new)
  --server <PUBKEY>            Connect to specific agent pubkey
  --encryption <MODE>          Encryption mode: optional, required, disabled (default: optional)
```

### UserAgent Architecture

**Initialization:**
1. Load or generate Nostr private key
2. Connect to Nostr relays
3. Display available servers (if no --server specified)
4. Initialize MCP session with selected server

**Terminal UI:**
- Color terminal using `crossterm` crate
- Chat-style interface with message history
- Display user messages and agent responses
- Show available tools/resources/prompts
- Status indicators (connected, typing, error)

**Commands:**
- `/list tools` - List available tools
- `/list resources` - List available resources
- `/list prompts` - List available prompts
- `/call <tool> <args>` - Call a tool
- `/servers` - List discovered servers
- `/connect <pubkey>` - Connect to different server
- `/quit` - Exit

**Message Flow:**
1. User types message
2. Send as MCP request (e.g., tool call, prompt execution)
3. Display agent response in chat
4. Handle progress notifications for long-running operations

## Dependencies

**Required Crates:**
- `nostr-sdk` - Nostr protocol implementation
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `crossterm` - Terminal UI (UserAgent)
- `reqwest` - Ollama HTTP client (Agent)
- `thiserror` - Error handling

## Implementation Notes

**Protocol Compliance:**
- Follow ContextVM spec for event formats
- Use kind 25910 for all MCP messages (ephemeral)
- Use kinds 11316-11320 for server announcements (replaceable)
- Support NIP-44 encryption and NIP-59 gift wrapping

**Nostr Event Structure:**
- `content` field contains stringified JSON-RPC MCP message
- Use `p` tag for addressing (provider/client pubkey)
- Use `e` tag for request/response correlation
- Use `cap` tag for capability pricing metadata

**Session Management:**
- Initialize with MCP handshake (initialize/initialized)
- Track sessions by client pubkey
- Cleanup after timeout (default: 5 minutes)

**Ollama Integration:**
- HTTP API client for model inference
- System prompt includes agent subject expertise
- Context management for multi-turn conversations
- Streaming support for long responses

**Error Recovery:**
- Reconnect to relays on connection loss
- Retry failed event publishing
- Handle malformed messages gracefully
