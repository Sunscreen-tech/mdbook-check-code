use crate::compilation::CompilationResult;
use anyhow::Result;
use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::path::Path;
use std::time::Duration;

/// Internal helper for printing messages with consistent formatting.
fn print_message<S: Display>(level: &str, message: S) {
    eprintln!(
        "{} [{}] (mdbook_check_code): {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        level,
        message
    );
}

/// Prints an error message to stderr with mdBook-style timestamp and prefix.
pub fn print_error<S: Display>(message: S) {
    print_message("ERROR", message);
}

/// Prints an info message to stderr with mdBook-style timestamp and prefix.
pub fn print_info<S: Display>(message: S) {
    print_message("INFO", message);
}

/// Reports the approval error to stderr with mdBook-style formatting.
pub fn report_approval_error(book_toml_path: &Path) -> Result<()> {
    print_error("book.toml not approved for code execution");
    print_error("");
    print_error("For security, mdbook-check-code requires explicit approval before");
    print_error("running compilers specified in book.toml.");
    print_error("");
    print_error("To approve this configuration after reviewing it:");
    print_error("  mdbook-check-code allow");
    print_error("");
    print_error(format!("Current book.toml: {}", book_toml_path.display()));

    Ok(())
}

/// Reports compilation errors to stderr with mdBook-style formatting.
///
/// # Errors
///
/// Returns an error after printing all failures (to stop the build).
pub fn report_compilation_errors(failed_results: &[&CompilationResult]) -> Result<()> {
    for result in failed_results {
        print_error("Compilation failed");
        print_error(format!("File: {}", result.chapter_path().display()));
        print_error(format!(
            "Block: #{} ({})",
            result.block_index(),
            result.language()
        ));
        print_error("");

        if let Some(error_msg) = result.error_message() {
            for line in error_msg.lines() {
                print_error(line);
            }
        }

        print_error("");
        print_error("Code block:");
        print_error(format!("```{}", result.language()));

        for line in result.code().lines() {
            print_error(line);
        }

        print_error("```");
        print_error("");
    }

    let failed_files: HashSet<_> = failed_results.iter().map(|r| r.chapter_path()).collect();
    print_error("Failed to compile code in the following files:");
    for file in failed_files {
        print_error(format!("  {}", file.display()));
    }
    print_error("Code compilation failed");

    anyhow::bail!("Code compilation failed");
}

/// Prints compilation statistics to stderr.
///
/// Shows:
/// - Total blocks validated with per-language counts
/// - Total time and average time per block
/// - Detailed per-language timing (RUST_LOG=debug)
/// - Individual block timings (RUST_LOG=debug)
pub fn print_compilation_statistics(results: &[CompilationResult], parallel_duration: Duration) {
    let successful_results: Vec<_> = results.iter().filter(|r| r.success()).collect();
    let total_blocks = successful_results.len();

    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    for result in &successful_results {
        *lang_counts
            .entry(result.language().to_string())
            .or_insert(0) += 1;
    }

    let mut sorted_stats: Vec<_> = lang_counts.iter().collect();
    sorted_stats.sort_by_key(|(lang, _)| *lang);

    let stats_str = sorted_stats
        .iter()
        .map(|(lang, count)| format!("{}: {}", lang, count))
        .collect::<Vec<_>>()
        .join(", ");

    let sum_duration: Duration = results.iter().map(|r| r.duration()).sum();
    let sum_ms = sum_duration.as_millis();
    let avg_ms = if !results.is_empty() {
        sum_ms / results.len() as u128
    } else {
        0
    };
    let parallel_ms = parallel_duration.as_millis();

    print_info(format!(
        "Successfully validated {} code block(s) ({})",
        total_blocks,
        stats_str
    ));
    print_info(format!(
        "Preprocessor finished in {}ms (avg {}ms per block)",
        parallel_ms,
        avg_ms
    ));

    log::debug!("Timing breakdown by language:");
    for (lang, count) in sorted_stats {
        let lang_results: Vec<_> = successful_results
            .iter()
            .filter(|r| &r.language().to_string() == lang)
            .collect();
        let lang_total: Duration = lang_results.iter().map(|r| r.duration()).sum();
        let lang_avg_ms = lang_total.as_millis() / *count as u128;
        log::debug!("  {}: avg {}ms over {} blocks", lang, lang_avg_ms, count);
    }

    log::debug!("Individual compilation timings:");
    for result in results {
        log::debug!(
            "[CODE_COMPILE_TIME] [{}] {} block #{}: {}ms",
            result.language(),
            result.chapter_path().display(),
            result.block_index(),
            result.duration().as_millis()
        );
    }
}
