use crate::config::CheckCodeConfig;
use crate::extractor::extract_code_blocks_with_propagation;
use crate::language::LanguageRegistry;
use anyhow::{Context, Result};
use chrono::Local;
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tempfile::TempDir;

/// Maximum size of a single code block in bytes (1MB)
const MAX_CODE_BLOCK_SIZE: usize = 1_000_000;

/// Maximum number of code blocks per chapter
const MAX_BLOCKS_PER_CHAPTER: usize = 1000;

/// A configuration-driven mdBook preprocessor that validates code blocks.
///
/// # Overview
///
/// This preprocessor extracts code blocks from markdown chapters and validates
/// them using configured compilers. All language support is configured via
/// `book.toml` - no languages are built-in.
///
/// # Configuration
///
/// Languages are configured in `book.toml` under `[preprocessor.check-code.languages.<name>]`.
///
/// # Example
///
/// ```toml
/// [preprocessor.check-code.languages.c]
/// enabled = true
/// compiler = "gcc"
/// flags = ["-fsyntax-only"]
/// fence_markers = ["c"]
/// ```
///
/// # Security
///
/// The preprocessor validates compiler paths to prevent command injection attacks.
/// Compiler paths cannot contain shell metacharacters (`;`, `|`, `&`, `` ` ``) or
/// use parent directory traversal (`..`).
pub struct CheckCodePreprocessor;

impl CheckCodePreprocessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CheckCodePreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for CheckCodePreprocessor {
    fn name(&self) -> &str {
        "check-code"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // Check if book.toml is approved
        let book_toml_path = ctx.root.join("book.toml");

        if !crate::approval::is_approved(&book_toml_path)? {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S");
            eprintln!(
                "{} [ERROR] (mdbook_check_code): book.toml not approved for code execution",
                timestamp
            );
            eprintln!("{} [ERROR] (mdbook_check_code): ", timestamp);
            eprintln!(
                "{} [ERROR] (mdbook_check_code): For security, mdbook-check-code requires explicit approval before",
                timestamp
            );
            eprintln!(
                "{} [ERROR] (mdbook_check_code): running compilers specified in book.toml.",
                timestamp
            );
            eprintln!("{} [ERROR] (mdbook_check_code): ", timestamp);
            eprintln!(
                "{} [ERROR] (mdbook_check_code): To approve this configuration after reviewing it:",
                timestamp
            );
            eprintln!(
                "{} [ERROR] (mdbook_check_code):   mdbook-check-code allow",
                timestamp
            );
            eprintln!("{} [ERROR] (mdbook_check_code): ", timestamp);
            eprintln!(
                "{} [ERROR] (mdbook_check_code): Current book.toml: {}",
                timestamp,
                book_toml_path.display()
            );
            anyhow::bail!("book.toml not approved");
        }

        // Parse configuration
        let config = CheckCodeConfig::from_preprocessor_context(ctx)?;

        // Build language registry from configuration
        let registry = LanguageRegistry::from_config(&config);

        // Create a temporary directory for compiled files
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        log::info!("Using temporary directory: {:?}", temp_dir.path());

        // Get the book source directory
        let src_dir = ctx.root.join(&ctx.config.book.src);

        let mut failed_files = HashSet::new();
        let mut stats: HashMap<String, usize> = HashMap::new();

        // Process all chapters recursively (including nested ones)
        // Using Book::for_each_mut() is the standard pattern in mdBook preprocessors
        // and automatically handles traversal of chapter.sub_items at all depths
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if let Some(chapter_path) = &chapter.path {
                    let full_path = src_dir.join(chapter_path);

                    log::info!("Checking chapter: {}", chapter.name);

                    // Process code blocks in this chapter
                    if let Err(_e) = process_chapter(
                        &chapter.content,
                        &full_path,
                        &registry,
                        &temp_dir,
                        &mut stats,
                    ) {
                        // Error already printed in process_chapter
                        failed_files.insert(full_path);
                    }
                }
            }
        });

        if !failed_files.is_empty() {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S");

            eprintln!(
                "{} [ERROR] (mdbook_check_code): Failed to compile code in the following files:",
                timestamp
            );
            for file in failed_files {
                eprintln!(
                    "{} [ERROR] (mdbook_check_code):   {}",
                    timestamp,
                    file.display()
                );
            }
            eprintln!(
                "{} [ERROR] (mdbook_check_code): Code compilation failed",
                timestamp
            );
            anyhow::bail!("");
        }

        // Print statistics in mdBook log format (always visible, regardless of log level)
        let total_blocks: usize = stats.values().sum();
        if total_blocks > 0 {
            let mut sorted_stats: Vec<_> = stats.iter().collect();
            sorted_stats.sort_by_key(|(lang, _)| *lang);

            // Format: "c: 2, parasol-c: 3, typescript: 3"
            let stats_str = sorted_stats
                .iter()
                .map(|(lang, count)| format!("{}: {}", lang, count))
                .collect::<Vec<_>>()
                .join(", ");

            // Use mdBook's log format: timestamp [LEVEL] (module): message
            let now = Local::now();
            eprintln!(
                "{} [INFO] (mdbook_check_code): Successfully validated {} code block(s) ({})",
                now.format("%Y-%m-%d %H:%M:%S"),
                total_blocks,
                stats_str
            );
        } else {
            let now = Local::now();
            eprintln!(
                "{} [INFO] (mdbook_check_code): No code blocks found to validate",
                now.format("%Y-%m-%d %H:%M:%S")
            );
        }

        log::info!("All code blocks compiled successfully.");
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

/// Process all code blocks in a chapter
fn process_chapter(
    content: &str,
    chapter_path: &Path,
    registry: &LanguageRegistry,
    temp_dir: &TempDir,
    stats: &mut HashMap<String, usize>,
) -> Result<()> {
    // Extract code blocks with propagation handling
    let code_blocks = extract_code_blocks_with_propagation(content);

    if code_blocks.is_empty() {
        return Ok(());
    }

    // Check if we have too many blocks
    if code_blocks.len() > MAX_BLOCKS_PER_CHAPTER {
        anyhow::bail!(
            "Chapter {} has {} code blocks, exceeding limit of {}",
            chapter_path.display(),
            code_blocks.len(),
            MAX_BLOCKS_PER_CHAPTER
        );
    }

    let chapter_name = chapter_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".md");

    for (i, (final_code, block)) in code_blocks.into_iter().enumerate() {
        // Check code block size limit
        if final_code.len() > MAX_CODE_BLOCK_SIZE {
            anyhow::bail!(
                "Code block #{} in {} exceeds size limit of {} bytes ({} bytes)",
                i,
                chapter_path.display(),
                MAX_CODE_BLOCK_SIZE,
                final_code.len()
            );
        }
        // Find the language implementation (with optional variant)
        let language = match registry.find_by_fence(&block.language, block.variant.as_deref()) {
            Some(lang) => lang,
            None => {
                // Unknown language or variant, skip silently
                continue;
            }
        };

        // Generate a descriptive name for this code block
        let block_name = format!("{}_{}_block_{}", language.name(), chapter_name, i);

        let temp_file_path =
            temp_dir
                .path()
                .join(format!("{}{}", block_name, language.file_extension()));

        log::info!("  Compiling {} block: {}", language.name(), block_name);

        // Compile the code block
        if let Err(e) = language.compile(&final_code, &temp_file_path) {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S");

            eprintln!(
                "{} [ERROR] (mdbook_check_code): Compilation failed",
                timestamp
            );
            eprintln!(
                "{} [ERROR] (mdbook_check_code): File: {}",
                timestamp,
                chapter_path.display()
            );
            eprintln!(
                "{} [ERROR] (mdbook_check_code): Block: #{} ({})",
                timestamp,
                i,
                language.name()
            );
            eprintln!("{} [ERROR] (mdbook_check_code): ", timestamp);

            // Print error with each line prefixed
            let error_msg = format!("{}", e);
            for line in error_msg.lines() {
                eprintln!("{} [ERROR] (mdbook_check_code): {}", timestamp, line);
            }

            eprintln!("{} [ERROR] (mdbook_check_code): ", timestamp);
            eprintln!("{} [ERROR] (mdbook_check_code): Code block:", timestamp);
            eprintln!(
                "{} [ERROR] (mdbook_check_code): ```{}",
                timestamp, block.language
            );

            // Print code with each line prefixed
            for line in final_code.lines() {
                eprintln!("{} [ERROR] (mdbook_check_code): {}", timestamp, line);
            }

            eprintln!("{} [ERROR] (mdbook_check_code): ```", timestamp);

            anyhow::bail!(
                "Failed to compile block #{} ({}) in {}",
                i,
                language.name(),
                chapter_path.display()
            );
        }

        // Update statistics
        *stats.entry(language.name().to_string()).or_insert(0) += 1;
    }

    Ok(())
}
