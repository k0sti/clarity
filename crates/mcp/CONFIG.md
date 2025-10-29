# MCP Configuration System

The MCP system uses a two-tier configuration approach:
- **Shared config** (`config.toml`): Common settings for all agents and the user
- **Agent configs** (`agents/*.toml`): Agent-specific information

## Directory Structure

```
crates/mcp/
├── config.toml           # Shared configuration
└── agents/               # Agent-specific configs
    ├── gardener.toml
    ├── rust-expert.toml
    └── math-tutor.toml
```

## Shared Configuration (`config.toml`)

Contains settings shared by all agents and the user:

- **Nostr relays**: Connection endpoints for the Nostr network
- **Ollama settings**: LLM host and model configuration
- **Encryption mode**: Security settings
- **Private keys**: All agent and user keys in one place

### Example `config.toml`

```toml
[nostr]
relays = ["wss://strfry.atlantislabs.space"]

[ollama]
host = "http://localhost:11434"
model = "llama3.2"

[encryption]
mode = "optional"  # optional, required, or disabled

[keys]
user = "nsec1..."
gardener = "nsec1..."
rust_expert = "nsec1..."
math_tutor = "nsec1..."
```

## Agent Configuration (`agents/*.toml`)

Agent configs only contain agent-specific information:

### Example `agents/gardener.toml`

```toml
[agent]
name = "Gardening Expert"
subject = "gardening"
about = "I'm an expert in gardening, plant care, and sustainable farming practices"
```

## How It Works

1. **Agent startup**:
   - Loads `config.toml` (shared settings)
   - Loads agent-specific config (e.g., `agents/gardener.toml`)
   - Merges both configs (agent-specific overrides shared if present)
   - Looks up private key using agent ID (filename: `gardener` → key: `keys.gardener`)

2. **Key generation**:
   - If no key found in shared config, generates new one
   - Prints instructions showing exactly where to add it
   - Example output:
     ```
     Generated new private key!
     Public key (npub): npub1...

     Add this to your shared config file (crates/mcp/config.toml):
     [keys]
     gardener = "nsec1..."
     ```

3. **Agent ID derivation**:
   - Automatically derived from config filename
   - `agents/gardener.toml` → agent_id: `gardener`
   - `agents/rust-expert.toml` → agent_id: `rust_expert`
   - Hyphens converted to underscores for TOML compatibility

## Running Agents

### Using justfile (recommended):

```bash
just gardener      # Run gardening expert
just rust-expert   # Run Rust expert
just math-tutor    # Run math tutor
just user          # Run user TUI
```

### Direct cargo commands:

```bash
# Run with auto-detected agent ID
cargo run --bin mcp-agent -- --config crates/mcp/agents/gardener.toml

# Override agent ID
cargo run --bin mcp-agent -- --config crates/mcp/agents/gardener.toml --agent-id my_gardener

# Override shared config path
cargo run --bin mcp-agent -- --config crates/mcp/agents/gardener.toml --shared-config /path/to/config.toml

# Run user agent
cargo run --bin mcp-user
```

## CLI Overrides

All CLI arguments override config file values:

```bash
# Override relay
mcp-agent --config agents/gardener.toml --relay wss://other-relay.com

# Override Ollama settings
mcp-agent --config agents/gardener.toml --ollama-host http://10.0.0.5:11434 --ollama-model llama3.1

# Override encryption mode
mcp-agent --config agents/gardener.toml --encryption required

# Override private key (for testing)
mcp-agent --config agents/gardener.toml --private-key nsec1...
```

## Adding New Agents

1. Create new agent config file:
   ```bash
   cat > crates/mcp/agents/my-expert.toml <<EOF
   [agent]
   name = "My Expert"
   subject = "my expertise area"
   about = "Description of what I do"
   EOF
   ```

2. Add justfile command (optional):
   ```justfile
   # Run my expert agent
   my-expert:
       just agent my-expert
   ```

3. Run the agent (key will be auto-generated):
   ```bash
   just agent my-expert
   # OR
   cargo run --bin mcp-agent -- --config crates/mcp/agents/my-expert.toml
   ```

4. Copy the generated key to `config.toml`:
   ```toml
   [keys]
   my_expert = "nsec1..."  # Note: hyphens become underscores
   ```

## User Agent

The user agent also uses the shared config:

```bash
# Run with default config
cargo run --bin mcp-user

# Use different shared config
cargo run --bin mcp-user -- --shared-config /path/to/config.toml

# Override settings
cargo run --bin mcp-user -- --relay wss://other-relay.com --encryption required
```

User's private key is stored as `keys.user` in `config.toml`.

## Benefits of This Design

1. **Single source of truth**: All keys in one file
2. **DRY principle**: No duplicated settings across agent configs
3. **Easy management**: Update relay/Ollama settings in one place
4. **Flexible**: CLI overrides still available for testing
5. **Clear separation**: Agent identity vs shared infrastructure
6. **Auto-ID**: Agent IDs derived from filenames automatically
7. **Helpful**: Clear instructions when keys need to be added
