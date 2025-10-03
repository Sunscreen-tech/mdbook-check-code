use crate::config::{CheckCodeConfig, LanguageConfig};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Get default fence markers for a language based on highlight.js language definitions.
///
/// This function returns the canonical language name plus common aliases that highlight.js
/// recognizes for syntax highlighting. If no built-in mapping exists, returns just the
/// language name itself.
///
/// # Arguments
///
/// * `lang_name` - The language name from the configuration section (e.g., "c", "typescript")
///
/// # Returns
///
/// A vector of fence marker strings that should recognize this language in markdown.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(get_default_fence_markers("c"), vec!["c", "h"]);
/// assert_eq!(get_default_fence_markers("typescript"), vec!["typescript", "ts", "tsx", "mts", "cts"]);
/// assert_eq!(get_default_fence_markers("unknown"), vec!["unknown"]);
/// ```
///
/// # Reference
///
/// Language aliases are based on highlight.js SUPPORTED_LANGUAGES.md:
/// https://github.com/highlightjs/highlight.js/blob/main/SUPPORTED_LANGUAGES.md
pub fn get_default_fence_markers(lang_name: &str) -> Vec<String> {
    match lang_name {
        // System & Shell
        "bash" => vec!["bash", "sh", "zsh"],
        "powershell" => vec!["powershell", "ps", "ps1"],
        "shell" => vec!["shell", "console"],

        // C family
        "c" => vec!["c", "h"],
        "cpp" => vec!["cpp", "hpp", "cc", "hh", "c++", "h++", "cxx", "hxx"],
        "csharp" => vec!["csharp", "cs"],
        "objectivec" => vec!["objectivec", "mm", "objc", "obj-c"],

        // JVM languages
        "java" => vec!["java", "jsp"],
        "kotlin" => vec!["kotlin", "kt"],
        "scala" => vec!["scala"],

        // Web languages
        "javascript" => vec!["javascript", "js", "jsx"],
        "typescript" => vec!["typescript", "ts", "tsx", "mts", "cts"],
        "html" => vec!["html", "xhtml"],
        "xml" => vec!["xml", "rss", "atom", "xsd", "xsl", "plist", "svg"],
        "css" => vec!["css"],
        "json" => vec!["json", "jsonc", "json5"],
        "php" => vec!["php"],

        // Functional languages
        "rust" => vec!["rust", "rs"],
        "go" => vec!["go", "golang"],
        "haskell" => vec!["haskell", "hs"],
        "ocaml" => vec!["ocaml", "ml"],
        "erlang" => vec!["erlang", "erl"],
        "elixir" => vec!["elixir"],
        "fsharp" => vec!["fsharp", "fs", "fsx", "fsi", "fsscript"],
        "elm" => vec!["elm"],

        // Scripting languages
        "python" => vec!["python", "py", "gyp"],
        "ruby" => vec!["ruby", "rb", "gemspec", "podspec", "thor", "irb"],
        "perl" => vec!["perl", "pl", "pm"],
        "lua" => vec!["lua", "pluto"],
        "r" => vec!["r"],

        // Mobile
        "dart" => vec!["dart"],
        "swift" => vec!["swift"],

        // Data & Config
        "yaml" => vec!["yaml", "yml"],
        "toml" => vec!["toml"],
        "sql" => vec!["sql"],
        "graphql" => vec!["graphql", "gql"],

        // Build & DevOps
        "dockerfile" => vec!["dockerfile", "docker"],
        "makefile" => vec!["makefile", "mk", "mak", "make"],
        "nix" => vec!["nix"],

        // Documentation
        "markdown" => vec!["markdown", "md", "mkdown", "mkd"],
        "latex" => vec!["tex"],

        // Blockchain
        "solidity" => vec!["solidity", "sol"],

        // Other
        "zig" => vec!["zig"],
        "matlab" => vec!["matlab"],

        // Default: use the language name itself
        _ => vec![lang_name],
    }
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// A language implementation configured from `book.toml`.
///
/// This struct represents a language whose behavior is entirely determined by
/// configuration rather than hardcoded logic, using the compiler path, flags,
/// and preamble specified in the configuration.
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

    /// Returns the name of this language (e.g., "c", "c-parasol", "typescript").
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the file extension for this language (e.g., ".c", ".ts").
    pub fn file_extension(&self) -> &str {
        &self.file_extension
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
    pub fn compile(&self, code: &str, temp_file: &Path) -> Result<()> {
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
/// if let Some(lang) = registry.find_by_fence("c", None) {
///     lang.compile(code, &temp_file)?;
/// }
/// ```
pub struct LanguageRegistry {
    config: CheckCodeConfig,
}

impl LanguageRegistry {
    /// Creates a new language registry from configuration.
    ///
    /// The registry stores the configuration and creates language instances
    /// on demand when `find_by_fence` is called.
    pub fn from_config(config: &CheckCodeConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Finds a language by its fence marker and optional variant.
    ///
    /// # Arguments
    ///
    /// * `fence` - The fence marker from a markdown code block (e.g., "c", "ts")
    /// * `variant` - Optional variant name (e.g., Some("parasol") for C with Parasol compiler)
    ///
    /// # Returns
    ///
    /// * `Some(Box<dyn Language>)` if a language with this fence marker exists
    /// * `None` if no language is configured for this fence marker
    ///
    /// # Variant Handling
    ///
    /// When a variant is specified, the variant's configuration (compiler, flags, preamble)
    /// overrides the base language configuration. The variant inherits fence_markers and
    /// file_extension from the base language.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Base C language
    /// if let Some(lang) = registry.find_by_fence("c", None) {
    ///     println!("Found language: {}", lang.name());
    /// }
    ///
    /// // Parasol variant of C
    /// if let Some(lang) = registry.find_by_fence("c", Some("parasol")) {
    ///     println!("Found language: {}", lang.name());
    /// }
    /// ```
    pub fn find_by_fence(&self, fence: &str, variant: Option<&str>) -> Option<ConfiguredLanguage> {
        // Find the base language config by fence marker (using resolved fence markers)
        let (lang_name, base_config) = self.config.languages().iter().find(|(name, config)| {
            if !config.enabled {
                return false;
            }
            let fence_markers = config.get_fence_markers(name);
            fence_markers.contains(&fence.to_string())
        })?;

        // If no variant is specified, create base language with resolved fence markers
        let variant_name = match variant {
            None => {
                // Get resolved fence markers for the base language
                let resolved_fence_markers = base_config.get_fence_markers(lang_name);

                // Create config with resolved fence markers
                let resolved_config = crate::config::LanguageConfig {
                    enabled: base_config.enabled,
                    compiler: base_config.compiler.clone(),
                    flags: base_config.flags.clone(),
                    preamble: base_config.preamble.clone(),
                    fence_markers: resolved_fence_markers,
                    variants: base_config.variants.clone(),
                };

                return Some(ConfiguredLanguage::new(lang_name.clone(), resolved_config));
            }
            Some(v) => v,
        };

        // Look up the variant config
        let variant_config = base_config.variants.get(variant_name)?;

        // Get resolved fence markers from base config
        let resolved_fence_markers = base_config.get_fence_markers(lang_name);

        // Create merged config: variant settings override base settings
        let merged_config = crate::config::LanguageConfig {
            enabled: base_config.enabled,
            compiler: variant_config.compiler.clone(),
            flags: variant_config.flags.clone(),
            preamble: variant_config.preamble.clone(),
            fence_markers: resolved_fence_markers,
            variants: std::collections::HashMap::new(), // Variants don't inherit variants
        };

        // Create a new language with variant-specific name
        let variant_lang_name = format!("{}-{}", lang_name, variant_name);
        Some(ConfiguredLanguage::new(variant_lang_name, merged_config))
    }
}
