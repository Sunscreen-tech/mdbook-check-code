use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

/// A code block extracted from markdown
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// The programming language (e.g., "c", "typescript", "parasol-c")
    pub language: String,
    /// The actual code content
    pub code: String,
    /// Whether this block should be ignored
    pub ignore: bool,
    /// Whether this block should be propagated to subsequent blocks
    pub propagate: bool,
}

/// Extract code blocks from markdown content using pulldown-cmark
pub fn extract_code_blocks(content: &str) -> Vec<CodeBlock> {
    let parser = Parser::new(content);
    let mut code_blocks = Vec::new();
    let mut in_code_block = false;
    let mut current_code = String::new();
    let mut current_language = String::new();
    let mut current_ignore = false;
    let mut current_propagate = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                in_code_block = true;
                current_code.clear();

                // Parse the fence info string (e.g., "c", "typescript,ignore", "parasol-c,propagate")
                let info_str = info.as_ref();
                let (lang, flags) = parse_fence_info(info_str);

                current_language = lang;
                current_ignore = flags.contains(&"ignore");
                current_propagate = flags.contains(&"propagate");
            }

            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    code_blocks.push(CodeBlock {
                        language: current_language.clone(),
                        code: current_code.clone(),
                        ignore: current_ignore,
                        propagate: current_propagate,
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

/// Parse fence info string into language and flags
/// Examples:
/// - "c" -> ("c", [])
/// - "typescript,ignore" -> ("typescript", ["ignore"])
/// - "parasol-c,propagate" -> ("parasol-c", ["propagate"])
fn parse_fence_info(info: &str) -> (String, Vec<&str>) {
    let parts: Vec<&str> = info.split(',').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return (String::new(), Vec::new());
    }

    let language = parts[0].to_string();
    let flags = parts[1..].to_vec();

    (language, flags)
}

/// Extract code blocks and handle propagation within a file
/// Propagated code is accumulated and prepended to subsequent non-propagated blocks
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
        let (lang, flags) = parse_fence_info("c");
        assert_eq!(lang, "c");
        assert!(flags.is_empty());

        let (lang, flags) = parse_fence_info("typescript,ignore");
        assert_eq!(lang, "typescript");
        assert_eq!(flags, vec!["ignore"]);

        let (lang, flags) = parse_fence_info("parasol-c,propagate");
        assert_eq!(lang, "parasol-c");
        assert_eq!(flags, vec!["propagate"]);
    }
}
