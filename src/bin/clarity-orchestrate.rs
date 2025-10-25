// Clarity Orchestrate - AI orchestration with specialized experts

use clarity::orchestration::{Orchestrator, Translator};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let file_path = PathBuf::from(&args[1]);

    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        return Ok(());
    }

    println!("🔮 Clarity Orchestration System");
    println!("================================\n");

    // Get model from environment or use default
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());
    println!("📡 Using model: {}\n", model);

    // Translate content
    println!("🔄 Translating content...");
    let translator = Translator::new();
    let translated = translator.translate_file(&file_path).await?;

    println!("✓ Content translated");
    println!("  Type: {:?}", translated.content_type);
    println!("  Size: {} bytes\n", translated.text.len());

    // Create orchestrator
    println!("🎯 Analyzing and routing to experts...\n");
    let orchestrator = Orchestrator::new(model);

    // Process through experts
    let results = orchestrator.process(translated).await?;

    println!("\n📊 Results from {} expert(s):\n", results.len());
    println!("================================\n");

    // Display results
    for result in results {
        println!("🤖 Expert: {}", result.expert.as_str());
        println!("📌 Status: {:?}", result.status);
        println!();
        println!("{}", result.output);
        println!();

        if !result.artifacts.is_empty() {
            println!("📦 Artifacts created:");
            for artifact in &result.artifacts {
                println!("  - {} ({})", artifact.name, artifact.artifact_type);
                if let Some(path) = &artifact.path {
                    println!("    Location: {}", path.display());
                }
            }
            println!();
        }

        if let Some(error) = result.error {
            println!("❌ Error: {}", error);
            println!();
        }

        println!("--------------------------------\n");
    }

    println!("✓ Orchestration complete!");

    Ok(())
}

fn print_usage() {
    println!("Clarity Orchestrate - AI orchestration with specialized experts");
    println!();
    println!("Usage: clarity-orchestrate <file>");
    println!();
    println!("The system will:");
    println!("  1. Translate the file content into structured form");
    println!("  2. Use an LLM to determine which experts should handle it");
    println!("  3. Route to appropriate experts (Producer, Artist, Scribe, Agent, Analyst)");
    println!("  4. Return results and any artifacts created");
    println!();
    println!("Environment variables:");
    println!("  OLLAMA_MODEL    - Model to use for routing (default: gpt-oss:20b)");
    println!();
    println!("Examples:");
    println!("  clarity-orchestrate document.md");
    println!("  OLLAMA_MODEL=llama3.1 clarity-orchestrate code.rs");
}
