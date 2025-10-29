//! MCP Agent binary - LLM agent providing services over ContextVM

use clap::Parser;
use mcp::config::MergedConfig;
use mcp::core::types::{EncryptionMode, ServerInfo};
use mcp::gateway::Gateway;
use mcp::signer;
use mcp::transport::server::NostrServerTransportConfig;
use nostr_sdk::nips::nip19::ToBech32;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to agent configuration file (TOML)
    #[arg(short, long)]
    config: PathBuf,

    /// Agent ID (for looking up key in shared config: user, gardener, rust_expert, math_tutor)
    #[arg(long)]
    agent_id: Option<String>,

    /// Path to shared configuration (default: crates/mcp/config.toml)
    #[arg(long, default_value = "crates/mcp/config.toml")]
    shared_config: PathBuf,

    /// Nostr relay URLs (overrides config)
    #[arg(long)]
    relay: Vec<String>,

    /// Nostr private key (nsec or hex format) (overrides config)
    #[arg(long)]
    private_key: Option<String>,

    /// Ollama API host (overrides config)
    #[arg(long)]
    ollama_host: Option<String>,

    /// Ollama model (overrides config)
    #[arg(long)]
    ollama_model: Option<String>,

    /// Encryption mode: optional, required, disabled (overrides config)
    #[arg(long)]
    encryption: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Derive agent_id from config path if not provided
    let agent_id = args.agent_id.clone().unwrap_or_else(|| {
        args.config
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace('-', "_"))
            .unwrap_or_else(|| "unknown".to_string())
    });

    println!("Loading configuration from: {}", args.config.display());
    println!("Using agent ID: {}", agent_id);

    // Load and merge configurations
    let mut config = MergedConfig::load(
        args.shared_config.to_str().unwrap(),
        args.config.to_str().unwrap(),
        &agent_id,
    )?;

    // Apply CLI overrides
    if !args.relay.is_empty() {
        config.nostr.relays = args.relay;
    }
    if let Some(pk) = args.private_key {
        config.nostr.private_key = Some(pk);
    }
    if let Some(host) = args.ollama_host {
        config.ollama.host = host;
    }
    if let Some(model) = args.ollama_model {
        config.ollama.model = model;
    }
    if let Some(encryption) = args.encryption {
        config.encryption.mode = encryption;
    }

    // Parse encryption mode
    let encryption_mode = match config.encryption.mode.as_str() {
        "optional" => EncryptionMode::Optional,
        "required" => EncryptionMode::Required,
        "disabled" => EncryptionMode::Disabled,
        _ => {
            eprintln!("Invalid encryption mode: {}", config.encryption.mode);
            std::process::exit(1);
        }
    };

    // Get or generate signer
    let signer = if let Some(sk) = &config.nostr.private_key {
        signer::from_sk(sk)?
    } else {
        let keys = signer::generate();
        println!("\nGenerated new private key!");
        println!("Public key (npub): {}", keys.public_key().to_bech32()?);
        println!("\nAdd this to your shared config file ({}):", args.shared_config.display());
        println!("[keys]");
        println!("{} = \"{}\"", agent_id, keys.secret_key().to_bech32()?);
        println!();
        keys
    };

    println!("Agent: {}", config.agent.name);
    println!("Subject: {}", config.agent.subject);
    println!("Agent pubkey: {}", signer.public_key().to_bech32()?);
    println!("Ollama host: {}", config.ollama.host);
    println!("Ollama model: {}", config.ollama.model);

    // Create server info
    let server_info = ServerInfo {
        name: Some(config.agent.name.clone()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        about: config.agent.about.clone(),
        ..Default::default()
    };

    // Create config
    let transport_config = NostrServerTransportConfig {
        relay_urls: config.nostr.relays.clone(),
        encryption_mode,
        server_info: Some(server_info),
        session_timeout: Duration::from_secs(300),
    };

    // Create gateway
    let gateway = Gateway::new(signer, transport_config).await?;

    println!("Generating tools using Ollama...");

    // Generate tools based on subject using LLM
    let tools = match mcp::ollama::generate_tools_for_subject(
        &config.ollama.host,
        &config.ollama.model,
        &config.agent.subject,
    )
    .await
    {
        Ok(tools) => {
            println!("Generated {} tools for {}", tools.len(), config.agent.subject);
            for tool in &tools {
                if let Some(name) = tool.get("name").and_then(|v| v.as_str()) {
                    println!("  - {}", name);
                }
            }
            tools
        }
        Err(e) => {
            eprintln!("Warning: Failed to generate tools from LLM: {}", e);
            eprintln!("Using fallback query tool");
            vec![serde_json::json!({
                "name": "query",
                "description": format!("Ask questions about {}", config.agent.subject),
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "question": {
                            "type": "string",
                            "description": "Your question"
                        }
                    },
                    "required": ["question"]
                }
            })]
        }
    };

    println!("Publishing tools to relay...");
    gateway.publish_tools(tools).await?;

    println!("Starting agent gateway...");

    // Start the gateway to listen for requests
    gateway.start().await?;

    Ok(())
}
