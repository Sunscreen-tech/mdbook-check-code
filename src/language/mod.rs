use crate::config::{CheckCodeConfig, LanguageConfig};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Trait for language-specific compilation and validation
pub trait Language: Send + Sync {
    /// The name of this language (e.g., "parasol-c", "c", "typescript")
    fn name(&self) -> &str;

    /// Fence markers that identify this language in markdown code blocks
    /// (e.g., ["c"], ["typescript", "ts"], ["parasol-c", "parasol"])
    fn fence_markers(&self) -> &[String];

    /// The file extension for this language (e.g., ".c", ".ts")
    fn file_extension(&self) -> &str;

    /// Compile or validate the given code
    ///
    /// # Arguments
    /// * `code` - The source code to validate
    /// * `temp_file` - Path where the code should be written for compilation
    ///
    /// # Returns
    /// Ok(()) if compilation succeeds, Err with details if it fails
    fn compile(&self, code: &str, temp_file: &Path) -> Result<()>;

    /// Optional preamble to prepend to all code blocks
    fn preamble(&self) -> Option<&str> {
        None
    }

    /// Extract identifiers (function names, class names, etc.) from the code
    /// Used for generating descriptive temporary file names
    fn extract_identifiers(&self, _code: &str) -> Vec<String> {
        Vec::new()
    }
}

/// A language configured from book.toml
pub struct ConfiguredLanguage {
    name: String,
    config: LanguageConfig,
}

impl ConfiguredLanguage {
    pub fn new(name: String, config: LanguageConfig) -> Self {
        Self { name, config }
    }

    /// Determine file extension from fence markers (first marker + common extensions)
    fn determine_file_extension(&self) -> String {
        if let Some(first_marker) = self.config.fence_markers.first() {
            match first_marker.as_str() {
                "c" | "parasol-c" | "parasol" => ".c".to_string(),
                "cpp" | "c++" | "cxx" => ".cpp".to_string(),
                "rust" | "rs" => ".rs".to_string(),
                "python" | "py" => ".py".to_string(),
                "javascript" | "js" => ".js".to_string(),
                "typescript" | "ts" => ".ts".to_string(),
                "go" => ".go".to_string(),
                "java" => ".java".to_string(),
                _ => format!(".{}", first_marker),
            }
        } else {
            ".txt".to_string()
        }
    }
}

impl Language for ConfiguredLanguage {
    fn name(&self) -> &str {
        &self.name
    }

    fn fence_markers(&self) -> &[String] {
        &self.config.fence_markers
    }

    fn file_extension(&self) -> &str {
        // We need a static lifetime, so we'll recalculate each time
        // In practice this is called infrequently
        Box::leak(self.determine_file_extension().into_boxed_str())
    }

    fn compile(&self, code: &str, temp_file: &Path) -> Result<()> {
        // Write code with optional preamble to temp file
        let mut file = File::create(temp_file).context("Failed to create temporary file")?;

        if let Some(ref preamble) = self.config.preamble {
            writeln!(file, "{}", preamble)?;
            writeln!(file)?;
        }

        write!(file, "{}", code)?;
        drop(file);

        // Execute compiler with configured flags
        let output = Command::new(&self.config.compiler)
            .args(&self.config.flags)
            .arg(temp_file)
            .output()
            .with_context(|| {
                format!(
                    "Failed to execute compiler '{}' for language '{}'",
                    self.config.compiler, self.name
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            anyhow::bail!("{} compilation failed:\n{}", self.name, error_msg);
        }

        Ok(())
    }

    fn preamble(&self) -> Option<&str> {
        self.config.preamble.as_deref()
    }

    fn extract_identifiers(&self, _code: &str) -> Vec<String> {
        // No identifier extraction - return empty vec
        Vec::new()
    }
}

/// Registry of available languages
pub struct LanguageRegistry {
    languages: Vec<Box<dyn Language>>,
}

impl LanguageRegistry {
    /// Create a new language registry from configuration
    pub fn from_config(config: &CheckCodeConfig) -> Self {
        let mut languages: Vec<Box<dyn Language>> = Vec::new();

        for (name, lang_config) in config.languages() {
            if lang_config.enabled {
                languages.push(Box::new(ConfiguredLanguage::new(
                    name.clone(),
                    lang_config.clone(),
                )));
            }
        }

        Self { languages }
    }

    /// Find a language by its fence marker
    pub fn find_by_fence(&self, fence: &str) -> Option<&dyn Language> {
        self.languages
            .iter()
            .find(|lang| lang.fence_markers().contains(&fence.to_string()))
            .map(|boxed| boxed.as_ref())
    }
}
