mod config;
mod extractor;
mod language;
mod preprocessor;

use anyhow::Result;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use preprocessor::CheckCodePreprocessor;
use std::io;
use std::process::exit;

pub fn main() {
    // Initialize logging
    env_logger::init();

    // Handle mdbook preprocessor commands
    let mut args = std::env::args().skip(1);

    if let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("mdbook-check-code {}", env!("CARGO_PKG_VERSION"));
                exit(0);
            }
            "--help" | "-h" => {
                print_help();
                exit(0);
            }
            "supports" => {
                // Check if we support the renderer
                if let Some(renderer) = args.next() {
                    let preprocessor = CheckCodePreprocessor::new();
                    if preprocessor.supports_renderer(&renderer) {
                        exit(0);
                    } else {
                        exit(1);
                    }
                }
            }
            _ => {}
        }
    }

    // Run as preprocessor
    if let Err(_e) = handle_preprocessing() {
        // Error already printed in preprocessor with proper formatting
        exit(1);
    }
}

fn print_help() {
    println!("mdbook-check-code {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("A configuration-driven mdBook preprocessor that validates code blocks");
    println!("by compiling them with user-specified compilers.");
    println!();
    println!("USAGE:");
    println!("    mdbook-check-code [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
    println!();
    println!("This preprocessor is typically invoked by mdBook during the build process.");
    println!("Configure languages in your book.toml under [preprocessor.check-code.languages]");
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let preprocessor = CheckCodePreprocessor::new();
    let processed_book = preprocessor.run(&ctx, book)?;

    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
