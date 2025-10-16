use crate::compilation::CompilationTask;
use crate::extractor::extract_code_blocks_with_propagation;
use crate::language::LanguageRegistry;
use anyhow::Result;
use mdbook::book::{Book, BookItem};
use std::path::Path;
use tempfile::TempDir;

/// Maximum size of a single code block in bytes (1MB)
pub const MAX_CODE_BLOCK_SIZE: usize = 1_000_000;

/// Maximum number of code blocks per chapter
pub const MAX_BLOCKS_PER_CHAPTER: usize = 1000;

/// Collects all compilation tasks from the book.
///
/// Iterates through all chapters, extracts code blocks with propagation,
/// validates size limits, and builds CompilationTask instances.
///
/// # Errors
///
/// Returns an error if:
/// - A chapter exceeds MAX_BLOCKS_PER_CHAPTER
/// - A code block exceeds MAX_CODE_BLOCK_SIZE
pub fn collect_compilation_tasks(
    book: &mut Book,
    src_dir: &Path,
    registry: &LanguageRegistry,
    temp_dir: &TempDir,
) -> Result<Vec<CompilationTask>> {
    let mut tasks = Vec::new();
    let mut task_counter = 0;
    let mut collection_errors = Vec::new();

    book.for_each_mut(|item| {
        if let BookItem::Chapter(chapter) = item {
            if let Some(chapter_path) = &chapter.path {
                let full_path = src_dir.join(chapter_path);

                log::debug!("Collecting tasks from chapter: {}", chapter.name);

                let code_blocks = extract_code_blocks_with_propagation(&chapter.content);

                if code_blocks.is_empty() {
                    return;
                }

                if code_blocks.len() > MAX_BLOCKS_PER_CHAPTER {
                    collection_errors.push(format!(
                        "Chapter {} has {} code blocks, exceeding limit of {}",
                        full_path.display(),
                        code_blocks.len(),
                        MAX_BLOCKS_PER_CHAPTER
                    ));
                    return;
                }

                let chapter_name = full_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .trim_end_matches(".md");

                for (i, (final_code, block)) in code_blocks.into_iter().enumerate() {
                    if final_code.len() > MAX_CODE_BLOCK_SIZE {
                        collection_errors.push(format!(
                            "Code block #{} in {} exceeds size limit of {} bytes ({} bytes)",
                            i,
                            full_path.display(),
                            MAX_CODE_BLOCK_SIZE,
                            final_code.len()
                        ));
                        continue;
                    }

                    let language =
                        match registry.find_by_fence(&block.language, block.variant.as_deref()) {
                            Some(lang) => lang,
                            None => {
                                continue;
                            }
                        };

                    let block_name = format!(
                        "{}_{}_block_{}",
                        language.name(),
                        chapter_name,
                        task_counter
                    );
                    task_counter += 1;

                    let temp_file_path = temp_dir.path().join(format!(
                        "{}{}",
                        block_name,
                        language.file_extension()
                    ));

                    tasks.push(CompilationTask::new(
                        language,
                        temp_file_path,
                        chapter_path.clone(),
                        i,
                        final_code,
                    ));
                }
            }
        }
    });

    if !collection_errors.is_empty() {
        for error in &collection_errors {
            log::error!("{}", error);
        }
        anyhow::bail!(
            "Failed to collect compilation tasks due to {} error(s)",
            collection_errors.len()
        );
    }

    Ok(tasks)
}
