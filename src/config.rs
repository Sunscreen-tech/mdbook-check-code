use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Configuration for the check-code preprocessor
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CheckCodeConfig {
    /// Language-specific configurations
    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,
}

/// Configuration for a specific language
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

impl CheckCodeConfig {
    /// Parse configuration from mdbook PreprocessorContext and expand environment variables
    pub fn from_preprocessor_context(
        ctx: &mdbook::preprocess::PreprocessorContext,
    ) -> Result<Self> {
        // Try to get our preprocessor's configuration
        let mut config: CheckCodeConfig = if let Some(config_value) =
            ctx.config.get("preprocessor.check-code")
        {
            config_value.clone().try_into()?
        } else {
            Self::default()
        };

        // Expand environment variables in all language configs
        for (_name, lang_config) in config.languages.iter_mut() {
            lang_config.compiler = expand_env_vars(&lang_config.compiler);
            for flag in lang_config.flags.iter_mut() {
                *flag = expand_env_vars(flag);
            }
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
fn expand_env_vars(s: &str) -> String {
    let mut result = s.to_string();

    // Find all ${...} patterns and expand them
    while let Some(start) = result.find("${") {
        if let Some(end_offset) = result[start..].find('}') {
            let end = start + end_offset;
            let var_name = &result[start + 2..end];

            // Get the environment variable value
            let value = env::var(var_name).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Environment variable '{}' not found, leaving unexpanded",
                    var_name
                );
                format!("${{{}}}", var_name)
            });

            // Replace the ${VAR} with its value
            result.replace_range(start..=end, &value);
        } else {
            // No closing brace found, stop processing
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_vars_with_var() {
        env::set_var("TEST_VAR", "/usr/bin/test");
        let result = expand_env_vars("${TEST_VAR}/clang");
        assert_eq!(result, "/usr/bin/test/clang");
        env::remove_var("TEST_VAR");
    }

    #[test]
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
