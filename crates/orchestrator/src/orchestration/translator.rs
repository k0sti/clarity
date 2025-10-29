// Translator - converts any content into structured textual form

use super::types::{ContentType, TranslatedContent};
use std::path::Path;

/// Translator decodes various content types into structured text
pub struct Translator {
    // Future: Add support for external tools like Whisper for audio
}

impl Translator {
    pub fn new() -> Self {
        Self {}
    }

    /// Translate content from a file path
    pub async fn translate_file(&self, path: impl AsRef<Path>) -> Result<TranslatedContent, TranslatorError> {
        let path = path.as_ref();

        // Detect content type
        let content_type = path
            .extension()
            .and_then(|e| e.to_str())
            .map(ContentType::from_extension)
            .unwrap_or(ContentType::Unknown);

        // Read file
        let raw_content = tokio::fs::read(path)
            .await
            .map_err(|e| TranslatorError::IoError(e.to_string()))?;

        self.translate_bytes(&raw_content, content_type, Some(path))
            .await
    }

    /// Translate raw bytes with a known content type
    pub async fn translate_bytes(
        &self,
        bytes: &[u8],
        content_type: ContentType,
        source: Option<&Path>,
    ) -> Result<TranslatedContent, TranslatorError> {
        let text = match content_type {
            ContentType::Text | ContentType::Code => self.translate_text(bytes)?,
            ContentType::Structured => self.translate_structured(bytes)?,
            ContentType::Document => self.translate_document(bytes)?,
            ContentType::Image => self.translate_image(bytes, source).await?,
            ContentType::Audio => self.translate_audio(bytes, source).await?,
            ContentType::Video => self.translate_video(bytes, source).await?,
            ContentType::Archive => self.translate_archive(bytes)?,
            ContentType::Unknown => self.translate_text(bytes)?,
        };

        let mut translated = TranslatedContent::new(content_type, text);

        // Add source metadata
        if let Some(path) = source {
            translated = translated.with_metadata("source", path.display().to_string());
            translated = translated.with_metadata("filename",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            );
        }

        Ok(translated)
    }

    /// Translate plain text content
    fn translate_text(&self, bytes: &[u8]) -> Result<String, TranslatorError> {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| TranslatorError::EncodingError(e.to_string()))
    }

    /// Translate structured data (JSON, YAML, etc.)
    fn translate_structured(&self, bytes: &[u8]) -> Result<String, TranslatorError> {
        let text = self.translate_text(bytes)?;

        // Try to parse and pretty-print JSON
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            return Ok(serde_json::to_string_pretty(&value)
                .unwrap_or(text));
        }

        // Return as-is if not JSON
        Ok(text)
    }

    /// Translate document formats (PDF, DOCX, etc.)
    fn translate_document(&self, bytes: &[u8]) -> Result<String, TranslatorError> {
        // For now, try to extract any text
        // TODO: Add proper PDF/DOCX parsing libraries
        String::from_utf8_lossy(bytes)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .pipe(Ok)
    }

    /// Translate images (via OCR or description)
    async fn translate_image(&self, _bytes: &[u8], source: Option<&Path>) -> Result<String, TranslatorError> {
        // TODO: Implement OCR or use vision model
        let desc = format!(
            "Image file detected: {}\n\nNote: OCR and vision analysis not yet implemented. \
            This would typically extract text from the image or generate a description.",
            source.and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.img")
        );
        Ok(desc)
    }

    /// Translate audio (via transcription)
    async fn translate_audio(&self, _bytes: &[u8], source: Option<&Path>) -> Result<String, TranslatorError> {
        // TODO: Implement Whisper integration
        let desc = format!(
            "Audio file detected: {}\n\nNote: Audio transcription not yet implemented. \
            This would typically use Whisper or similar to transcribe speech to text.",
            source.and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.mp3")
        );
        Ok(desc)
    }

    /// Translate video (via transcription + scene analysis)
    async fn translate_video(&self, _bytes: &[u8], source: Option<&Path>) -> Result<String, TranslatorError> {
        // TODO: Implement video processing
        let desc = format!(
            "Video file detected: {}\n\nNote: Video analysis not yet implemented. \
            This would typically extract audio transcription and key frame descriptions.",
            source.and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.mp4")
        );
        Ok(desc)
    }

    /// Translate archive contents
    fn translate_archive(&self, _bytes: &[u8]) -> Result<String, TranslatorError> {
        // TODO: Implement archive listing
        Ok("Archive file detected.\n\nNote: Archive extraction not yet implemented. \
            This would typically list contents and extract text from supported files."
            .to_string())
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for pipe syntax
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

#[derive(Debug, thiserror::Error)]
pub enum TranslatorError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_translate_text() {
        let translator = Translator::new();
        let text = b"Hello, world!";
        let result = translator.translate_bytes(text, ContentType::Text, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().text, "Hello, world!");
    }

    #[tokio::test]
    async fn test_translate_json() {
        let translator = Translator::new();
        let json = br#"{"key":"value"}"#;
        let result = translator.translate_bytes(json, ContentType::Structured, None).await;
        assert!(result.is_ok());
        let translated = result.unwrap();
        assert!(translated.text.contains("key"));
        assert!(translated.text.contains("value"));
    }
}
