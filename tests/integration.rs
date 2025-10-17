//! Integration tests for mdbook-check-code
//!
//! These tests verify the full end-to-end workflow by running the
//! preprocessor against test fixtures in isolated environments.
//!
//! ## Test Architecture
//!
//! Each test uses `TestFixture` to create an isolated environment with:
//! - Temporary book directory (copy of fixtures)
//! - Automatic cleanup via RAII (Drop trait)
//!
//! Tests use `CheckCodePreprocessor::new_for_testing()` to bypass approval checks,
//! allowing fully parallel execution without environment variable manipulation.
//!
//! ## Adding New Tests
//!
//! 1. Create a new fixture in tests/fixtures/ if needed
//! 2. Use `TestFixture::new("path/to/fixture")`
//! 3. Use `#[tokio::test]` for async tests
//! 4. Assert on the returned Result or Book

mod common;

use anyhow::Result;
use common::{PreprocessorTest, TestFixture};
use mdbook::preprocess::CmdPreprocessor;
use mdbook::MDBook;
use mdbook_check_code::CheckCodePreprocessor;

// ===== Tests =====

#[tokio::test]
async fn integration_valid_code_blocks_compile_successfully() -> Result<()> {
    let fixture = TestFixture::new("tests/fixtures/valid_cases")?;
    let test = PreprocessorTest::from_fixture(&fixture)?;

    let result = test.run().await;

    assert!(
        result.is_ok(),
        "Valid code should compile: {:?}",
        result.err()
    );
    Ok(())
}

#[tokio::test]
async fn integration_multiple_languages_supported() -> Result<()> {
    let fixture = TestFixture::new("tests/fixtures/valid_cases")?;
    let test = PreprocessorTest::from_fixture(&fixture)?;

    // Fixture contains C, TypeScript, and Solidity
    let result = test.run().await;

    assert!(
        result.is_ok(),
        "Multi-language support failed: {:?}",
        result.err()
    );
    Ok(())
}

#[tokio::test]
async fn integration_compilation_errors_detected() -> Result<()> {
    let fixture = TestFixture::new("tests/fixtures/error_cases")?;
    let test = PreprocessorTest::from_fixture(&fixture)?;

    let result = test.run().await;

    assert!(result.is_err(), "Invalid code should fail compilation");

    if let Err(e) = result {
        let error_msg = format!("{:#}", e);
        assert!(
            error_msg.contains("compilation"),
            "Unexpected error: {}",
            error_msg
        );
    }

    Ok(())
}

#[tokio::test]
async fn integration_book_structure_unchanged() -> Result<()> {
    let fixture = TestFixture::new("tests/fixtures/valid_cases")?;
    let md = MDBook::load(fixture.book_path())?;
    let original_sections = md.book.sections.len();

    let test = PreprocessorTest::from_fixture(&fixture)?;
    let result_book = test.run().await?;

    assert_eq!(
        result_book.sections.len(),
        original_sections,
        "Preprocessor should not modify book structure"
    );

    Ok(())
}

#[tokio::test]
async fn integration_nested_chapters_processed() -> Result<()> {
    let fixture = TestFixture::new("tests/fixtures/valid_cases")?;
    let md = MDBook::load(fixture.book_path())?;

    // Verify nested structure exists:
    // - parasol_examples/
    // - other_langs/c_examples/
    // - other_langs/ts_examples/

    let has_nested = md.book.iter().any(|item| {
        if let mdbook::book::BookItem::Chapter(ch) = item {
            ch.path
                .as_ref()
                .is_some_and(|p| p.to_str().is_some_and(|s| s.contains('/')))
        } else {
            false
        }
    });

    assert!(has_nested, "Fixture should contain nested chapters");

    let test = PreprocessorTest::from_fixture(&fixture)?;
    let result = test.run().await;

    assert!(result.is_ok(), "Nested chapters failed: {:?}", result.err());
    Ok(())
}

#[tokio::test]
async fn integration_unapproved_book_rejected() -> Result<()> {
    let fixture = TestFixture::new("tests/fixtures/valid_cases")?;
    let md = MDBook::load(fixture.book_path())?;

    let input_json = serde_json::json!([
        {
            "root": md.root,
            "config": md.config,
            "renderer": "html",
            "mdbook_version": env!("CARGO_PKG_VERSION"),
        },
        md.book
    ]);

    let input_str = serde_json::to_string(&input_json)?;
    let (ctx, book) = CmdPreprocessor::parse_input(input_str.as_bytes())?;

    // Use regular preprocessor (NOT new_for_testing) to check approval
    let preprocessor = CheckCodePreprocessor::new();
    let result = preprocessor.run_async(&ctx, book).await;

    assert!(result.is_err(), "Unapproved book should be rejected");

    if let Err(e) = result {
        let error_msg = format!("{:#}", e);
        assert!(
            error_msg.contains("not approved"),
            "Wrong error: {}",
            error_msg
        );
    }

    Ok(())
}
