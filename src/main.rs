use anyhow::{Context, Result};
use mdbook::book::Book;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, Write as _};
use std::path::PathBuf;
use std::process::{Command, exit};
use tempfile::TempDir;

pub fn main() {
    // Handle mdbook preprocessor commands
    let mut args = std::env::args().skip(1);

    if let Some(arg) = args.next() {
        if arg == "supports" {
            // Check if we support the renderer
            if let Some(renderer) = args.next() {
                // We support all renderers
                if renderer == "not-supported" {
                    exit(1);
                }
                exit(0);
            }
        }
    }

    // Run as preprocessor
    if let Err(e) = handle_preprocessing() {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let processed_book = CheckParasolPreprocessor.run(&ctx, book)?;

    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

struct CheckParasolPreprocessor;

impl Preprocessor for CheckParasolPreprocessor {
    fn name(&self) -> &str {
        "check-parasol"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        // Read the clang binary path from environment variable or default to "clang"
        let clang_binary = env::var("CLANG").unwrap_or_else(|_| "clang".to_string());
        let clang_binary_path = PathBuf::from(&clang_binary);
        eprintln!("Using clang binary: {:?}", clang_binary_path);

        // Create a temporary directory for compiled files
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        eprintln!("Using temporary directory: {:?}", temp_dir.path());

        // Get the book source directory
        let src_dir = ctx.root.join(&ctx.config.book.src);

        let mut failed_files = HashSet::new();

        // Process each chapter in the book
        for section in &book.sections {
            if let mdbook::BookItem::Chapter(chapter) = section {
                if let Some(chapter_path) = &chapter.path {
                    let full_path = src_dir.join(chapter_path);

                    eprintln!("Checking chapter: {}", chapter.name);

                    // Extract and compile C code blocks
                    if let Err(e) = process_chapter_code_blocks(
                        &chapter.content,
                        &full_path,
                        &clang_binary_path,
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

/// Process all C code blocks in a chapter
fn process_chapter_code_blocks(
    content: &str,
    chapter_path: &PathBuf,
    clang_binary_path: &PathBuf,
    temp_dir: &TempDir,
) -> Result<()> {
    let code_blocks = extract_c_code_blocks(content);

    if code_blocks.is_empty() {
        return Ok(());
    }

    let chapter_name = chapter_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".md");

    for (i, (code_block, function_names)) in code_blocks.into_iter().enumerate() {
        let block_name = if function_names.is_empty() {
            format!("code_block_{chapter_name}_{i}")
        } else {
            let func_names = function_names.join("_");
            format!("code_block_{chapter_name}_{func_names}_{i}")
        };

        let temp_file_path = temp_dir.path().join(format!("{block_name}.c"));
        let mut temp_file = File::create(&temp_file_path)
            .context("Failed to create temporary C file")?;

        // Write the code block to the temporary file
        // Note: No preamble needed - compiler now has #include<parasol.h>
        writeln!(temp_file, "{}", code_block)?;

        eprintln!("  Compiling {block_name}");

        run_clang(clang_binary_path, &temp_file_path, chapter_path)?;
    }

    Ok(())
}

/// Extracts C code blocks from a Markdown file content.
/// Ignores code blocks with the `ignore` flag.
/// Returns a vector of tuples containing the code block and a list of function names.
fn extract_c_code_blocks(content: &str) -> Vec<(String, Vec<String>)> {
    let mut code_blocks = Vec::new();
    let mut in_code_block = false;
    let mut in_propagated_block = false;
    let mut current_block = String::new();
    let mut function_names = Vec::new();

    let function_regex =
        Regex::new(r"\[\[clang::fhe_program\]\]\s+[\w\s\*]+\s+(\w+)\s*\(").unwrap();

    let mut propagated_code = String::new();

    for line in content.lines() {
        // We are starting a code block.
        if line.starts_with("```c") && !line.contains("ignore") {
            in_code_block = true;

            if line.contains("propagate") {
                in_propagated_block = true;
            }

            continue;
        }

        // We are ending a code block
        if line.trim() == "```" && in_code_block {
            // Combine propagated code with the current code block.
            let mut code = String::new();
            if !in_propagated_block {
                code.push_str(&propagated_code);
            }
            code.push_str(&current_block);

            code_blocks.push((code, function_names.clone()));

            in_code_block = false;
            in_propagated_block = false;
            current_block.clear();
            function_names.clear();

            continue;
        }

        if in_code_block {
            if line.contains("[[clang::fhe_program]]") {
                if let Some(captures) = function_regex.captures(line) {
                    if let Some(name) = captures.get(1) {
                        function_names.push(name.as_str().to_string());
                    }
                }
            }

            current_block.push_str(line);
            current_block.push('\n');

            if in_propagated_block {
                propagated_code.push_str(line);
                propagated_code.push('\n');
            }
        }
    }

    code_blocks
}

fn run_clang(
    clang_binary_path: &PathBuf,
    file_to_compile: &PathBuf,
    md_path: &PathBuf,
) -> Result<()> {
    let output = Command::new(clang_binary_path)
        .arg("-O2")
        .arg("-target")
        .arg("parasol")
        .arg(file_to_compile.to_str().unwrap())
        .arg("-c")
        .arg("-o")
        .arg(format!("{}.o", file_to_compile.to_str().unwrap()))
        .output()
        .context("Failed to execute clang")?;

    if !output.status.success() {
        eprintln!("\nFailed to compile code block in file {:?}:", md_path);
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        eprintln!("Code was:\n");

        let temp_file_contents = std::fs::read_to_string(file_to_compile)
            .unwrap_or_else(|_| "Failed to read temporary file contents.".to_string());
        eprintln!("{}\n", temp_file_contents);

        anyhow::bail!("Compilation failed");
    }

    Ok(())
}
