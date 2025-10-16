use crate::compilation::CompilationResult;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

/// Formats an error message with mdBook-style timestamp and prefix.
fn format_error<'a>(
    timestamp: &chrono::format::DelayedFormat<chrono::format::StrftimeItems<'a>>,
    message: &str,
) -> String {
    format!("{} [ERROR] (mdbook_check_code): {}", timestamp, message)
}

/// Reports the approval error to stderr with mdBook-style formatting.
pub fn report_approval_error(book_toml_path: &Path) -> Result<()> {
    use chrono::Local;
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");

    eprintln!(
        "{}",
        format_error(&timestamp, "book.toml not approved for code execution")
    );
    eprintln!("{}", format_error(&timestamp, ""));
    eprintln!(
        "{}",
        format_error(
            &timestamp,
            "For security, mdbook-check-code requires explicit approval before"
        )
    );
    eprintln!(
        "{}",
        format_error(&timestamp, "running compilers specified in book.toml.")
    );
    eprintln!("{}", format_error(&timestamp, ""));
    eprintln!(
        "{}",
        format_error(
            &timestamp,
            "To approve this configuration after reviewing it:"
        )
    );
    eprintln!("{}", format_error(&timestamp, "  mdbook-check-code allow"));
    eprintln!("{}", format_error(&timestamp, ""));
    eprintln!(
        "{}",
        format_error(
            &timestamp,
            &format!("Current book.toml: {}", book_toml_path.display())
        )
    );

    Ok(())
}

/// Reports compilation errors to stderr with mdBook-style formatting.
///
/// # Errors
///
/// Returns an error after printing all failures (to stop the build).
pub fn report_compilation_errors(failed_results: &[&CompilationResult]) -> Result<()> {
    use chrono::Local;
    use std::collections::HashSet;

    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");

    for result in failed_results {
        eprintln!("{}", format_error(&timestamp, "Compilation failed"));
        eprintln!(
            "{}",
            format_error(
                &timestamp,
                &format!("File: {}", result.chapter_path().display())
            )
        );
        eprintln!(
            "{}",
            format_error(
                &timestamp,
                &format!(
                    "Block: #{} ({})",
                    result.block_index(),
                    result.language().name()
                )
            )
        );
        eprintln!("{}", format_error(&timestamp, ""));

        if let Some(error_msg) = result.error_message() {
            for line in error_msg.lines() {
                eprintln!("{}", format_error(&timestamp, line));
            }
        }

        eprintln!("{}", format_error(&timestamp, ""));
        eprintln!("{}", format_error(&timestamp, "Code block:"));
        eprintln!(
            "{}",
            format_error(&timestamp, &format!("```{}", result.language().name()))
        );

        for line in result.code().lines() {
            eprintln!("{}", format_error(&timestamp, line));
        }

        eprintln!("{}", format_error(&timestamp, "```"));
        eprintln!("{}", format_error(&timestamp, ""));
    }

    let failed_files: HashSet<_> = failed_results.iter().map(|r| r.chapter_path()).collect();
    eprintln!(
        "{}",
        format_error(&timestamp, "Failed to compile code in the following files:")
    );
    for file in failed_files {
        eprintln!(
            "{}",
            format_error(&timestamp, &format!("  {}", file.display()))
        );
    }
    eprintln!("{}", format_error(&timestamp, "Code compilation failed"));

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
    use chrono::Local;

    let successful_results: Vec<_> = results.iter().filter(|r| r.success()).collect();
    let total_blocks = successful_results.len();

    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    for result in &successful_results {
        *lang_counts
            .entry(result.language().name().to_string())
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

    eprintln!(
        "{} [INFO] (mdbook_check_code): Successfully validated {} code block(s) ({})",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        total_blocks,
        stats_str
    );
    eprintln!(
        "{} [INFO] (mdbook_check_code): Preprocessor finished in {}ms (avg {}ms per block)",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        parallel_ms,
        avg_ms
    );

    log::debug!("Timing breakdown by language:");
    for (lang, count) in sorted_stats {
        let lang_results: Vec<_> = successful_results
            .iter()
            .filter(|r| r.language().name() == lang)
            .collect();
        let lang_total: Duration = lang_results.iter().map(|r| r.duration()).sum();
        let lang_avg_ms = lang_total.as_millis() / *count as u128;
        log::debug!("  {}: avg {}ms over {} blocks", lang, lang_avg_ms, count);
    }

    log::debug!("Individual compilation timings:");
    for result in results {
        log::debug!(
            "[CODE_COMPILE_TIME] [{}] {} block #{}: {}ms",
            result.language().name(),
            result.chapter_path().display(),
            result.block_index(),
            result.duration().as_millis()
        );
    }
}
