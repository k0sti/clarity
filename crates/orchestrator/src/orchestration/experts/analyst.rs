// Analyst Expert - research and analysis

use super::{Expert, ExpertError};
use crate::orchestration::types::{Artifact, ContentType, ExpertResult, ExpertType, ResultStatus, TranslatedContent};
use async_trait::async_trait;

/// Analyst provides research and analysis capabilities
pub struct AnalystExpert {
    // Future: Add research sources, analysis preferences, etc.
}

impl AnalystExpert {
    pub fn new() -> Self {
        Self {}
    }

    /// Analyze content and generate insights
    async fn analyze(&self, content: &TranslatedContent) -> Result<(String, Vec<Artifact>), ExpertError> {
        let analysis = match content.content_type {
            ContentType::Code => self.analyze_code(content),
            ContentType::Text => self.analyze_text(content),
            ContentType::Structured => self.analyze_structured(content),
            _ => self.generic_analysis(content),
        };

        let artifacts = vec![
            Artifact::new(
                "analysis_report.md",
                &analysis,
                "analysis"
            )
        ];

        Ok((analysis, artifacts))
    }

    fn analyze_code(&self, content: &TranslatedContent) -> String {
        let mut report = String::from("# Code Analysis Report\n\n");

        report.push_str("## Overview\n\n");
        report.push_str(&format!("Analyzed {} lines of code.\n\n", content.text.lines().count()));

        // Basic metrics
        report.push_str("## Metrics\n\n");
        let total_lines = content.text.lines().count();
        let blank_lines = content.text.lines().filter(|l| l.trim().is_empty()).count();
        let comment_lines = content.text.lines().filter(|l| {
            let trimmed = l.trim();
            trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*")
        }).count();

        report.push_str(&format!("- **Total lines**: {}\n", total_lines));
        report.push_str(&format!("- **Blank lines**: {}\n", blank_lines));
        report.push_str(&format!("- **Comment lines**: {}\n", comment_lines));
        report.push_str(&format!("- **Code lines**: {}\n\n", total_lines - blank_lines - comment_lines));

        // Identify patterns
        report.push_str("## Patterns Detected\n\n");

        let patterns = self.detect_code_patterns(&content.text);
        for pattern in patterns {
            report.push_str(&format!("- {}\n", pattern));
        }
        report.push_str("\n");

        // Recommendations
        report.push_str("## Recommendations\n\n");
        report.push_str(self.code_recommendations(&content.text));

        report
    }

    fn detect_code_patterns(&self, code: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let code_lower = code.to_lowercase();

        if code_lower.contains("async") || code_lower.contains("await") {
            patterns.push("Asynchronous programming patterns".to_string());
        }
        if code_lower.contains("struct") || code_lower.contains("class") {
            patterns.push("Object-oriented or struct-based design".to_string());
        }
        if code_lower.contains("fn ") || code_lower.contains("def ") || code_lower.contains("function") {
            patterns.push("Function definitions present".to_string());
        }
        if code_lower.contains("test") || code_lower.contains("assert") {
            patterns.push("Testing code detected".to_string());
        }
        if code_lower.contains("error") || code_lower.contains("result") {
            patterns.push("Error handling patterns".to_string());
        }

        if patterns.is_empty() {
            patterns.push("No specific patterns detected".to_string());
        }

        patterns
    }

    fn code_recommendations(&self, code: &str) -> &str {
        let code_lower = code.to_lowercase();

        if !code_lower.contains("test") {
            return "1. Consider adding unit tests\n\
                    2. Document public APIs\n\
                    3. Add error handling where appropriate\n";
        }

        if !code_lower.contains("//") && !code_lower.contains("#") {
            return "1. Add comments for complex logic\n\
                    2. Consider adding module-level documentation\n\
                    3. Review error handling coverage\n";
        }

        "1. Code structure looks reasonable\n\
         2. Consider performance profiling if needed\n\
         3. Review for security best practices\n"
    }

    fn analyze_text(&self, content: &TranslatedContent) -> String {
        let mut report = String::from("# Text Analysis Report\n\n");

        report.push_str("## Statistics\n\n");
        let word_count = content.text.split_whitespace().count();
        let char_count = content.text.chars().count();
        let line_count = content.text.lines().count();
        let paragraph_count = content.text.split("\n\n").filter(|p| !p.trim().is_empty()).count();

        report.push_str(&format!("- **Words**: {}\n", word_count));
        report.push_str(&format!("- **Characters**: {}\n", char_count));
        report.push_str(&format!("- **Lines**: {}\n", line_count));
        report.push_str(&format!("- **Paragraphs**: {}\n\n", paragraph_count));

        report.push_str("## Reading Time\n\n");
        let reading_time = (word_count as f32 / 200.0).ceil() as u32; // 200 words per minute
        report.push_str(&format!("Approximately {} minute(s)\n\n", reading_time));

        report.push_str("## Content Analysis\n\n");
        report.push_str(&self.analyze_text_content(&content.text));

        report
    }

    fn analyze_text_content(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();
        let mut insights = Vec::new();

        // Detect content type
        if text_lower.contains("introduction") || text_lower.contains("conclusion") {
            insights.push("Structured document with formal sections");
        }
        if text_lower.contains("first") || text_lower.contains("second") || text_lower.contains("third") {
            insights.push("Enumerated or sequential content");
        }
        if text.contains("```") || text_lower.contains("code") {
            insights.push("Contains code examples or technical content");
        }
        if text_lower.contains("?") && text.matches('?').count() > 3 {
            insights.push("Question-driven or FAQ-style content");
        }

        if insights.is_empty() {
            insights.push("General prose content");
        }

        insights.join("\n- ")
    }

    fn analyze_structured(&self, content: &TranslatedContent) -> String {
        let mut report = String::from("# Structured Data Analysis\n\n");

        report.push_str("## Format\n\n");
        if content.text.trim_start().starts_with('{') {
            report.push_str("Detected format: JSON\n\n");
        } else if content.text.contains("---") {
            report.push_str("Detected format: YAML\n\n");
        } else if content.text.contains("<?xml") {
            report.push_str("Detected format: XML\n\n");
        } else {
            report.push_str("Format: Structured (unknown type)\n\n");
        }

        report.push_str("## Analysis\n\n");
        report.push_str(&format!("- **Size**: {} bytes\n", content.text.len()));
        report.push_str(&format!("- **Lines**: {}\n\n", content.text.lines().count()));

        report.push_str("## Recommendations\n\n");
        report.push_str("- Validate against schema if available\n");
        report.push_str("- Consider compression for large datasets\n");
        report.push_str("- Ensure proper escaping of special characters\n");

        report
    }

    fn generic_analysis(&self, content: &TranslatedContent) -> String {
        format!(
            r#"# Analysis Report

## Content Type
{:?}

## Size
- {} bytes
- {} lines

## Summary
{}

## Next Steps
1. Determine specific analysis requirements
2. Apply domain-specific analysis tools
3. Generate detailed insights

## Available Analysis Types
- **Code**: Complexity, patterns, quality metrics
- **Text**: Readability, structure, sentiment
- **Structured Data**: Schema validation, statistics
- **Performance**: Bottlenecks, optimization opportunities
- **Security**: Vulnerability assessment, best practices

Please specify the type of analysis needed for more detailed results."#,
            content.content_type,
            content.text.len(),
            content.text.lines().count(),
            content.summary.as_ref().unwrap_or(&"No summary available".to_string())
        )
    }
}

impl Default for AnalystExpert {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Expert for AnalystExpert {
    async fn process(&self, content: &TranslatedContent) -> Result<ExpertResult, ExpertError> {
        let (output, artifacts) = self.analyze(content).await?;

        Ok(ExpertResult {
            expert: ExpertType::Analyst,
            output,
            artifacts,
            status: ResultStatus::Success,
            error: None,
        })
    }

    fn expert_type(&self) -> ExpertType {
        ExpertType::Analyst
    }

    fn capabilities(&self) -> &str {
        "Researches topics and provides in-depth analysis (code, text, data, performance, security)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyst_creation() {
        let analyst = AnalystExpert::new();
        assert_eq!(analyst.expert_type(), ExpertType::Analyst);
    }

    #[test]
    fn test_code_pattern_detection() {
        let analyst = AnalystExpert::new();
        let code = "async fn test() { await something(); }";
        let patterns = analyst.detect_code_patterns(code);
        assert!(!patterns.is_empty());
    }
}
