//! Common test utilities for integration tests
//!
//! This module contains shared test fixtures and helper functions used across
//! integration tests. These utilities are not compiled into the library.

use anyhow::Result;
use mdbook::book::Book;
use mdbook::preprocess::CmdPreprocessor;
use mdbook::MDBook;
use mdbook_check_code::CheckCodePreprocessor;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Isolated test fixture with automatic cleanup
///
/// Creates a temporary copy of a test fixture book, allowing tests to run
/// in parallel without interfering with each other.
pub struct TestFixture {
    _book_dir: TempDir,
    book_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture from the default valid_cases directory
    pub fn new() -> Result<Self> {
        Self::new_from("tests/fixtures/valid_cases")
    }

    /// Create a new test fixture from a specific source directory
    pub fn new_from(source: impl AsRef<Path>) -> Result<Self> {
        let book_dir = TempDir::new()?;

        // Copy fixture to temp location
        copy_dir_all(source.as_ref(), book_dir.path())?;

        Ok(Self {
            book_path: book_dir.path().to_path_buf(),
            _book_dir: book_dir,
        })
    }

    /// Get the path to the book directory
    pub fn book_path(&self) -> &Path {
        &self.book_path
    }
}

/// Helper to run preprocessor on a test book
///
/// Wraps an MDBook instance and provides a convenient async run method
/// that simulates how mdBook would invoke the preprocessor.
pub struct PreprocessorTest {
    book: MDBook,
}

impl PreprocessorTest {
    /// Create a preprocessor test from a fixture
    pub fn from_fixture(fixture: &TestFixture) -> Result<Self> {
        let book = MDBook::load(fixture.book_path())?;
        Ok(Self { book })
    }

    /// Run the preprocessor on the test book
    ///
    /// Uses `CheckCodePreprocessor::new_for_testing()` to bypass approval checks,
    /// allowing tests to run without manual approval.
    pub async fn run(&self) -> Result<Book> {
        // Create JSON input like mdbook would send
        let input_json = serde_json::json!([
            {
                "root": self.book.root,
                "config": self.book.config,
                "renderer": "html",
                "mdbook_version": env!("CARGO_PKG_VERSION"),
            },
            self.book.book
        ]);

        let input_str = serde_json::to_string(&input_json)?;
        let (ctx, book) = CmdPreprocessor::parse_input(input_str.as_bytes())?;

        let preprocessor = CheckCodePreprocessor::new_for_testing();
        preprocessor.run_async(&ctx, book).await
    }
}

/// Recursively copy all files and directories from src to dst
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
