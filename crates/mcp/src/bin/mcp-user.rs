//! MCP UserAgent binary - Terminal UI for humans to interact with agents

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mcp::config::SharedConfig;
use mcp::core::constants::{SERVER_ANNOUNCEMENT_KIND, TOOLS_LIST_KIND};
use mcp::core::types::EncryptionMode;
use mcp::proxy::Proxy;
use mcp::signer;
use mcp::transport::client::NostrClientTransportConfig;
use nostr_sdk::prelude::*;
use nostr_sdk::nips::nip19::ToBech32;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to shared configuration (default: crates/mcp/config.toml)
    #[arg(long, default_value = "crates/mcp/config.toml")]
    shared_config: PathBuf,

    /// Nostr relay URLs (overrides config)
    #[arg(long)]
    relay: Vec<String>,

    /// Nostr private key (nsec or hex format) (overrides config)
    #[arg(long)]
    private_key: Option<String>,

    /// Connect to specific agent (npub or hex pubkey)
    #[arg(long)]
    server: Option<String>,

    /// Encryption mode: optional, required, disabled (overrides config)
    #[arg(long)]
    encryption: Option<String>,
}

#[derive(Debug, Clone)]
struct DiscoveredAgent {
    pubkey: PublicKey,
    name: String,
    _version: Option<String>,
    about: Option<String>,
    tools: Vec<serde_json::Value>,
}

enum AppEvent {
    AgentDiscovered(DiscoveredAgent),
    ToolsDiscovered { pubkey: PublicKey, tools: Vec<serde_json::Value> },
    Quit,
}

struct App {
    input: String,
    messages: Vec<String>,
    discovered_agents: HashMap<PublicKey, DiscoveredAgent>,
    connected_agent: Option<PublicKey>,
    _proxy: Arc<Proxy>,
}

impl App {
    fn new(proxy: Arc<Proxy>) -> Self {
        Self {
            input: String::new(),
            messages: vec![
                "Welcome to MCP UserAgent!".to_string(),
                "Discovering agents on relay...".to_string(),
                "".to_string(),
                "Commands:".to_string(),
                "  /list            - List discovered agents".to_string(),
                "  /connect <n>     - Connect to agent by number".to_string(),
                "  /connect <npub>  - Connect to agent by npub".to_string(),
                "  /tools           - Show tools from connected agent".to_string(),
                "  /help            - Show this help".to_string(),
                "  /quit            - Exit".to_string(),
                "".to_string(),
            ],
            discovered_agents: HashMap::new(),
            connected_agent: None,
            _proxy: proxy,
        }
    }

    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
    }

    fn handle_agent_discovered(&mut self, agent: DiscoveredAgent) {
        let name = agent.name.clone();
        let pubkey_npub = agent.pubkey.to_bech32().unwrap_or_else(|_| agent.pubkey.to_hex());
        self.discovered_agents.insert(agent.pubkey, agent);
        self.add_message(format!(
            "🔍 Discovered agent: {} ({}...)",
            name,
            &pubkey_npub[..16]
        ));
    }

    fn handle_tools_discovered(&mut self, pubkey: PublicKey, tools: Vec<serde_json::Value>) {
        if let Some(agent) = self.discovered_agents.get_mut(&pubkey) {
            let agent_name = agent.name.clone();
            agent.tools = tools.clone();
            self.add_message(format!(
                "🛠️  {} tools available from {}",
                tools.len(),
                agent_name
            ));
        }
    }

    fn handle_command(&mut self, input: String) -> Option<AppEvent> {
        let input = input.trim();

        if input.is_empty() {
            return None;
        }

        match input {
            "/quit" | "/exit" => {
                return Some(AppEvent::Quit);
            }
            "/list" => {
                if self.discovered_agents.is_empty() {
                    self.add_message("No agents discovered yet.".to_string());
                } else {
                    self.add_message("".to_string());
                    self.add_message("Discovered agents:".to_string());

                    // Collect messages first to avoid borrow checker issues
                    let agent_msgs: Vec<String> = self
                        .discovered_agents
                        .values()
                        .enumerate()
                        .map(|(idx, agent)| {
                            let about = agent.about.as_deref().unwrap_or("No description");
                            format!("  {}. {} - {}", idx + 1, agent.name, about)
                        })
                        .collect();

                    for msg in agent_msgs {
                        self.add_message(msg);
                    }
                    self.add_message("".to_string());
                }
            }
            cmd if cmd.starts_with("/connect ") => {
                let arg = cmd.strip_prefix("/connect ").unwrap().trim();

                // Try to parse as number (index)
                if let Ok(idx) = arg.parse::<usize>() {
                    if idx == 0 || idx > self.discovered_agents.len() {
                        self.add_message(format!("Invalid agent number. Use /list to see available agents."));
                    } else {
                        let agent = self.discovered_agents.values().nth(idx - 1).unwrap();
                        self.connected_agent = Some(agent.pubkey);
                        self.add_message(format!("✓ Connected to: {}", agent.name));
                    }
                } else {
                    // Try to parse as pubkey (hex or npub)
                    match PublicKey::parse(arg) {
                        Ok(pk) => {
                            self.connected_agent = Some(pk);
                            let npub = pk.to_bech32().unwrap_or_else(|_| pk.to_hex());
                            self.add_message(format!("✓ Connected to: {}", npub));
                        }
                        Err(e) => {
                            self.add_message(format!("Invalid agent number or npub: {}", e));
                        }
                    }
                }
            }
            "/tools" => {
                if let Some(pubkey) = &self.connected_agent {
                    if let Some(agent) = self.discovered_agents.get(pubkey) {
                        if agent.tools.is_empty() {
                            self.add_message("No tools discovered yet for this agent.".to_string());
                        } else {
                            // Clone the data we need before mutating self
                            let agent_name = agent.name.clone();
                            let tools: Vec<(String, String)> = agent.tools.iter()
                                .map(|tool| {
                                    let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                                    let desc = tool.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    (name, desc)
                                })
                                .collect();

                            self.add_message("".to_string());
                            self.add_message(format!("Tools from {}:", agent_name));
                            for (name, desc) in tools {
                                self.add_message(format!("  • {} - {}", name, desc));
                            }
                            self.add_message("".to_string());
                        }
                    } else {
                        self.add_message("Connected agent not found in discovered list.".to_string());
                    }
                } else {
                    self.add_message("Not connected to any agent. Use /list and /connect".to_string());
                }
            }
            "/help" => {
                self.add_message("".to_string());
                self.add_message("Commands:".to_string());
                self.add_message("  /list            - List discovered agents".to_string());
                self.add_message("  /connect <n>     - Connect to agent by number".to_string());
                self.add_message("  /connect <npub>  - Connect to agent by npub".to_string());
                self.add_message("  /tools           - Show tools from connected agent".to_string());
                self.add_message("  /help            - Show this help".to_string());
                self.add_message("  /quit            - Exit".to_string());
                self.add_message("".to_string());
            }
            _ => {
                if let Some(pubkey) = &self.connected_agent {
                    let agent_name = self
                        .discovered_agents
                        .get(pubkey)
                        .map(|a| a.name.as_str())
                        .unwrap_or("Unknown");
                    self.add_message(format!("→ [{}] {}", agent_name, input));
                    self.add_message("  (Not implemented yet)".to_string());
                    // TODO: Send actual MCP request
                } else {
                    self.add_message("Not connected to any agent. Use /list and /connect".to_string());
                }
            }
        }

        None
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load shared config
    let shared_config = SharedConfig::from_file(&args.shared_config).unwrap_or_default();

    // Get relay URLs (CLI overrides config)
    let relay_urls = if !args.relay.is_empty() {
        args.relay
    } else {
        shared_config.nostr.relays.clone()
    };

    // Parse encryption mode (CLI overrides config)
    let encryption_mode_str = args.encryption.as_deref().unwrap_or(&shared_config.encryption.mode);
    let encryption_mode = match encryption_mode_str {
        "optional" => EncryptionMode::Optional,
        "required" => EncryptionMode::Required,
        "disabled" => EncryptionMode::Disabled,
        _ => {
            eprintln!("Invalid encryption mode: {}", encryption_mode_str);
            std::process::exit(1);
        }
    };

    // Get private key (CLI > config > generate)
    let private_key = args.private_key
        .or_else(|| shared_config.get_key("user"));

    // Get or generate signer
    let signer = if let Some(sk) = private_key {
        signer::from_sk(&sk)?
    } else {
        let keys = signer::generate();
        eprintln!("\nGenerated new private key for user!");
        eprintln!("Public key (npub): {}", keys.public_key().to_bech32()?);
        eprintln!("\nAdd this to your shared config file ({}):", args.shared_config.display());
        eprintln!("[keys]");
        eprintln!("user = \"{}\"", keys.secret_key().to_bech32()?);
        eprintln!();
        keys
    };

    // Create config
    let config = NostrClientTransportConfig {
        relay_urls,
        encryption_mode,
    };

    // Create and connect proxy
    let proxy = Arc::new(Proxy::new(signer.clone(), config).await?);
    proxy.connect().await?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(proxy.clone());

    // Start agent discovery task
    let (event_tx, mut event_rx) = mpsc::channel(100);
    let discovery_tx = event_tx.clone();

    tokio::spawn(async move {
        if let Err(e) = discover_agents(signer, discovery_tx).await {
            eprintln!("Discovery error: {}", e);
        }
    });

    // Main event loop
    let result = run_app(&mut terminal, &mut app, &mut event_rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn discover_agents(
    signer: Keys,
    event_tx: mpsc::Sender<AppEvent>,
) -> anyhow::Result<()> {
    // Get relay pool client from signer
    let client = Client::builder().signer(signer).build();

    // Add and connect to relay
    client.add_relay("wss://strfry.atlantislabs.space").await?;
    client.connect().await;

    // Subscribe to both server announcement and tools list events
    let filter = Filter::new().kinds(vec![
        Kind::from(SERVER_ANNOUNCEMENT_KIND),
        Kind::from(TOOLS_LIST_KIND),
    ]);

    client.subscribe(filter, None).await?;

    let mut notifications = client.notifications();

    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event { event, .. } = notification {
            match event.kind.as_u16() {
                SERVER_ANNOUNCEMENT_KIND => {
                    // Parse server announcement
                    if let Ok(server_info) = serde_json::from_str::<serde_json::Value>(&event.content) {
                        let agent = DiscoveredAgent {
                            pubkey: event.pubkey,
                            name: server_info["name"]
                                .as_str()
                                .unwrap_or("Unknown")
                                .to_string(),
                            _version: server_info["version"].as_str().map(String::from),
                            about: server_info["about"].as_str().map(String::from),
                            tools: Vec::new(), // Will be populated when tools list arrives
                        };

                        let _ = event_tx.send(AppEvent::AgentDiscovered(agent)).await;
                    }
                }
                TOOLS_LIST_KIND => {
                    // Parse tools list
                    if let Ok(tools_data) = serde_json::from_str::<serde_json::Value>(&event.content) {
                        if let Some(tools_array) = tools_data["tools"].as_array() {
                            let tools: Vec<serde_json::Value> = tools_array.clone();
                            let _ = event_tx.send(AppEvent::ToolsDiscovered {
                                pubkey: event.pubkey,
                                tools,
                            }).await;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    event_rx: &mut mpsc::Receiver<AppEvent>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),      // Messages area
                    Constraint::Length(1),   // Separator
                    Constraint::Length(3),   // Input area
                ])
                .split(f.area());

            // Messages area
            let messages: Vec<ListItem> = app
                .messages
                .iter()
                .rev()
                .take(chunks[0].height as usize)
                .rev()
                .map(|m| {
                    let style = if m.starts_with("🔍") {
                        Style::default().fg(Color::Green)
                    } else if m.starts_with("✓") {
                        Style::default().fg(Color::Cyan)
                    } else if m.starts_with("→") {
                        Style::default().fg(Color::Yellow)
                    } else if m.starts_with("🛠️") {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default()
                    };
                    ListItem::new(m.as_str()).style(style)
                })
                .collect();

            let messages_widget = List::new(messages)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(messages_widget, chunks[0]);

            // Separator
            let separator = Paragraph::new("─".repeat(chunks[1].width as usize))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(separator, chunks[1]);

            // Input area
            let status = if let Some(pubkey) = &app.connected_agent {
                app.discovered_agents
                    .get(pubkey)
                    .map(|a| format!(" [Connected: {}]", a.name))
                    .unwrap_or_else(|| " [Connected]".to_string())
            } else {
                " [No agent connected]".to_string()
            };

            let input_widget = Paragraph::new(Line::from(vec![
                Span::styled(">", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::raw(&app.input),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        status,
                        Style::default().fg(Color::Cyan),
                    ))
            );
            f.render_widget(input_widget, chunks[2]);
        })?;

        // Poll for events with timeout
        let timeout = std::time::Duration::from_millis(100);

        // Check for app events (non-blocking)
        if let Ok(app_event) = event_rx.try_recv() {
            match app_event {
                AppEvent::AgentDiscovered(agent) => {
                    app.handle_agent_discovered(agent);
                }
                AppEvent::ToolsDiscovered { pubkey, tools } => {
                    app.handle_tools_discovered(pubkey, tools);
                }
                AppEvent::Quit => return Ok(()),
            }
        }

        // Check for keyboard input
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            let input = app.input.clone();
                            app.input.clear();
                            if let Some(AppEvent::Quit) = app.handle_command(input) {
                                return Ok(());
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
