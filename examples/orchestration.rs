// Orchestration example - demonstrates the expert routing system

use clarity::orchestration::{ContentType, Orchestrator, TranslatedContent, Translator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔮 Clarity Orchestration Example");
    println!("=================================\n");

    // Get model from environment
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());
    println!("Using model: {}\n", model);

    // Example 1: Translate and process a code snippet
    println!("📝 Example 1: Code Analysis");
    println!("---------------------------");

    let code_content = r#"
// Simple Rust function
pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }
}
"#;

    let translator = Translator::new();
    let code_translated = translator
        .translate_bytes(code_content.as_bytes(), ContentType::Code, None)
        .await?;

    let orchestrator = Orchestrator::new(&model);
    println!("\n🎯 Routing code content to experts...\n");

    let results = orchestrator.process(code_translated).await?;

    for result in results {
        println!("\n🤖 {}: {}", result.expert.as_str(), result.status.as_str());
        println!("{}", result.output);

        if !result.artifacts.is_empty() {
            println!("\n📦 Artifacts: {}", result.artifacts.len());
        }
    }

    println!("\n\n");

    // Example 2: Creative content
    println!("🎨 Example 2: Creative Request");
    println!("------------------------------");

    let creative_content = TranslatedContent::new(
        ContentType::Text,
        "Write a short story about an AI orchestration system that coordinates \
         specialized experts to solve complex problems."
            .to_string(),
    )
    .with_metadata("request_type", "creative");

    println!("\n🎯 Routing creative request to experts...\n");

    let results = orchestrator.process(creative_content).await?;

    for result in results {
        println!("\n🤖 {}: {}", result.expert.as_str(), result.status.as_str());
        println!("{}", result.output);
    }

    println!("\n\n");

    // Example 3: Documentation request
    println!("📚 Example 3: Documentation");
    println!("---------------------------");

    let doc_content = TranslatedContent::new(
        ContentType::Text,
        "Document the concept of AI orchestration:\n\n\
         AI orchestration is a pattern where a central coordinator uses LLM reasoning \
         to route requests to specialized expert agents. Each expert focuses on a specific \
         domain (creation, analysis, documentation, execution, or creativity). This enables \
         more sophisticated problem-solving than a single generalist model."
            .to_string(),
    )
    .with_metadata("request_type", "documentation")
    .with_summary("Concept explanation for AI orchestration");

    println!("\n🎯 Routing documentation request to experts...\n");

    let results = orchestrator.process(doc_content).await?;

    for result in results {
        println!("\n🤖 {}: {}", result.expert.as_str(), result.status.as_str());
        println!("{}", result.output);
    }

    println!("\n\n✓ Orchestration examples complete!");

    Ok(())
}

// Helper trait for status display
trait StatusDisplay {
    fn as_str(&self) -> &str;
}

impl StatusDisplay for clarity::orchestration::ResultStatus {
    fn as_str(&self) -> &str {
        match self {
            clarity::orchestration::ResultStatus::Success => "Success",
            clarity::orchestration::ResultStatus::Partial => "Partial",
            clarity::orchestration::ResultStatus::Failed => "Failed",
        }
    }
}
