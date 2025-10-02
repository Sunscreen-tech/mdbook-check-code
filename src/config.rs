use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Configuration for the check-code preprocessor.
///
/// This structure is deserialized from the `[preprocessor.check-code]` section
/// of `book.toml`. It contains all language-specific configurations.
///
/// # Example
///
/// ```toml
/// [preprocessor.check-code]
///
/// [preprocessor.check-code.languages.c]
/// enabled = true
/// compiler = "gcc"
/// flags = ["-fsyntax-only"]
/// fence_markers = ["c"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CheckCodeConfig {
    /// Language-specific configurations indexed by language name
    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,
}

/// Configuration for a specific language.
///
/// Each language configuration specifies how code blocks should be validated
/// for that language. All fields support environment variable expansion using
/// `${VAR_NAME}` syntax.
///
/// # Security
///
/// Compiler paths are validated to prevent command injection. Paths cannot
/// contain shell metacharacters or use parent directory traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Whether this language is enabled for checking
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Compiler executable (supports ${VAR} environment variable expansion)
    pub compiler: String,

    /// Compiler flags
    #[serde(default)]
    pub flags: Vec<String>,

    /// Optional preamble to prepend to all code blocks
    #[serde(default)]
    pub preamble: Option<String>,

    /// Fence markers that identify this language in markdown
    pub fence_markers: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl LanguageConfig {
    /// Validate the configuration for security and correctness
    pub fn validate(&self) -> Result<()> {
        // Ensure compiler path doesn't contain shell metacharacters
        let dangerous_chars = [';', '|', '&', '`', '\n', '\r'];
        for ch in dangerous_chars {
            if self.compiler.contains(ch) {
                anyhow::bail!(
                    "Compiler path contains invalid character '{}': {}",
                    ch.escape_default(),
                    self.compiler
                );
            }
        }

        // Ensure compiler path doesn't use parent directory traversal
        let compiler_path = Path::new(&self.compiler);
        for component in compiler_path.components() {
            if matches!(component, std::path::Component::ParentDir) {
                anyhow::bail!("Compiler path cannot contain '..': {}", self.compiler);
            }
        }

        // Ensure fence_markers is not empty
        if self.fence_markers.is_empty() {
            anyhow::bail!("Language configuration must have at least one fence marker");
        }

        // Ensure compiler is not empty
        if self.compiler.is_empty() {
            anyhow::bail!("Compiler path cannot be empty");
        }

        Ok(())
    }
}

impl CheckCodeConfig {
    /// Parse configuration from mdbook PreprocessorContext and expand environment variables
    pub fn from_preprocessor_context(
        ctx: &mdbook::preprocess::PreprocessorContext,
    ) -> Result<Self> {
        // Try to get our preprocessor's configuration
        let mut config: CheckCodeConfig =
            if let Some(config_value) = ctx.config.get("preprocessor.check-code") {
                config_value.clone().try_into()?
            } else {
                Self::default()
            };

        // Expand environment variables in all language configs and validate
        for (name, lang_config) in config.languages.iter_mut() {
            lang_config.compiler = expand_env_vars(&lang_config.compiler);
            for flag in lang_config.flags.iter_mut() {
                *flag = expand_env_vars(flag);
            }

            // Validate the configuration for security
            lang_config
                .validate()
                .with_context(|| format!("Invalid configuration for language '{}'", name))?;
        }

        Ok(config)
    }

    /// Get all configured languages
    pub fn languages(&self) -> &HashMap<String, LanguageConfig> {
        &self.languages
    }
}

/// Expand environment variables in a string
/// Supports ${VAR_NAME} syntax
/// This function processes the string in a single pass to avoid re-processing expanded values
fn expand_env_vars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'

            // Collect variable name
            let mut var_name = String::new();
            let mut found_close = false;

            for ch in chars.by_ref() {
                if ch == '}' {
                    found_close = true;
                    break;
                }
                var_name.push(ch);
            }

            if found_close {
                // Try to expand the variable
                match env::var(&var_name) {
                    Ok(value) => result.push_str(&value),
                    Err(_) => {
                        log::warn!(
                            "Environment variable '{}' not found, leaving unexpanded",
                            var_name
                        );
                        result.push_str("${");
                        result.push_str(&var_name);
                        result.push('}');
                    }
                }
            } else {
                // No closing brace found, treat as literal
                result.push_str("${");
                result.push_str(&var_name);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_expand_env_vars_with_var() {
        env::set_var("TEST_VAR", "/usr/bin/test");
        let result = expand_env_vars("${TEST_VAR}/clang");
        assert_eq!(result, "/usr/bin/test/clang");
        env::remove_var("TEST_VAR");
    }

    #[test]
    #[serial]
    fn test_expand_env_vars_without_var() {
        env::remove_var("NONEXISTENT_VAR");
        let result = expand_env_vars("${NONEXISTENT_VAR}");
        assert_eq!(result, "${NONEXISTENT_VAR}");
    }

    #[test]
    fn test_expand_env_vars_no_expansion() {
        let result = expand_env_vars("/usr/bin/gcc");
        assert_eq!(result, "/usr/bin/gcc");
    }
}
