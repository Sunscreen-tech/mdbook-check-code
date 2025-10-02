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
    // Handle mdbook preprocessor commands
    let mut args = std::env::args().skip(1);

    if let Some(arg) = args.next() {
        if arg == "supports" {
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
    }

    // Run as preprocessor
    if let Err(e) = handle_preprocessing() {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let preprocessor = CheckCodePreprocessor::new();
    let processed_book = preprocessor.run(&ctx, book)?;

    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
