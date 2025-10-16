use crate::approval::is_approved;
use crate::config::CheckCodeConfig;
use crate::language::LanguageRegistry;
use crate::{compilation, reporting, task_collector};
use anyhow::{Context, Result};
use chrono::Local;
use mdbook::book::Book;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use tempfile::TempDir;

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

impl CheckCodePreprocessor {
    pub async fn run_async(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        eprintln!(
            "{} [INFO] (mdbook_check_code): Preprocessor started",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let book_toml_path = ctx.root.join("book.toml");
        if !is_approved(&book_toml_path)? {
            reporting::report_approval_error(&book_toml_path)?;
            anyhow::bail!("book.toml not approved");
        }

        let config = CheckCodeConfig::from_preprocessor_context(ctx)?;
        let registry = LanguageRegistry::from_config(&config);
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        log::debug!("Using temporary directory: {:?}", temp_dir.path());
        let src_dir = ctx.root.join(&ctx.config.book.src);

        let tasks =
            task_collector::collect_compilation_tasks(&mut book, &src_dir, &registry, &temp_dir)?;

        if tasks.is_empty() {
            log::info!("No code blocks found to validate");
            return Ok(book);
        }

        log::debug!("Collected {} compilation tasks", tasks.len());

        let max_concurrent = get_max_concurrency(config.parallel_jobs);
        log::debug!(
            "Using max_concurrent = {} ({})",
            max_concurrent,
            if config.parallel_jobs.is_some() {
                "configured"
            } else {
                "default"
            }
        );
        let (results, duration) = compilation::compile_tasks(tasks, max_concurrent).await;

        let (_successful, failed): (Vec<_>, Vec<_>) = results.iter().partition(|r| r.success());

        if !failed.is_empty() {
            reporting::report_compilation_errors(&failed)?;
        }

        reporting::print_compilation_statistics(&results, duration);

        log::debug!("Preprocessor completed successfully.");
        Ok(book)
    }
}

impl Preprocessor for CheckCodePreprocessor {
    fn name(&self) -> &str {
        "check-code"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        tokio::runtime::Handle::current().block_on(self.run_async(ctx, book))
    }

    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}

fn get_max_concurrency(parallel_jobs: Option<usize>) -> usize {
    parallel_jobs
        .filter(|&j| j > 0)
        .unwrap_or_else(|| num_cpus::get() * 8) // 8x for I/O-bound subprocess work
}
