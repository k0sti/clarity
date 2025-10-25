// Scribe Expert - stores data in Obsidian markdown vault

use super::{Expert, ExpertError};
use crate::orchestration::types::{Artifact, ExpertResult, ExpertType, ResultStatus, TranslatedContent};
use async_trait::async_trait;
use chrono::Local;
use std::path::PathBuf;

/// Scribe manages Obsidian vault documentation
pub struct ScribeExpert {
    vault_path: PathBuf,
    default_location: String,
}

impl ScribeExpert {
    pub fn new() -> Self {
        let vault_path = dirs::home_dir()
            .map(|h| h.join("obsidian/vault"))
            .unwrap_or_else(|| PathBuf::from("./vault"));

        Self {
            vault_path,
            default_location: "Clarity".to_string(),
        }
    }

    pub fn with_vault(vault_path: PathBuf) -> Self {
        Self {
            vault_path,
            default_location: "Clarity".to_string(),
        }
    }

    /// Create a note in the Obsidian vault
    async fn create_note(&self, content: &TranslatedContent) -> Result<Artifact, ExpertError> {
        let title = self.generate_title(content);
        let note_content = self.format_note(content);
        let note_path = self.determine_note_path(content, &title);

        Ok(Artifact::new(
            title,
            note_content,
            "obsidian_note"
        ).with_path(note_path))
    }

    fn generate_title(&self, content: &TranslatedContent) -> String {
        // Try to extract title from metadata
        if let Some(filename) = content.metadata.get("filename") {
            return filename.trim_end_matches(|c| c == '.' || char::is_alphanumeric(c)).to_string();
        }

        // Generate from first line or summary
        if let Some(summary) = &content.summary {
            let first_line = summary.lines().next().unwrap_or("Note");
            return first_line.chars().take(50).collect();
        }

        // Fallback to timestamp-based title
        format!("Note {}", Local::now().format("%Y-%m-%d %H-%M"))
    }

    fn format_note(&self, content: &TranslatedContent) -> String {
        let mut note = String::new();

        // Add frontmatter
        note.push_str("---\n");
        note.push_str(&format!("created: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        note.push_str(&format!("type: {:?}\n", content.content_type));

        // Add tags
        note.push_str("tags:\n");
        note.push_str("  - clarity\n");
        note.push_str(&format!("  - {:?}\n", content.content_type).to_lowercase());

        // Add metadata as frontmatter
        for (key, value) in &content.metadata {
            note.push_str(&format!("{}: {}\n", key, value));
        }

        note.push_str("---\n\n");

        // Add summary if available
        if let Some(summary) = &content.summary {
            note.push_str("## Summary\n\n");
            note.push_str(summary);
            note.push_str("\n\n");
        }

        // Add main content
        note.push_str("## Content\n\n");
        note.push_str(&content.text);
        note.push_str("\n\n");

        // Add metadata section
        if !content.metadata.is_empty() {
            note.push_str("## Metadata\n\n");
            for (key, value) in &content.metadata {
                note.push_str(&format!("- **{}**: {}\n", key, value));
            }
            note.push_str("\n");
        }

        // Add backlink section
        note.push_str("## Related Notes\n\n");
        note.push_str("- [[Index]]\n");
        note.push_str(&format!("- [[{}]]\n", self.default_location));

        note
    }

    fn determine_note_path(&self, content: &TranslatedContent, title: &str) -> PathBuf {
        let location = content
            .metadata
            .get("vault_location")
            .map(|s| s.as_str())
            .unwrap_or(&self.default_location);

        let sanitized_title = title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
            .collect::<String>();

        self.vault_path
            .join(location)
            .join(format!("{}.md", sanitized_title))
    }

    /// Write note to vault
    async fn write_note(&self, artifact: &Artifact) -> Result<(), ExpertError> {
        if let Some(path) = &artifact.path {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| ExpertError::IoError(format!("Failed to create vault dir: {}", e)))?;
            }

            tokio::fs::write(path, &artifact.content)
                .await
                .map_err(|e| ExpertError::IoError(format!("Failed to write note: {}", e)))?;
        }

        Ok(())
    }
}

impl Default for ScribeExpert {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Expert for ScribeExpert {
    async fn process(&self, content: &TranslatedContent) -> Result<ExpertResult, ExpertError> {
        let artifact = self.create_note(content).await?;

        self.write_note(&artifact).await?;

        let output = format!(
            "Created Obsidian note: {}\nLocation: {}",
            artifact.name,
            artifact.path.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );

        Ok(ExpertResult {
            expert: ExpertType::Scribe,
            output,
            artifacts: vec![artifact],
            status: ResultStatus::Success,
            error: None,
        })
    }

    fn expert_type(&self) -> ExpertType {
        ExpertType::Scribe
    }

    fn capabilities(&self) -> &str {
        "Documents information in Obsidian vault with proper formatting, tags, and backlinks"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scribe_creation() {
        let scribe = ScribeExpert::new();
        assert_eq!(scribe.expert_type(), ExpertType::Scribe);
    }

    #[test]
    fn test_title_generation() {
        let scribe = ScribeExpert::new();
        let content = TranslatedContent::new(
            crate::orchestration::types::ContentType::Text,
            "Test content".to_string()
        );
        let title = scribe.generate_title(&content);
        assert!(!title.is_empty());
    }
}
