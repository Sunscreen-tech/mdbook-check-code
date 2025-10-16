mod approval;
mod compilation;
mod config;
mod extractor;
mod language;
mod preprocessor;
mod reporting;
mod task_collector;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use preprocessor::CheckCodePreprocessor;
use std::io;
use std::path::PathBuf;
use std::process::exit;

const LONG_ABOUT: &str = r##"A configuration-driven mdBook preprocessor that validates code blocks by compiling
them with user-specified compilers.

## Integration with mdBook

This preprocessor is typically invoked automatically by mdBook during the build
process. Configure it in your book.toml file and mdBook will handle execution.

## Configuration Example (book.toml)

```toml
[preprocessor.check-code]

# C configuration
[preprocessor.check-code.languages.c]
enabled = true
compiler = "gcc"
flags = ["-fsyntax-only"]

# Parasol variant - uses Sunscreen LLVM for FHE compilation
[preprocessor.check-code.languages.c.variants.parasol]
compiler = "${CLANG}"                    # Supports ${VAR} expansion
flags = ["-target", "parasol", "-O2"]
preamble = "#include <parasol.h>"        # Prepended to all blocks

# TypeScript configuration
[preprocessor.check-code.languages.typescript]
enabled = true
compiler = "tsc"
flags = ["--noEmit", "--skipLibCheck"]

# Solidity configuration
[preprocessor.check-code.languages.solidity]
enabled = true
compiler = "solc"
```

For custom languages, you can optionally specify `fence_markers` to map multiple
markdown fence identifiers to the same language (e.g., ["ts", "typescript"]).

Language variants are referenced using the `variant=name` attribute:
  Example: ```c,variant=parasol

## Code Block Flags

- `ignore` - Skip compilation for this block
  Example: ```c,ignore

- `propagate` - Share code with subsequent blocks in the same file
  Example: ```c,propagate

- `variant=name` - Use a language variant
  Example: ```c,variant=parasol

## Environment Variables

- `CLANG` - Path to Sunscreen LLVM clang (required for Parasol C variant)
- `RUST_LOG` - Set to "info" to see detailed compilation logs
  Example: `RUST_LOG=info mdbook build`

For more information, visit: https://github.com/Sunscreen-tech/mdbook-check-code
"##;

#[derive(Parser)]
#[command(
    name = "mdbook-check-code",
    author,
    version,
    about = "A configuration-driven mdBook preprocessor that validates code blocks",
    long_about = LONG_ABOUT
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check whether a renderer is supported by this preprocessor
    Supports {
        /// The renderer name to check (e.g., "html", "markdown")
        renderer: String,
    },
    /// Approve the current book.toml for code execution
    Allow,
    /// Remove approval for the current book.toml
    Deny,
    /// Show approval status for the current book
    Status,
    /// List all approved books
    List,
}

pub fn main() {
    // Initialize logging with mdBook-compatible format
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use chrono::Local;
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] (mdbook_check_code): {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Supports { renderer }) => {
            let preprocessor = CheckCodePreprocessor::new();
            if preprocessor.supports_renderer(&renderer) {
                exit(0);
            } else {
                exit(1);
            }
        }
        Some(Commands::Allow) => {
            let book_toml = match find_book_toml() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            };
            if let Err(e) = approval::approve(&book_toml) {
                eprintln!("Error: {}", e);
                exit(1);
            }
            println!("Approved: {}", book_toml.display());
            exit(0);
        }
        Some(Commands::Deny) => {
            let book_toml = match find_book_toml() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            };
            if let Err(e) = approval::deny(&book_toml) {
                eprintln!("Error: {}", e);
                exit(1);
            }
            println!("Removed approval: {}", book_toml.display());
            exit(0);
        }
        Some(Commands::Status) => {
            let book_toml = match find_book_toml() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            };
            match approval::is_approved(&book_toml) {
                Ok(true) => {
                    println!("Approved: {}", book_toml.display());
                    exit(0);
                }
                Ok(false) => {
                    println!("Not approved: {}", book_toml.display());
                    exit(1);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            }
        }
        Some(Commands::List) => match approval::list_approved() {
            Ok(approved) => {
                if approved.is_empty() {
                    println!("No approved books");
                } else {
                    for path in approved {
                        println!("{}", path);
                    }
                }
                exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                exit(1);
            }
        },
        None => {
            // Run as preprocessor (default when called by mdbook)
            if let Err(_e) = handle_preprocessing() {
                // Error already printed in preprocessor with proper formatting
                exit(1);
            }
        }
    }
}

fn find_book_toml() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let book_toml = current_dir.join("book.toml");

    if !book_toml.exists() {
        anyhow::bail!("No book.toml found in current directory");
    }

    Ok(book_toml)
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let preprocessor = CheckCodePreprocessor::new();
    let processed_book = preprocessor.run(&ctx, book)?;

    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
