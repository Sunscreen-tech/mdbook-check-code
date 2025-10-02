use crate::config::{CheckCodeConfig, LanguageConfig};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Trait for language-specific compilation and validation.
///
/// This trait defines the interface for validating code blocks in different
/// programming languages. Implementations must be thread-safe (`Send + Sync`)
/// to support potential parallel compilation in the future.
///
/// # Example Implementation
///
/// ```ignore
/// struct MyLanguage {
///     name: String,
///     fence_markers: Vec<String>,
///     file_extension: String,
/// }
///
/// impl Language for MyLanguage {
///     fn name(&self) -> &str { &self.name }
///     fn fence_markers(&self) -> &[String] { &self.fence_markers }
///     fn file_extension(&self) -> &str { &self.file_extension }
///     fn compile(&self, code: &str, temp_file: &Path) -> Result<()> {
///         // Validation logic here
///         Ok(())
///     }
/// }
/// ```
pub trait Language: Send + Sync {
    /// Returns the name of this language (e.g., "parasol-c", "c", "typescript").
    fn name(&self) -> &str;

    /// Returns the fence markers that identify this language in markdown.
    ///
    /// Multiple fence markers can map to the same language, e.g.,
    /// `["typescript", "ts"]` for TypeScript.
    fn fence_markers(&self) -> &[String];

    /// Returns the file extension for this language (e.g., ".c", ".ts").
    fn file_extension(&self) -> &str;

    /// Compiles or validates the given code.
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to validate (may include preambles)
    /// * `temp_file` - Path where the code should be written for compilation
    ///
    /// # Returns
    ///
    /// * `Ok(())` if compilation succeeds
    /// * `Err` with compilation error details if it fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The temporary file cannot be created or written
    /// - The compiler executable cannot be found or executed
    /// - The code fails to compile
    fn compile(&self, code: &str, temp_file: &Path) -> Result<()>;

    /// Extracts identifiers (function names, class names, etc.) from the code.
    ///
    /// Used for generating descriptive temporary file names. The default
    /// implementation returns an empty vector.
    fn extract_identifiers(&self, _code: &str) -> Vec<String> {
        Vec::new()
    }
}

/// A language implementation configured from `book.toml`.
///
/// This struct represents a language whose behavior is entirely determined by
/// configuration rather than hardcoded logic. It implements the `Language` trait
/// using the compiler path, flags, and preamble specified in the configuration.
///
/// # Configuration-Driven Design
///
/// All compilation behavior comes from `book.toml`:
/// - Compiler path (with `${VAR}` environment variable expansion)
/// - Compiler flags (array of strings)
/// - Optional preamble (prepended to all blocks)
/// - Fence markers (which markdown fences map to this language)
pub struct ConfiguredLanguage {
    name: String,
    config: LanguageConfig,
    file_extension: String,
}

impl ConfiguredLanguage {
    pub fn new(name: String, config: LanguageConfig) -> Self {
        let file_extension = Self::determine_file_extension(&config.fence_markers);
        Self {
            name,
            config,
            file_extension,
        }
    }

    /// Determine file extension from fence markers (first marker + common extensions)
    fn determine_file_extension(fence_markers: &[String]) -> String {
        if let Some(first_marker) = fence_markers.first() {
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
        &self.file_extension
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

    fn extract_identifiers(&self, _code: &str) -> Vec<String> {
        // No identifier extraction - return empty vec
        Vec::new()
    }
}

/// Registry of available languages for code validation.
///
/// The registry is built from the configuration and provides lookup
/// functionality to find languages by their fence markers.
///
/// # Example
///
/// ```ignore
/// let config = CheckCodeConfig::from_preprocessor_context(&ctx)?;
/// let registry = LanguageRegistry::from_config(&config);
///
/// // Find a language by fence marker
/// if let Some(lang) = registry.find_by_fence("c") {
///     lang.compile(code, &temp_file)?;
/// }
/// ```
pub struct LanguageRegistry {
    languages: Vec<Box<dyn Language>>,
}

impl LanguageRegistry {
    /// Creates a new language registry from configuration.
    ///
    /// Only enabled languages are included in the registry. Each language
    /// is instantiated as a `ConfiguredLanguage` based on its settings
    /// in `book.toml`.
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

    /// Finds a language by its fence marker.
    ///
    /// # Arguments
    ///
    /// * `fence` - The fence marker from a markdown code block (e.g., "c", "ts")
    ///
    /// # Returns
    ///
    /// * `Some(&dyn Language)` if a language with this fence marker exists
    /// * `None` if no language is configured for this fence marker
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(lang) = registry.find_by_fence("parasol-c") {
    ///     println!("Found language: {}", lang.name());
    /// }
    /// ```
    pub fn find_by_fence(&self, fence: &str) -> Option<&dyn Language> {
        self.languages
            .iter()
            .find(|lang| lang.fence_markers().contains(&fence.to_string()))
            .map(|boxed| boxed.as_ref())
    }
}
