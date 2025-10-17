use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

/// A code block extracted from markdown with its metadata.
///
/// Code blocks are identified by fenced code syntax in markdown:
///
/// ````markdown
/// ```c
/// int main() { return 0; }
/// ```
/// ````
///
/// # Attributes
///
/// Code blocks can have comma-separated attributes in the fence info string:
///
/// - `ignore` - Skip compilation for this block
/// - `propagate` - Make code available to subsequent blocks in the same file
/// - `variant=<name>` - Use a specific variant of the language
///
/// # Example
///
/// ````markdown
/// ```c,propagate
/// struct Point { int x, y; };
/// ```
///
/// ```c,variant=parasol
/// // Use Parasol C compiler variant
/// [[clang::fhe_program]] uint8_t add(uint8_t a, uint8_t b) { return a + b; }
/// ```
/// ````
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// The programming language from the fence marker (e.g., "c", "typescript", "rust")
    pub language: String,
    /// The actual code content
    pub code: String,
    /// Whether this block should be ignored (skipped during compilation)
    pub ignore: bool,
    /// Whether this block's code should be propagated to subsequent blocks
    pub propagate: bool,
    /// The variant of the language to use (e.g., "parasol" for C)
    pub variant: Option<String>,
}

/// Extracts code blocks from markdown content using pulldown-cmark.
///
/// This function parses markdown and extracts all fenced code blocks with their
/// language and attributes. It does not handle propagation - use
/// [`extract_code_blocks_with_propagation`] for that.
///
/// # Arguments
///
/// * `content` - The markdown content to parse
///
/// # Returns
///
/// A vector of [`CodeBlock`] instances, one for each fenced code block found.
///
/// # Example
///
/// ````ignore
/// let markdown = r#"# My Code
///
/// ```c
/// int main() { return 0; }
/// ```
/// "#;
///
/// let blocks = extract_code_blocks(markdown);
/// assert_eq!(blocks.len(), 1);
/// assert_eq!(blocks[0].language, "c");
/// ````
pub fn extract_code_blocks(content: &str) -> Vec<CodeBlock> {
    let parser = Parser::new(content);
    let mut code_blocks = Vec::new();
    let mut in_code_block = false;
    let mut current_code = String::new();
    let mut current_language = String::new();
    let mut current_ignore = false;
    let mut current_propagate = false;
    let mut current_variant = None;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                in_code_block = true;
                current_code.clear();

                // Parse the fence info string (e.g., "c", "typescript,ignore", "c,variant=parasol")
                let info_str = info.as_ref();
                let (lang, flags, variant) = parse_fence_info(info_str);

                current_language = lang;
                current_ignore = flags.contains(&"ignore");
                current_propagate = flags.contains(&"propagate");
                current_variant = variant;
            }

            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    code_blocks.push(CodeBlock {
                        language: current_language.clone(),
                        code: current_code.clone(),
                        ignore: current_ignore,
                        propagate: current_propagate,
                        variant: current_variant.clone(),
                    });

                    in_code_block = false;
                }
            }

            Event::Text(text) => {
                if in_code_block {
                    current_code.push_str(&text);
                }
            }

            _ => {}
        }
    }

    code_blocks
}

/// Parse fence info string into language, flags, and variant
/// Examples:
/// - "c" -> ("c", [], None)
/// - "typescript,ignore" -> ("typescript", ["ignore"], None)
/// - "c,variant=parasol" -> ("c", [], Some("parasol"))
/// - "c,propagate,variant=parasol" -> ("c", ["propagate"], Some("parasol"))
fn parse_fence_info(info: &str) -> (String, Vec<&str>, Option<String>) {
    let parts: Vec<&str> = info.split(',').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return (String::new(), Vec::new(), None);
    }

    let language = parts[0].to_string();
    let mut flags = Vec::new();
    let mut variant = None;

    // Parse attributes (flags and variant)
    for part in &parts[1..] {
        if let Some(variant_value) = part.strip_prefix("variant=") {
            variant = Some(variant_value.to_string());
        } else {
            flags.push(*part);
        }
    }

    (language, flags, variant)
}

/// Extracts code blocks with propagation support.
///
/// This function handles the `propagate` attribute, which allows code from earlier
/// blocks to be automatically included in later blocks within the same file.
/// Propagation never leaks between different markdown files.
///
/// # Propagation Behavior
///
/// - Blocks marked with `propagate` have their code accumulated
/// - Non-propagated blocks receive all accumulated code as a preamble
/// - Propagated blocks do NOT receive accumulated code (they only contribute)
/// - Blocks marked with `ignore` are skipped entirely
///
/// # Arguments
///
/// * `content` - The markdown content to parse
///
/// # Returns
///
/// A vector of tuples `(final_code, original_block)` where:
/// - `final_code` includes any propagated code prepended
/// - `original_block` is the original code block metadata
///
/// # Example
///
/// ````markdown
/// ```c,propagate
/// struct Point { int x, y; };
/// ```
///
/// ```c
/// // This block will have Point definition prepended
/// struct Point p = {1, 2};
/// ```
/// ````
///
/// The second block will be compiled with:
/// ```c
/// struct Point { int x, y; };
///
/// struct Point p = {1, 2};
/// ```
pub fn extract_code_blocks_with_propagation(content: &str) -> Vec<(String, CodeBlock)> {
    let code_blocks = extract_code_blocks(content);
    let mut result = Vec::new();
    let mut propagated_code = String::new();

    for block in code_blocks {
        if block.ignore {
            continue;
        }

        let mut final_code = String::new();

        // If this is not a propagated block, prepend accumulated propagated code
        if !block.propagate && !propagated_code.is_empty() {
            final_code.push_str(&propagated_code);
            final_code.push('\n');
        }

        final_code.push_str(&block.code);

        // If this is a propagated block, add to accumulated code
        if block.propagate {
            propagated_code.push_str(&block.code);
            propagated_code.push('\n');
        }

        result.push((final_code, block));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_code_block() {
        let markdown = r#"
# Test

```c
int main() {
    return 0;
}
```
"#;

        let blocks = extract_code_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "c");
        assert!(!blocks[0].ignore);
        assert!(!blocks[0].propagate);
        assert_eq!(blocks[0].variant, None);
        assert!(blocks[0].code.contains("int main()"));
    }

    #[test]
    fn test_extract_with_ignore_flag() {
        let markdown = r#"
```c,ignore
This is ignored
```
"#;

        let blocks = extract_code_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].ignore);
    }

    #[test]
    fn test_extract_with_propagate_flag() {
        let markdown = r#"
```c,propagate
typedef struct { int x; } Point;
```

```c
Point p;
```
"#;

        let blocks = extract_code_blocks_with_propagation(markdown);
        assert_eq!(blocks.len(), 2);

        // First block (propagate)
        assert!(blocks[0].1.propagate);

        // Second block should have propagated code prepended
        assert!(blocks[1].0.contains("typedef struct"));
        assert!(blocks[1].0.contains("Point p;"));
    }

    #[test]
    fn test_parse_fence_info() {
        let (lang, flags, variant) = parse_fence_info("c");
        assert_eq!(lang, "c");
        assert!(flags.is_empty());
        assert_eq!(variant, None);

        let (lang, flags, variant) = parse_fence_info("typescript,ignore");
        assert_eq!(lang, "typescript");
        assert_eq!(flags, vec!["ignore"]);
        assert_eq!(variant, None);

        let (lang, flags, variant) = parse_fence_info("c,propagate");
        assert_eq!(lang, "c");
        assert_eq!(flags, vec!["propagate"]);
        assert_eq!(variant, None);

        let (lang, flags, variant) = parse_fence_info("c,variant=parasol");
        assert_eq!(lang, "c");
        assert!(flags.is_empty());
        assert_eq!(variant, Some("parasol".to_string()));

        let (lang, flags, variant) = parse_fence_info("c,propagate,variant=parasol");
        assert_eq!(lang, "c");
        assert_eq!(flags, vec!["propagate"]);
        assert_eq!(variant, Some("parasol".to_string()));
    }
}
