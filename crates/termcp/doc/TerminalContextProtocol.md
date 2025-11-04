# Terminal Context Protocol (MCP) Server Implementation Design: "termcp"

The Model Context Protocol enables AI applications to interact with external tools through a standardized interface. This design document outlines **termcp**, a production-ready Rust MCP server that provides read/write access to interactive terminal programs, specifically optimized for LLM CLI tools like Aider, GPTme, and similar AI coding assistants.

## Architecture Overview

### System Design

termcp follows a **stateful client-server architecture** using MCP's JSON-RPC 2.0 protocol over stdio transport. The server maintains persistent terminal sessions that survive across multiple tool invocations, allowing LLMs to execute complex multi-step workflows.

```
┌──────────────────────────────────────┐
│   MCP Host (Claude Desktop/IDE)      │
│   ┌────────────────────────────┐    │
│   │  LLM Client                │    │
│   └────────────┬───────────────┘    │
└────────────────┼────────────────────┘
                 │ JSON-RPC over stdio
┌────────────────┴────────────────────┐
│  termcp MCP Server (Rust)           │
│  ┌──────────────────────────────┐  │
│  │  Protocol Handler            │  │
│  │  (MCP SDK Integration)       │  │
│  └────────┬─────────────────────┘  │
│  ┌────────┴─────────────────────┐  │
│  │  Session Manager             │  │
│  │  - Session lifecycle         │  │
│  │  - State tracking            │  │
│  │  - Timeout management        │  │
│  └────────┬─────────────────────┘  │
│  ┌────────┴─────────────────────┐  │
│  │  PTY Handler                 │  │
│  │  - Process spawning          │  │
│  │  - I/O multiplexing          │  │
│  │  - Escape sequence parsing   │  │
│  └────────┬─────────────────────┘  │
└───────────┼──────────────────────────┘
            │
┌───────────┴──────────────────────────┐
│  Terminal Program (bash/python/etc)  │
└──────────────────────────────────────┘
```

### Core Components

**1. Protocol Handler**
- Implements MCP server protocol using Rust SDK
- Handles initialization, capability negotiation
- Routes tool calls to appropriate handlers
- Manages JSON-RPC request/response lifecycle

**2. Session Manager**
- Maintains HashMap of active sessions (SessionId → Session)
- Enforces session timeouts (default: 20 minutes idle)
- Handles session creation, retrieval, cleanup
- Thread-safe with Arc&lt;Mutex&gt; for concurrent access

**3. PTY Handler**
- Uses `portable-pty` for cross-platform PTY management
- Spawns child processes in PTY environment
- Manages bidirectional I/O (read/write)
- Parses ANSI escape sequences using `vte` crate
- Handles terminal resizing (SIGWINCH)

**4. Terminal Process**
- User-specified command (bash, python, etc.)
- Runs in PTY environment with full terminal capabilities
- Receives input from write operations
- Produces output captured by PTY

## MCP Tool Definitions

termcp exposes **three core tools** that provide complete terminal control for LLM interactions:

### Tool 1: write_stdin

**Purpose**: Send input to the terminal program, simulating typed commands or text.

**Definition**:
```json
{
  "name": "write_stdin",
  "description": "Write text to the terminal's stdin. Use this to send commands or input to the running program. Automatically handles newlines - add \\n to execute commands.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session identifier. Use 'default' for the main session or create named sessions for multiple terminals."
      },
      "text": {
        "type": "string",
        "description": "Text to write to stdin. Include \\n for newlines or command execution."
      }
    },
    "required": ["session_id", "text"]
  }
}
```

**Implementation Details**:
- Converts text to bytes (UTF-8 encoding)
- Writes directly to PTY master writer
- Flushes immediately to ensure delivery
- Non-blocking write with timeout (5 seconds)
- Returns write confirmation or error

**Usage Example**:
```json
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "default",
    "text": "ls -la\n"
  }
}
```

### Tool 2: send_keys

**Purpose**: Send special key sequences (Ctrl+C, arrow keys, function keys) that can't be typed as regular text.

**Definition**:
```json
{
  "name": "send_keys",
  "description": "Send special key sequences to the terminal (Ctrl+C, Ctrl+D, arrow keys, etc.). Use for interrupt signals or navigation.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session identifier"
      },
      "key": {
        "type": "string",
        "enum": [
          "ctrl_c", "ctrl_d", "ctrl_z", "ctrl_l",
          "enter", "tab", "escape",
          "up", "down", "left", "right",
          "home", "end", "pageup", "pagedown",
          "f1", "f2", "f3", "f4", "f5", "f6",
          "f7", "f8", "f9", "f10", "f11", "f12"
        ],
        "description": "Special key to send"
      }
    },
    "required": ["session_id", "key"]
  }
}
```

**Implementation Details**:
- Maps key names to ANSI escape sequences or control characters
- Control characters: Ctrl+C = 0x03, Ctrl+D = 0x04, Ctrl+Z = 0x1A
- Arrow keys: Up = ESC[A, Down = ESC[B, Left = ESC[D, Right = ESC[C
- Function keys: F1 = ESC[11~, F2 = ESC[12~, etc.
- Writes raw bytes directly to PTY

**Key Mapping Table**:
```rust
fn key_to_bytes(key: &str) -> Vec<u8> {
    match key {
        "ctrl_c" => vec![0x03],
        "ctrl_d" => vec![0x04],
        "ctrl_z" => vec![0x1A],
        "enter" => vec![0x0D],
        "tab" => vec![0x09],
        "escape" => vec![0x1B],
        "up" => b"\x1B[A".to_vec(),
        "down" => b"\x1B[B".to_vec(),
        "left" => b"\x1B[D".to_vec(),
        "right" => b"\x1B[C".to_vec(),
        "home" => b"\x1B[H".to_vec(),
        "end" => b"\x1B[F".to_vec(),
        // ... additional mappings
        _ => vec![]
    }
}
```

### Tool 3: read_output

**Purpose**: Read accumulated output from the terminal program since the last read.

**Definition**:
```json
{
  "name": "read_output",
  "description": "Read output from the terminal. Returns all accumulated output since the last read, including both stdout and stderr. Output is raw text including ANSI escape sequences.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {
        "type": "string",
        "description": "Session identifier"
      },
      "timeout_ms": {
        "type": "integer",
        "description": "How long to wait for output (milliseconds). Default: 1000. Use 0 for immediate return.",
        "default": 1000,
        "minimum": 0,
        "maximum": 30000
      },
      "max_bytes": {
        "type": "integer",
        "description": "Maximum bytes to read. Default: 65536 (64KB)",
        "default": 65536,
        "minimum": 1024,
        "maximum": 1048576
      }
    },
    "required": ["session_id"]
  }
}
```

**Implementation Details**:
- Non-blocking read from PTY master reader
- Accumulates output in buffer until timeout or max_bytes reached
- Returns raw UTF-8 string (includes ANSI escape codes)
- Optional ANSI stripping for clean text (configurable)
- Thread-safe buffer management

**Buffer Strategy**:
```rust
struct OutputBuffer {
    data: Vec<u8>,
    max_size: usize,
    last_read: Instant,
}

impl OutputBuffer {
    fn read_with_timeout(&mut self, reader: &mut Reader, timeout: Duration) -> io::Result<String> {
        let start = Instant::now();
        let mut buf = [0u8; 4096];
        
        loop {
            if start.elapsed() >= timeout {
                break;
            }
            
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    self.data.extend_from_slice(&buf[..n]);
                    if self.data.len() >= self.max_size {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        }
        
        String::from_utf8_lossy(&self.data).into_owned()
    }
}
```

### Optional Tool 4: resize_terminal

**Purpose**: Change terminal dimensions (for advanced use cases).

**Definition**:
```json
{
  "name": "resize_terminal",
  "description": "Resize the terminal window. Useful when output formatting depends on terminal size.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_id": {"type": "string"},
      "rows": {
        "type": "integer",
        "description": "Terminal height in rows",
        "minimum": 1,
        "maximum": 500,
        "default": 24
      },
      "cols": {
        "type": "integer",
        "description": "Terminal width in columns",
        "minimum": 1,
        "maximum": 500,
        "default": 80
      }
    },
    "required": ["session_id"]
  }
}
```

**Implementation**:
```rust
fn resize_terminal(session: &mut Session, rows: u16, cols: u16) -> Result<()> {
    let new_size = PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    };
    
    session.pty.resize(new_size)?;
    // Automatically sends SIGWINCH to child process
    Ok(())
}
```

## State Management Approach

### Session Structure

Each terminal session maintains comprehensive state:

```rust
use std::time::{Instant, Duration};
use portable_pty::{PtyPair, Child, MasterPty};
use std::sync::{Arc, Mutex};

struct Session {
    // Core PTY components
    pty_pair: PtyPair,
    child: Child,
    
    // I/O handles
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    
    // Session metadata
    session_id: String,
    command: String,
    created_at: Instant,
    last_activity: Arc<Mutex<Instant>>,
    
    // Terminal state
    terminal_size: PtySize,
    working_dir: PathBuf,
    environment: HashMap<String, String>,
    
    // Output management
    output_buffer: Arc<Mutex<OutputBuffer>>,
    
    // Process state
    exit_status: Arc<Mutex<Option<ExitStatus>>>,
}

impl Session {
    fn new(session_id: String, command: &str, size: PtySize) -> Result<Self> {
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(size)?;
        
        let cmd = CommandBuilder::new(command);
        let child = pty_pair.slave.spawn_command(cmd)?;
        
        let reader = Arc::new(Mutex::new(pty_pair.master.try_clone_reader()?));
        let writer = Arc::new(Mutex::new(pty_pair.master.take_writer()?));
        
        Ok(Session {
            pty_pair,
            child,
            reader,
            writer,
            session_id,
            command: command.to_string(),
            created_at: Instant::now(),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            terminal_size: size,
            working_dir: env::current_dir()?,
            environment: HashMap::new(),
            output_buffer: Arc::new(Mutex::new(OutputBuffer::new())),
            exit_status: Arc::new(Mutex::new(None)),
        })
    }
    
    fn update_activity(&self) {
        *self.last_activity.lock().unwrap() = Instant::now();
    }
    
    fn is_alive(&mut self) -> bool {
        if let Some(status) = *self.exit_status.lock().unwrap() {
            return false;
        }
        
        match self.child.try_wait() {
            Ok(Some(status)) => {
                *self.exit_status.lock().unwrap() = Some(status);
                false
            }
            Ok(None) => true,
            Err(_) => false,
        }
    }
    
    fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.lock().unwrap().elapsed() > timeout
    }
}
```

### Session Manager

The session manager handles lifecycle and cleanup:

```rust
struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<Mutex<Session>>>>>,
    default_timeout: Duration,
    cleanup_interval: Duration,
}

impl SessionManager {
    fn new(timeout_minutes: u64) -> Self {
        let manager = SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            default_timeout: Duration::from_secs(timeout_minutes * 60),
            cleanup_interval: Duration::from_secs(60),
        };
        
        // Spawn cleanup task
        manager.start_cleanup_task();
        manager
    }
    
    fn get_or_create(&self, session_id: &str, command: &str) -> Result<Arc<Mutex<Session>>> {
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(session) = sessions.get(session_id) {
            let mut sess = session.lock().unwrap();
            if sess.is_alive() && !sess.is_timed_out(self.default_timeout) {
                sess.update_activity();
                drop(sess);
                return Ok(Arc::clone(session));
            } else {
                // Session dead or timed out, remove it
                sessions.remove(session_id);
            }
        }
        
        // Create new session
        let session = Session::new(
            session_id.to_string(),
            command,
            PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 }
        )?;
        
        let session_arc = Arc::new(Mutex::new(session));
        sessions.insert(session_id.to_string(), Arc::clone(&session_arc));
        Ok(session_arc)
    }
    
    fn remove_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.remove(session_id) {
            let mut sess = session.lock().unwrap();
            // Cleanup: send SIGTERM, wait briefly, then SIGKILL if needed
            drop(sess.child.kill());
        }
        Ok(())
    }
    
    fn start_cleanup_task(&self) {
        let sessions = Arc::clone(&self.sessions);
        let timeout = self.default_timeout;
        let interval = self.cleanup_interval;
        
        thread::spawn(move || {
            loop {
                thread::sleep(interval);
                
                let mut sessions_lock = sessions.lock().unwrap();
                let mut to_remove = Vec::new();
                
                for (id, session) in sessions_lock.iter() {
                    let mut sess = session.lock().unwrap();
                    if !sess.is_alive() || sess.is_timed_out(timeout) {
                        to_remove.push(id.clone());
                    }
                }
                
                for id in to_remove {
                    sessions_lock.remove(&id);
                }
            }
        });
    }
}
```

### Concurrency Strategy

**Thread-Safe Design**:
- `Arc<Mutex<Session>>` for shared session access
- Separate reader/writer threads to prevent deadlocks
- Non-blocking I/O with timeouts
- Lock hierarchy to prevent deadlocks: SessionManager → Session → Buffer

**Background Reader Task**:
```rust
fn spawn_reader_task(session: Arc<Mutex<Session>>) {
    thread::spawn(move || {
        loop {
            let (reader, buffer, exit_status) = {
                let sess = session.lock().unwrap();
                if sess.exit_status.lock().unwrap().is_some() {
                    break;
                }
                (
                    Arc::clone(&sess.reader),
                    Arc::clone(&sess.output_buffer),
                    Arc::clone(&sess.exit_status)
                )
            };
            
            let mut reader = reader.lock().unwrap();
            let mut buf = [0u8; 4096];
            
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let mut buffer = buffer.lock().unwrap();
                    buffer.append(&buf[..n]);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    drop(reader);
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });
}
```

## Terminal Handling Implementation Details

### PTY Initialization

Using `portable-pty` for cross-platform support:

```rust
use portable_pty::{native_pty_system, CommandBuilder, PtySize, PtySystem};

fn create_pty_session(command: &str) -> Result<(PtyPair, Child)> {
    let pty_system = native_pty_system();
    
    // Create PTY with initial size
    let pty_pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    
    // Build command with proper shell handling
    let mut cmd = CommandBuilder::new(command);
    
    // Set environment variables
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    
    // Spawn process in PTY
    let child = pty_pair.slave.spawn_command(cmd)?;
    
    Ok((pty_pair, child))
}
```

### ANSI Escape Sequence Handling

Using `vte` crate for robust parsing:

```rust
use vte::{Parser, Perform};

struct TerminalEmulator {
    screen: Vec<Vec<Cell>>,
    cursor_x: usize,
    cursor_y: usize,
    // ... other terminal state
}

impl Perform for TerminalEmulator {
    fn print(&mut self, c: char) {
        // Handle printable characters
        self.screen[self.cursor_y][self.cursor_x] = Cell::new(c);
        self.cursor_x += 1;
    }
    
    fn execute(&mut self, byte: u8) {
        // Handle C0 control codes
        match byte {
            0x08 => self.cursor_x = self.cursor_x.saturating_sub(1), // Backspace
            0x09 => self.cursor_x = (self.cursor_x + 8) & !7, // Tab
            0x0A => self.cursor_y += 1, // Line feed
            0x0D => self.cursor_x = 0, // Carriage return
            _ => {}
        }
    }
    
    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, c: char) {
        // Handle CSI sequences (cursor movement, colors, etc.)
        match c {
            'A' => { // Cursor up
                let n = params.get(0).unwrap_or(&1);
                self.cursor_y = self.cursor_y.saturating_sub(*n as usize);
            }
            'B' => { // Cursor down
                let n = params.get(0).unwrap_or(&1);
                self.cursor_y += *n as usize;
            }
            'm' => { // SGR (colors, styles)
                self.apply_sgr(params);
            }
            // ... more sequences
            _ => {}
        }
    }
    
    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        // Handle OSC sequences (DANGEROUS - sanitize carefully)
        // For security, we ignore or strip most OSC sequences
    }
}

// Optional: Strip ANSI codes for clean output
fn strip_ansi_codes(input: &str) -> String {
    let re = regex::Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").unwrap();
    re.replace_all(input, "").to_string()
}
```

### UTF-8 Handling

Proper handling of multi-byte UTF-8 sequences:

```rust
struct Utf8Buffer {
    incomplete: Vec<u8>,
}

impl Utf8Buffer {
    fn push_bytes(&mut self, bytes: &[u8]) -> String {
        self.incomplete.extend_from_slice(bytes);
        
        // Find last valid UTF-8 boundary
        let mut valid_end = self.incomplete.len();
        while valid_end > 0 {
            if let Ok(s) = std::str::from_utf8(&self.incomplete[..valid_end]) {
                let result = s.to_string();
                self.incomplete.drain(..valid_end);
                return result;
            }
            valid_end -= 1;
        }
        
        // All bytes invalid (shouldn't happen with valid UTF-8)
        String::new()
    }
}
```

### Signal Handling

Managing process signals properly:

```rust
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

impl Session {
    fn send_signal(&mut self, signal: Signal) -> Result<()> {
        let pid = Pid::from_raw(self.child.process_id().unwrap() as i32);
        kill(pid, signal)?;
        Ok(())
    }
    
    fn interrupt(&mut self) -> Result<()> {
        self.send_signal(Signal::SIGINT)
    }
    
    fn terminate(&mut self) -> Result<()> {
        // Graceful shutdown sequence
        self.send_signal(Signal::SIGTERM)?;
        
        // Wait up to 5 seconds for graceful exit
        for _ in 0..50 {
            if let Ok(Some(_)) = self.child.try_wait() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        // Force kill if still running
        self.send_signal(Signal::SIGKILL)?;
        self.child.wait()?;
        Ok(())
    }
}
```

## Error Handling Strategies

### Error Type Hierarchy

Comprehensive error types for different failure modes:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TermcpError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Session terminated with exit code {code}")]
    SessionTerminated { code: i32, output: String },
    
    #[error("Session timed out after {0} seconds")]
    SessionTimeout(u64),
    
    #[error("PTY error: {0}")]
    PtyError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

// Convert to MCP-compatible error responses
impl TermcpError {
    fn to_mcp_error(&self) -> McpError {
        McpError {
            code: self.error_code(),
            message: self.to_string(),
            data: serde_json::to_value(self.details()).ok(),
        }
    }
    
    fn error_code(&self) -> i32 {
        match self {
            TermcpError::SessionNotFound(_) => -32001,
            TermcpError::SessionTerminated { .. } => -32002,
            TermcpError::SessionTimeout(_) => -32003,
            TermcpError::PtyError(_) => -32004,
            TermcpError::ResourceLimitExceeded(_) => -32005,
            TermcpError::PermissionDenied(_) => -32006,
            _ => -32000,
        }
    }
}
```

### Error Handling Patterns

**1. Graceful Degradation**:
```rust
fn read_output_with_fallback(session: &mut Session, timeout: Duration) -> Result<String> {
    match session.read_output(timeout) {
        Ok(output) => Ok(output),
        Err(TermcpError::SessionTimeout(_)) => {
            // Timeout is not fatal, return what we have
            Ok(session.output_buffer.lock().unwrap().drain())
        }
        Err(TermcpError::SessionTerminated { output, .. }) => {
            // Process died, return final output
            Ok(output)
        }
        Err(e) => Err(e),
    }
}
```

**2. Retry with Exponential Backoff**:
```rust
async fn write_with_retry(session: &mut Session, text: &str) -> Result<()> {
    let mut delay = Duration::from_millis(100);
    let max_retries = 3;
    
    for attempt in 0..max_retries {
        match session.write_stdin(text) {
            Ok(()) => return Ok(()),
            Err(e) if e.is_transient() && attempt < max_retries - 1 => {
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(TermcpError::IoError(io::Error::new(
        io::ErrorKind::TimedOut,
        "Write failed after retries"
    )))
}
```

**3. Resource Cleanup on Error**:
```rust
impl Drop for Session {
    fn drop(&mut self) {
        // Ensure process is terminated
        let _ = self.terminate();
        
        // Close file descriptors
        drop(&mut self.reader);
        drop(&mut self.writer);
    }
}
```

### Specific Error Scenarios

**Handling Process Exit**:
```rust
impl Session {
    fn check_process_status(&mut self) -> Result<()> {
        if let Some(status) = *self.exit_status.lock().unwrap() {
            return Err(TermcpError::SessionTerminated {
                code: status.exit_code().unwrap_or(-1),
                output: self.output_buffer.lock().unwrap().drain(),
            });
        }
        
        if let Ok(Some(status)) = self.child.try_wait() {
            *self.exit_status.lock().unwrap() = Some(status);
            return Err(TermcpError::SessionTerminated {
                code: status.exit_code().unwrap_or(-1),
                output: self.output_buffer.lock().unwrap().drain(),
            });
        }
        
        Ok(())
    }
}
```

**Handling Timeouts**:
```rust
fn read_with_timeout(reader: &mut Reader, timeout: Duration) -> Result<Vec<u8>> {
    let start = Instant::now();
    let mut buffer = Vec::new();
    
    loop {
        if start.elapsed() >= timeout {
            if buffer.is_empty() {
                return Err(TermcpError::SessionTimeout(timeout.as_secs()));
            }
            return Ok(buffer);
        }
        
        let mut chunk = vec![0u8; 4096];
        match reader.read(&mut chunk) {
            Ok(0) => break, // EOF
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    Ok(buffer)
}
```

**Handling Buffer Overflow**:
```rust
struct BoundedBuffer {
    data: VecDeque<u8>,
    max_size: usize,
}

impl BoundedBuffer {
    fn push(&mut self, bytes: &[u8]) -> Result<()> {
        if self.data.len() + bytes.len() > self.max_size {
            // Drop oldest data to make room
            let overflow = (self.data.len() + bytes.len()) - self.max_size;
            self.data.drain(..overflow);
        }
        
        self.data.extend(bytes);
        Ok(())
    }
}
```

## Example Usage Scenarios with LLM CLIs

### Scenario 1: Interactive Code Debugging with Aider

**Workflow**: LLM uses termcp to run Python code, encounter error, debug it.

```json
// Tool Call 1: Start Python REPL
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "debug_session",
    "text": "python3\n"
  }
}

// Tool Call 2: Read Python startup output
{
  "name": "read_output",
  "arguments": {
    "session_id": "debug_session",
    "timeout_ms": 2000
  }
}
// Response: "Python 3.11.5 ... >>> "

// Tool Call 3: Run code with bug
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "debug_session",
    "text": "def divide(a, b):\n    return a / b\nprint(divide(10, 0))\n"
  }
}

// Tool Call 4: Read error output
{
  "name": "read_output",
  "arguments": {
    "session_id": "debug_session",
    "timeout_ms": 1000
  }
}
// Response: "ZeroDivisionError: division by zero"

// Tool Call 5: Fix the code
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "debug_session",
    "text": "def divide(a, b):\n    return a / b if b != 0 else None\nprint(divide(10, 0))\n"
  }
}

// Tool Call 6: Verify fix
{
  "name": "read_output",
  "arguments": {
    "session_id": "debug_session",
    "timeout_ms": 1000
  }
}
// Response: "None\n>>> "
```

**Benefit**: The LLM maintains a persistent Python session, can test iteratively, see errors in context, and validate fixes immediately.

### Scenario 2: Git Operations with GPTme

**Workflow**: Create feature branch, make changes, commit, push.

```json
// Tool Call 1: Check git status
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "git_workflow",
    "text": "git status\n"
  }
}

// Tool Call 2: Read status
{
  "name": "read_output",
  "arguments": {
    "session_id": "git_workflow",
    "timeout_ms": 1000
  }
}
// Response: "On branch main\nnothing to commit, working tree clean"

// Tool Call 3: Create and switch to feature branch
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "git_workflow",
    "text": "git checkout -b feature/new-api\n"
  }
}

// ... make code changes via file operations ...

// Tool Call 4: Add changes
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "git_workflow",
    "text": "git add .\n"
  }
}

// Tool Call 5: Commit with message
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "git_workflow",
    "text": "git commit -m 'Add new API endpoint'\n"
  }
}

// Tool Call 6: Read commit output
{
  "name": "read_output",
  "arguments": {
    "session_id": "git_workflow",
    "timeout_ms": 2000
  }
}
// Response: "[feature/new-api abc1234] Add new API endpoint\n 2 files changed, 45 insertions(+)"
```

### Scenario 3: Interactive Testing with LLM CLI Tool

**Workflow**: Run tests, see failures, fix code, re-run tests.

```json
// Tool Call 1: Run test suite
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "test_session",
    "text": "pytest tests/ -v\n"
  }
}

// Tool Call 2: Read test results (allow time for tests to run)
{
  "name": "read_output",
  "arguments": {
    "session_id": "test_session",
    "timeout_ms": 10000
  }
}
// Response: "...FAILED tests/test_api.py::test_login - AssertionError: Expected 200, got 401"

// ... LLM analyzes failure, updates code ...

// Tool Call 3: Re-run specific failed test
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "test_session",
    "text": "pytest tests/test_api.py::test_login -v\n"
  }
}

// Tool Call 4: Verify fix
{
  "name": "read_output",
  "arguments": {
    "session_id": "test_session",
    "timeout_ms": 5000
  }
}
// Response: "tests/test_api.py::test_login PASSED"
```

### Scenario 4: Long-Running Process Management

**Workflow**: Start development server, test it, stop it.

```json
// Tool Call 1: Start server
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "server_session",
    "text": "python manage.py runserver\n"
  }
}

// Tool Call 2: Wait for server startup
{
  "name": "read_output",
  "arguments": {
    "session_id": "server_session",
    "timeout_ms": 5000
  }
}
// Response: "Starting development server at http://127.0.0.1:8000/"

// ... perform API tests in separate session ...

// Tool Call 3: Stop server gracefully
{
  "name": "send_keys",
  "arguments": {
    "session_id": "server_session",
    "key": "ctrl_c"
  }
}

// Tool Call 4: Confirm shutdown
{
  "name": "read_output",
  "arguments": {
    "session_id": "server_session",
    "timeout_ms": 2000
  }
}
// Response: "^C\nShutting down server..."
```

### Scenario 5: Interactive CLI Tool Navigation

**Workflow**: Navigate interactive menu-driven CLI tool.

```json
// Tool Call 1: Start interactive tool
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "cli_menu",
    "text": "npm init\n"
  }
}

// Tool Call 2: Read first prompt
{
  "name": "read_output",
  "arguments": {
    "session_id": "cli_menu",
    "timeout_ms": 1000
  }
}
// Response: "package name: (current-dir)"

// Tool Call 3: Enter package name
{
  "name": "write_stdin",
  "arguments": {
    "session_id": "cli_menu",
    "text": "my-project\n"
  }
}

// Tool Call 4: Read next prompt
{
  "name": "read_output",
  "arguments": {
    "session_id": "cli_menu",
    "timeout_ms": 500
  }
}
// Response: "version: (1.0.0)"

// Tool Call 5: Accept default (just press enter)
{
  "name": "send_keys",
  "arguments": {
    "session_id": "cli_menu",
    "key": "enter"
  }
}

// Continue through prompts...
```

## Configuration and CLI Arguments

### Command-Line Interface

```bash
# Start termcp MCP server with default settings
termcp bash

# Start with custom shell
termcp zsh

# Start with Python interpreter
termcp python3

# Start with specific timeout
termcp --session-timeout 30 bash

# Start with custom terminal size
termcp --rows 40 --cols 120 bash

# Enable ANSI stripping (clean output)
termcp --strip-ansi bash

# Enable debug logging
termcp --log-level debug bash
```

### Configuration Structure

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "termcp")]
#[command(about = "MCP server for terminal program interaction")]
struct Config {
    /// Command to execute in terminal
    #[arg(value_name = "COMMAND")]
    command: String,
    
    /// Arguments to pass to command
    #[arg(value_name = "ARGS")]
    args: Vec<String>,
    
    /// Session timeout in minutes
    #[arg(long, default_value = "20")]
    session_timeout: u64,
    
    /// Terminal rows
    #[arg(long, default_value = "24")]
    rows: u16,
    
    /// Terminal columns
    #[arg(long, default_value = "80")]
    cols: u16,
    
    /// Maximum output buffer size in KB
    #[arg(long, default_value = "1024")]
    max_buffer_kb: usize,
    
    /// Strip ANSI escape sequences from output
    #[arg(long, default_value = "false")]
    strip_ansi: bool,
    
    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,
    
    /// Working directory
    #[arg(long)]
    workdir: Option<PathBuf>,
}

impl Config {
    fn full_command(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}
```

## Security Considerations

### Input Validation

**Command Injection Prevention**:
```rust
fn validate_command(command: &str) -> Result<()> {
    // Check for dangerous shell operators
    let dangerous_patterns = [";", "&&", "||", "|", ">", "<", "`", "$("];
    
    for pattern in &dangerous_patterns {
        if command.contains(pattern) {
            return Err(TermcpError::InvalidCommand(
                format!("Command contains dangerous operator: {}", pattern)
            ));
        }
    }
    
    // Validate command exists and is executable
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(TermcpError::InvalidCommand("Empty command".to_string()));
    }
    
    let cmd_path = which::which(parts[0])
        .map_err(|_| TermcpError::InvalidCommand(
            format!("Command not found: {}", parts[0])
        ))?;
    
    // Check execute permissions (Unix-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&cmd_path)?;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(TermcpError::PermissionDenied(
                format!("Command not executable: {}", cmd_path.display())
            ));
        }
    }
    
    Ok(())
}
```

**Escape Sequence Sanitization**:
```rust
fn sanitize_output(output: &str, strip_ansi: bool) -> String {
    if strip_ansi {
        strip_ansi_codes(output)
    } else {
        // Remove dangerous OSC sequences
        let dangerous_osc = regex::Regex::new(r"\x1B\].*?(\x07|\x1B\\)").unwrap();
        dangerous_osc.replace_all(output, "").to_string()
    }
}
```

### Resource Limits

**Per-Session Limits**:
```rust
struct ResourceLimits {
    max_output_buffer: usize,  // 1MB default
    max_session_duration: Duration,  // 24 hours
    max_memory_mb: usize,  // 512MB per session
    max_cpu_percent: f32,  // 50% CPU
}

impl Session {
    fn enforce_limits(&mut self, limits: &ResourceLimits) -> Result<()> {
        // Check buffer size
        let buffer_size = self.output_buffer.lock().unwrap().len();
        if buffer_size > limits.max_output_buffer {
            return Err(TermcpError::ResourceLimitExceeded(
                format!("Output buffer exceeded {} bytes", limits.max_output_buffer)
            ));
        }
        
        // Check session duration
        if self.created_at.elapsed() > limits.max_session_duration {
            return Err(TermcpError::ResourceLimitExceeded(
                "Session exceeded maximum duration".to_string()
            ));
        }
        
        // Check process memory (platform-specific)
        #[cfg(target_os = "linux")]
        {
            let pid = self.child.process_id().unwrap();
            if let Ok(memory_kb) = get_process_memory(pid) {
                let memory_mb = memory_kb / 1024;
                if memory_mb > limits.max_memory_mb {
                    self.terminate()?;
                    return Err(TermcpError::ResourceLimitExceeded(
                        format!("Process exceeded {}MB memory", limits.max_memory_mb)
                    ));
                }
            }
        }
        
        Ok(())
    }
}
```

### Permissions and Isolation

**Principle of Least Privilege**:
```rust
fn setup_restricted_environment(cmd: &mut CommandBuilder) {
    // Set minimal environment
    cmd.clear_env();
    cmd.env("PATH", "/usr/bin:/bin");
    cmd.env("TERM", "xterm-256color");
    
    // Drop privileges if running as root (not recommended)
    #[cfg(unix)]
    {
        if nix::unistd::Uid::effective().is_root() {
            eprintln!("WARNING: Running as root is not recommended");
            // Optionally drop to specific user
        }
    }
}
```

## Dependencies and Project Structure

### Cargo.toml

```toml
[package]
name = "termcp"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "MCP server for terminal program interaction"
license = "MIT"

[dependencies]
# MCP SDK
mcp-server = "0.1"  # Official Rust MCP SDK

# PTY handling
portable-pty = "0.8"

# Terminal control
crossterm = "0.27"

# ANSI parsing
vte = "0.13"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# CLI parsing
clap = { version = "4.4", features = ["derive"] }

# Utilities
regex = "1.10"
which = "5.0"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.8"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### Project Structure

```
termcp/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── src/
│   ├── main.rs              # Entry point, CLI parsing, MCP server setup
│   ├── lib.rs               # Library interface
│   ├── config.rs            # Configuration structures
│   ├── session.rs           # Session struct and management
│   ├── session_manager.rs   # SessionManager implementation
│   ├── pty.rs               # PTY handling logic
│   ├── terminal.rs          # Terminal emulation and ANSI parsing
│   ├── tools.rs             # MCP tool implementations
│   ├── error.rs             # Error types and conversions
│   ├── buffer.rs            # Output buffer management
│   └── utils.rs             # Helper functions
├── tests/
│   ├── integration_test.rs  # End-to-end tests
│   ├── session_test.rs      # Session tests
│   └── tools_test.rs        # Tool tests
└── examples/
    ├── basic_usage.rs       # Simple example
    └── interactive.rs       # Interactive session example
```

### Main Entry Point

```rust
// src/main.rs
use anyhow::Result;
use clap::Parser;
use mcp_server::{Server, ServerOptions};
use tracing_subscriber;

mod config;
mod session_manager;
mod tools;
mod error;

use config::Config;
use session_manager::SessionManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let config = Config::parse();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();
    
    tracing::info!("Starting termcp MCP server");
    tracing::info!("Command: {}", config.full_command());
    
    // Create session manager
    let session_manager = SessionManager::new(
        config.session_timeout,
        config.rows,
        config.cols,
        config.max_buffer_kb * 1024,
        config.strip_ansi,
    );
    
    // Create MCP server
    let server = Server::new(ServerOptions {
        name: "termcp".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    });
    
    // Register tools
    tools::register_tools(&server, session_manager, &config)?;
    
    // Start server (stdio transport)
    server.run_stdio().await?;
    
    Ok(())
}
```

## Implementation Roadmap

### Phase 1: Core Functionality (Week 1-2)
- Set up project structure with Cargo
- Implement basic PTY handling with portable-pty
- Create Session and SessionManager structures
- Implement write_stdin tool
- Implement read_output tool with basic timeout
- Basic error handling

### Phase 2: Advanced Features (Week 3)
- Implement send_keys tool with full key mapping
- Add ANSI escape sequence parsing with vte
- Implement resize_terminal tool
- Add UTF-8 handling for multi-byte sequences
- Enhanced error types and handling

### Phase 3: MCP Integration (Week 4)
- Integrate MCP Rust SDK
- Implement JSON-RPC message handling
- Add proper tool registration and discovery
- Implement initialization and capability negotiation
- Test with Claude Desktop and other MCP clients

### Phase 4: Production Readiness (Week 5-6)
- Comprehensive error handling and recovery
- Resource limits and monitoring
- Security hardening (input validation, sanitization)
- Performance optimization (buffer management, lock contention)
- Extensive testing (unit, integration, chaos)
- Documentation and examples

### Phase 5: Polish (Week 7)
- CLI improvements and configuration options
- Logging and observability
- Package for distribution (cargo install, homebrew, etc.)
- Create integration examples with popular LLM tools
- User documentation and tutorials

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        let session = Session::new("test".to_string(), "echo hello", 
                                   PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .expect("Failed to create session");
        assert!(session.is_alive());
    }
    
    #[test]
    fn test_write_and_read() {
        let mut session = Session::new("test".to_string(), "cat", 
                                      PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .unwrap();
        
        session.write_stdin("hello\n").unwrap();
        thread::sleep(Duration::from_millis(100));
        
        let output = session.read_output(Duration::from_secs(1)).unwrap();
        assert!(output.contains("hello"));
    }
    
    #[test]
    fn test_session_timeout() {
        let manager = SessionManager::new(0); // 0 minute timeout
        let session = manager.get_or_create("test", "bash").unwrap();
        
        thread::sleep(Duration::from_secs(2));
        
        // Session should be removed after timeout
        assert!(manager.get_session("test").is_none());
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_mcp_server_initialization() {
    let mut cmd = Command::cargo_bin("termcp").unwrap();
    let output = cmd.arg("bash")
        .write_stdin(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#)
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("\"result\""));
}

#[test]
fn test_write_stdin_tool() {
    let mut cmd = Command::cargo_bin("termcp").unwrap();
    cmd.arg("bash")
        .write_stdin(r#"
            {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}
            {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"write_stdin","arguments":{"session_id":"default","text":"echo test\n"}}}
        "#)
        .assert()
        .success();
}
```

## Performance Considerations

### Optimization Strategies

**1. Buffer Management**:
- Use ring buffers (VecDeque) for efficient FIFO operations
- Pre-allocate buffers to avoid frequent allocations
- Implement zero-copy reads where possible

**2. Lock Contention**:
- Minimize lock hold time
- Use separate locks for reader/writer
- Consider lock-free data structures for high-frequency operations

**3. I/O Efficiency**:
- Use non-blocking I/O with tokio
- Batch reads to reduce syscalls
- Implement backpressure to prevent memory bloat

**4. Memory Management**:
- Implement bounded buffers with overflow policies
- Release memory from closed sessions promptly
- Use weak references for background tasks

### Benchmarks

```rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_write_throughput(c: &mut Criterion) {
        c.bench_function("write_1kb", |b| {
            let mut session = create_test_session();
            let data = "x".repeat(1024);
            
            b.iter(|| {
                session.write_stdin(black_box(&data)).unwrap();
            });
        });
    }
    
    fn benchmark_read_throughput(c: &mut Criterion) {
        c.bench_function("read_output", |b| {
            let mut session = create_test_session();
            
            b.iter(|| {
                session.read_output(Duration::from_millis(10)).unwrap();
            });
        });
    }
    
    criterion_group!(benches, benchmark_write_throughput, benchmark_read_throughput);
    criterion_main!(benches);
}
```

## Deployment and Usage

### Installation

```bash
# From source
git clone https://github.com/yourusername/termcp
cd termcp
cargo install --path .

# From crates.io (when published)
cargo install termcp
```

### Claude Desktop Configuration

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "termcp-bash": {
      "command": "termcp",
      "args": ["bash"],
      "env": {}
    },
    "termcp-python": {
      "command": "termcp",
      "args": ["--session-timeout", "30", "python3"],
      "env": {
        "PYTHONUNBUFFERED": "1"
      }
    }
  }
}
```

### Programmatic Usage

```rust
use termcp::{SessionManager, Session};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = SessionManager::new(20, 24, 80, 1024*1024, false);
    
    let session = manager.get_or_create("my_session", "bash")?;
    let mut sess = session.lock().unwrap();
    
    // Execute command
    sess.write_stdin("ls -la\n")?;
    
    // Read output
    tokio::time::sleep(Duration::from_millis(500)).await;
    let output = sess.read_output(Duration::from_secs(1))?;
    
    println!("Output: {}", output);
    
    Ok(())
}
```

## Conclusion

This comprehensive design provides a production-ready foundation for implementing **termcp**, a Terminal Context Protocol server in Rust. The architecture balances simplicity with robustness, leveraging battle-tested libraries like `portable-pty` for cross-platform PTY handling and `vte` for ANSI parsing.

**Key Design Decisions**:

1. **Stateful sessions** with persistent PTY connections enable multi-turn LLM interactions
2. **Three-tool interface** (write_stdin, send_keys, read_output) provides complete terminal control while remaining simple
3. **Session manager** handles lifecycle, timeouts, and cleanup automatically
4. **Comprehensive error handling** ensures graceful degradation and clear error messages
5. **Security-first approach** with input validation, escape sequence sanitization, and resource limits
6. **MCP integration** using JSON-RPC 2.0 provides standardized LLM tool access

**Primary Use Case**: Enabling AI coding assistants (Aider, GPTme, LLM CLI tools) to interact with terminal programs for tasks like testing, debugging, git operations, and running build tools—all within persistent, stateful sessions that preserve context across multiple operations.

The implementation provides the foundation for advanced LLM-terminal interactions while maintaining security, performance, and reliability suitable for production use.