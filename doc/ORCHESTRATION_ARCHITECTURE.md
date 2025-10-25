# Orchestration System Architecture

## Overview

The Clarity orchestration system provides intelligent routing of content to specialized AI experts based on intent and content type. It leverages local LLMs (via Ollama) to analyze incoming requests and dispatch them to the most appropriate expert.

## Architecture Flow

```
┌──────────────┐
│   Content    │
│  (Any Type)  │
└──────┬───────┘
       │
       v
┌──────────────────┐
│   Translator     │  Decodes content into structured textual form
│                  │  - Extracts text from documents
└──────┬───────────┘  - Transcribes audio/video
       │              - OCR for images
       │              - Parses structured data
       v
┌──────────────────┐
│   Orchestrator   │  Uses local LLM to analyze content & intent
│  (LLM-powered)   │  Routes to appropriate expert(s)
└──────┬───────────┘
       │
       v
┌──────┴───────────────────────────────────────┐
│                                               │
v                v               v              v
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ Producer │  │  Artist  │  │  Scribe  │  │  Agent   │  │ Analyst  │
└──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘
     │             │              │             │              │
     v             v              v             v              v
  Artifacts   Media Content   Obsidian MD   Tool Exec    Research
```

## Components

### 1. Translator

**Purpose**: Convert any input content into a structured, analyzable text format.

**Responsibilities**:
- Extract text from documents (PDF, DOCX, etc.)
- Transcribe audio/video files (via Whisper or similar)
- OCR images to extract text
- Parse structured formats (JSON, YAML, XML)
- Normalize content into consistent format
- Preserve metadata (timestamps, file types, sources)

**Output**: `TranslatedContent` struct containing:
- Original content type
- Extracted text
- Metadata (timestamps, file info, etc.)
- Content summary

### 2. Orchestrator

**Purpose**: Intelligently route content to appropriate expert(s) using LLM reasoning.

**Responsibilities**:
- Analyze translated content and user intent
- Consult local LLM to determine routing strategy
- Support multi-expert dispatch (parallel or sequential)
- Track routing decisions for transparency
- Handle expert failures and fallbacks

**Routing Decision Factors**:
- Content type (code, prose, data, media)
- User intent (create, analyze, store, execute)
- Required capabilities (creativity, precision, tools)
- Previous context/conversation history

**LLM Prompt Structure**:
```
System: You are a routing orchestrator. Analyze the content and intent,
then decide which expert(s) should handle this request:
- Producer: Creates files, artifacts, and structured outputs
- Artist: Generates creative content (stories, images, designs)
- Scribe: Documents information in Obsidian vault
- Agent: Executes actions using available tools
- Analyst: Researches topics and provides analysis

User request: {translated_content}
Context: {metadata}

Respond with JSON:
{
  "experts": ["Expert1", "Expert2"],
  "reasoning": "Why these experts",
  "sequence": "parallel" | "sequential"
}
```

### 3. Expert: Producer

**Purpose**: Create and manage artifact files (code, configs, documents).

**Responsibilities**:
- Generate source code files
- Create configuration files
- Write documentation
- Produce structured data files (JSON, YAML, CSV)
- Maintain file integrity and formatting

**Capabilities**:
- Template-based generation
- Code formatting and linting awareness
- Version control integration
- Multi-file project scaffolding

**Example Use Cases**:
- "Create a Rust module for handling webhooks"
- "Generate a docker-compose.yml for this stack"
- "Scaffold a new CLI tool project"

### 4. Expert: Artist

**Purpose**: Generate creative and varied content across all media forms.

**Responsibilities**:
- Write creative prose (stories, poems, essays)
- Generate image descriptions/prompts
- Create ASCII art and diagrams
- Design UI mockups (in text/SVG)
- Compose markdown presentations

**Capabilities**:
- Multiple creative styles and tones
- Visual content representation
- Multimedia content planning
- Brand/style consistency

**Example Use Cases**:
- "Write a sci-fi short story about AI orchestration"
- "Design an ASCII diagram of this system architecture"
- "Create a marketing landing page mockup"

### 5. Expert: Scribe

**Purpose**: Store and organize information in an Obsidian markdown vault.

**Responsibilities**:
- Create new notes with proper frontmatter
- Update existing notes intelligently
- Maintain note relationships (backlinks)
- Organize notes by topic/category
- Apply consistent formatting and tagging

**Obsidian Integration**:
- Respect vault structure
- Use proper wiki-links `[[Note Name]]`
- Add frontmatter (tags, dates, aliases)
- Create daily notes when appropriate
- Link related concepts automatically

**Example Use Cases**:
- "Document this API endpoint in my vault"
- "Create a meeting note for today"
- "Add this code snippet to my Rust learning notes"

### 6. Expert: Agent

**Purpose**: Execute actions using available tools and system capabilities.

**Responsibilities**:
- Run shell commands
- Execute API calls
- Perform file operations
- Interact with external services
- Report results and handle errors

**Tool Categories**:
- System tools (bash, file I/O)
- API tools (HTTP requests, webhooks)
- Data tools (parsing, transformation)
- Integration tools (git, databases)

**Safety**:
- Confirm destructive operations
- Sandbox dangerous commands
- Rate limiting for external APIs
- Audit logging

**Example Use Cases**:
- "Fetch the latest issues from GitHub"
- "Run the test suite and report results"
- "Deploy this code to staging"

### 7. Expert: Analyst

**Purpose**: Research topics and provide in-depth analysis.

**Responsibilities**:
- Gather information from multiple sources
- Synthesize findings into coherent reports
- Compare and contrast options
- Provide recommendations with reasoning
- Generate data visualizations (text-based)

**Capabilities**:
- Web search integration
- Code analysis and review
- Performance profiling interpretation
- Dependency analysis
- Security vulnerability assessment

**Example Use Cases**:
- "Compare Rust async runtimes and recommend one"
- "Analyze the performance bottlenecks in this code"
- "Research best practices for API rate limiting"

## Data Flow

### Input Processing

1. **Raw Input** → Translator
   - File path, URL, text, binary data

2. **Translator** → `TranslatedContent`
   ```rust
   struct TranslatedContent {
       content_type: ContentType,
       text: String,
       metadata: HashMap<String, String>,
       summary: Option<String>,
   }
   ```

3. **TranslatedContent** → Orchestrator
   - LLM analyzes content
   - Determines expert routing

4. **Orchestrator** → Expert(s)
   ```rust
   struct RoutingDecision {
       experts: Vec<ExpertType>,
       reasoning: String,
       execution: ExecutionMode, // Parallel | Sequential
   }
   ```

5. **Expert(s)** → Result
   ```rust
   struct ExpertResult {
       expert: ExpertType,
       output: String,
       artifacts: Vec<Artifact>,
       status: ResultStatus,
   }
   ```

## Configuration

Located in `.clarity.json`:

```json
{
  "orchestrator": {
    "model": "gpt-oss:20b",
    "temperature": 0.7,
    "max_routing_time": 30000,
    "fallback_expert": "Analyst"
  },
  "experts": {
    "producer": {
      "output_dir": "./artifacts",
      "default_language": "rust"
    },
    "scribe": {
      "vault_path": "~/obsidian/vault",
      "default_location": "Clarity"
    },
    "agent": {
      "confirm_destructive": true,
      "allowed_tools": ["bash", "http", "file"]
    }
  }
}
```

## Integration with File Watcher

The orchestration system integrates seamlessly with `clarity-watch`:

```bash
# Watch files and auto-orchestrate
just watch-last
```

When a file changes:
1. File watcher detects modification
2. File content → Translator
3. Translated content → Orchestrator
4. LLM routes to appropriate expert(s)
5. Expert processes and returns result
6. User sees recommendations/outputs

## Implementation Plan

1. ✅ Architecture documentation
2. ⏳ Core traits and types (`src/orchestration/mod.rs`)
3. ⏳ Translator implementation
4. ⏳ Orchestrator with LLM routing
5. ⏳ Individual expert implementations
6. ⏳ CLI integration
7. ⏳ Example/demo

## Example Usage

```rust
use clarity::orchestration::{Translator, Orchestrator};

// Create orchestrator
let translator = Translator::new();
let orchestrator = Orchestrator::new("gpt-oss:20b").await?;

// Process content
let raw_content = std::fs::read_to_string("document.pdf")?;
let translated = translator.translate(raw_content).await?;
let results = orchestrator.process(translated).await?;

// Handle expert outputs
for result in results {
    println!("{}: {}", result.expert, result.output);
}
```

## Benefits

1. **Intelligent Routing**: LLM-powered decision making
2. **Specialization**: Each expert focused on its domain
3. **Extensibility**: Easy to add new experts
4. **Transparency**: Routing decisions are explained
5. **Flexibility**: Supports parallel and sequential execution
6. **Integration**: Works with existing file watcher
