use crate::config::CheckCodeConfig;
use crate::extractor::extract_code_blocks_with_propagation;
use crate::language::LanguageRegistry;
use anyhow::{Context, Result};
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::collections::HashSet;
use std::path::PathBuf;
use tempfile::TempDir;

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

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        // Parse configuration
        let config = CheckCodeConfig::from_preprocessor_context(ctx)?;

        // Build language registry from configuration
        let registry = LanguageRegistry::from_config(&config);

        // Create a temporary directory for compiled files
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        eprintln!("Using temporary directory: {:?}", temp_dir.path());

        // Get the book source directory
        let src_dir = ctx.root.join(&ctx.config.book.src);

        let mut failed_files = HashSet::new();

        // Process each chapter in the book
        for section in &book.sections {
            if let BookItem::Chapter(chapter) = section {
                if let Some(chapter_path) = &chapter.path {
                    let full_path = src_dir.join(chapter_path);

                    eprintln!("Checking chapter: {}", chapter.name);

                    // Process code blocks in this chapter
                    if let Err(e) = process_chapter(
                        &chapter.content,
                        &full_path,
                        &registry,
                        &temp_dir,
                    ) {
                        eprintln!("Error processing {}: {}", full_path.display(), e);
                        failed_files.insert(full_path);
                    }
                }
            }
        }

        if !failed_files.is_empty() {
            eprintln!("\nFailed to compile code in the following files:");
            for file in failed_files {
                eprintln!("  {}", file.display());
            }
            anyhow::bail!("Code compilation failed");
        }

        eprintln!("\nAll code blocks compiled successfully.");
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

/// Process all code blocks in a chapter
fn process_chapter(
    content: &str,
    chapter_path: &PathBuf,
    registry: &LanguageRegistry,
    temp_dir: &TempDir,
) -> Result<()> {
    // Extract code blocks with propagation handling
    let code_blocks = extract_code_blocks_with_propagation(content);

    if code_blocks.is_empty() {
        return Ok(());
    }

    let chapter_name = chapter_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".md");

    for (i, (final_code, block)) in code_blocks.into_iter().enumerate() {
        // Find the language implementation
        let language = match registry.find_by_fence(&block.language) {
            Some(lang) => lang,
            None => {
                // Unknown language, skip silently
                continue;
            }
        };

        // Extract identifiers for better naming
        let identifiers = language.extract_identifiers(&block.code);

        // Generate a descriptive name for this code block
        let block_name = if identifiers.is_empty() {
            format!("{}_{}_block_{}", language.name(), chapter_name, i)
        } else {
            let ident_str = identifiers.join("_");
            format!("{}_{}_{}_block_{}", language.name(), chapter_name, ident_str, i)
        };

        let temp_file_path = temp_dir
            .path()
            .join(format!("{}{}", block_name, language.file_extension()));

        eprintln!("  Compiling {} block: {}", language.name(), block_name);

        // Compile the code block
        if let Err(e) = language.compile(&final_code, &temp_file_path) {
            eprintln!("\nFailed to compile code block in file {:?}:", chapter_path);
            eprintln!("{}", e);
            eprintln!("Code was:\n");
            eprintln!("{}\n", final_code);
            anyhow::bail!("Compilation failed");
        }
    }

    Ok(())
}
