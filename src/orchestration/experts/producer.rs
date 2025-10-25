// Producer Expert - creates files, artifacts, and structured outputs

use super::{Expert, ExpertError};
use crate::orchestration::types::{Artifact, ExpertResult, ExpertType, ResultStatus, TranslatedContent};
use async_trait::async_trait;
use std::path::PathBuf;

/// Producer creates and manages artifact files
pub struct ProducerExpert {
    output_dir: PathBuf,
}

impl ProducerExpert {
    pub fn new() -> Self {
        Self {
            output_dir: PathBuf::from("./artifacts"),
        }
    }

    pub fn with_output_dir(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Analyze content and determine what artifacts to create
    async fn analyze_and_create(&self, content: &TranslatedContent) -> Result<Vec<Artifact>, ExpertError> {
        let mut artifacts = Vec::new();

        // Determine what to create based on content
        match content.content_type {
            crate::orchestration::types::ContentType::Code => {
                artifacts.push(self.create_code_artifact(content)?);
            }
            crate::orchestration::types::ContentType::Structured => {
                artifacts.push(self.create_config_artifact(content)?);
            }
            crate::orchestration::types::ContentType::Text => {
                artifacts.push(self.create_document_artifact(content)?);
            }
            _ => {
                // Generic artifact
                artifacts.push(self.create_generic_artifact(content)?);
            }
        }

        Ok(artifacts)
    }

    fn create_code_artifact(&self, content: &TranslatedContent) -> Result<Artifact, ExpertError> {
        // Extract filename from metadata or generate one
        let filename = content
            .metadata
            .get("filename")
            .map(|s| s.as_str())
            .unwrap_or("generated.rs");

        let path = self.output_dir.join(filename);

        Ok(Artifact::new(filename, &content.text, "code").with_path(path))
    }

    fn create_config_artifact(&self, content: &TranslatedContent) -> Result<Artifact, ExpertError> {
        let filename = content
            .metadata
            .get("filename")
            .map(|s| s.as_str())
            .unwrap_or("config.json");

        let path = self.output_dir.join(filename);

        Ok(Artifact::new(filename, &content.text, "config").with_path(path))
    }

    fn create_document_artifact(&self, content: &TranslatedContent) -> Result<Artifact, ExpertError> {
        let filename = content
            .metadata
            .get("filename")
            .map(|s| s.as_str())
            .unwrap_or("document.md");

        let path = self.output_dir.join(filename);

        Ok(Artifact::new(filename, &content.text, "document").with_path(path))
    }

    fn create_generic_artifact(&self, content: &TranslatedContent) -> Result<Artifact, ExpertError> {
        let filename = content
            .metadata
            .get("filename")
            .map(|s| s.as_str())
            .unwrap_or("artifact.txt");

        let path = self.output_dir.join(filename);

        Ok(Artifact::new(filename, &content.text, "generic").with_path(path))
    }

    /// Write artifacts to disk
    async fn write_artifacts(&self, artifacts: &[Artifact]) -> Result<(), ExpertError> {
        // Ensure output directory exists
        tokio::fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|e| ExpertError::IoError(format!("Failed to create output dir: {}", e)))?;

        for artifact in artifacts {
            if let Some(path) = &artifact.path {
                tokio::fs::write(path, &artifact.content)
                    .await
                    .map_err(|e| ExpertError::IoError(format!("Failed to write artifact: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl Default for ProducerExpert {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Expert for ProducerExpert {
    async fn process(&self, content: &TranslatedContent) -> Result<ExpertResult, ExpertError> {
        // Analyze content and create artifacts
        let artifacts = self.analyze_and_create(content).await?;

        // Write artifacts to disk
        self.write_artifacts(&artifacts).await?;

        let output = format!(
            "Created {} artifact(s):\n{}",
            artifacts.len(),
            artifacts
                .iter()
                .map(|a| format!("  - {} ({})", a.name, a.artifact_type))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(ExpertResult {
            expert: ExpertType::Producer,
            output,
            artifacts,
            status: ResultStatus::Success,
            error: None,
        })
    }

    fn expert_type(&self) -> ExpertType {
        ExpertType::Producer
    }

    fn capabilities(&self) -> &str {
        "Creates files, artifacts, and structured outputs (code, configs, documents)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_producer_creation() {
        let producer = ProducerExpert::new();
        assert_eq!(producer.expert_type(), ExpertType::Producer);
    }
}
