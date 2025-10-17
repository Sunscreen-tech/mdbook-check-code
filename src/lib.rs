//! mdbook-check-code library
//!
//! This library provides the preprocessor implementation for validating code blocks
//! in mdBook projects. The primary interface is the mdbook-check-code binary, but
//! the library can be used programmatically for testing or custom integrations.
//!
//! ## Public API
//!
//! The main public interface is [`CheckCodePreprocessor`], which implements the
//! mdBook `Preprocessor` trait.
//!
//! Additional utilities:
//! - [`get_language_metadata`] - Get metadata for a language (fence markers and file extension)
//! - [`LanguageMetadata`] - Metadata structure for a language

mod approval;
mod compilation;
mod config;
mod extractor;
mod language;
mod preprocessor;
mod reporting;
mod task_collector;

pub use language::{get_language_metadata, LanguageMetadata};
pub use preprocessor::CheckCodePreprocessor;
