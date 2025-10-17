use anyhow::{Context, Result};
use directories::ProjectDirs;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Compute SHA256 hash of path + "\n" + content (direnv style)
pub fn compute_hash(path: &Path, content: &str) -> String {
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let input = format!("{}\n{}", canonical_path.display(), content);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get the approval directory path
fn get_approval_dir() -> Result<PathBuf> {
    // Check for XDG_DATA_HOME environment variable first (respects XDG standard on all platforms)
    if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg_data_home)
            .join("mdbook-check-code")
            .join("allow"));
    }

    // Fall back to platform-specific defaults via directories crate
    let proj_dirs = ProjectDirs::from("", "", "mdbook-check-code")
        .context("Failed to determine project directories")?;
    Ok(proj_dirs.data_dir().join("allow"))
}

/// Check if a book.toml is approved
pub fn is_approved(book_toml_path: &Path) -> Result<bool> {
    let content = fs::read_to_string(book_toml_path)
        .with_context(|| format!("Failed to read {}", book_toml_path.display()))?;
    let hash = compute_hash(book_toml_path, &content);
    let approval_dir = get_approval_dir()?;
    let approval_file = approval_dir.join(&hash);
    Ok(approval_file.exists())
}

/// Approve a book.toml
#[allow(dead_code)] // Used by CLI binary (main.rs), not library
pub fn approve(book_toml_path: &Path) -> Result<()> {
    let content = fs::read_to_string(book_toml_path)
        .with_context(|| format!("Failed to read {}", book_toml_path.display()))?;
    let hash = compute_hash(book_toml_path, &content);
    let approval_dir = get_approval_dir()?;

    // Create approval directory if it doesn't exist
    fs::create_dir_all(&approval_dir).with_context(|| {
        format!(
            "Failed to create approval directory: {}",
            approval_dir.display()
        )
    })?;

    // Write approval file with the path
    let approval_file = approval_dir.join(&hash);
    let canonical_path = book_toml_path
        .canonicalize()
        .unwrap_or_else(|_| book_toml_path.to_path_buf());
    fs::write(&approval_file, canonical_path.display().to_string())
        .with_context(|| format!("Failed to write approval file: {}", approval_file.display()))?;

    Ok(())
}

/// Deny (remove approval) for a book.toml
#[allow(dead_code)] // Used by CLI binary (main.rs), not library
pub fn deny(book_toml_path: &Path) -> Result<()> {
    let content = fs::read_to_string(book_toml_path)
        .with_context(|| format!("Failed to read {}", book_toml_path.display()))?;
    let hash = compute_hash(book_toml_path, &content);
    let approval_dir = get_approval_dir()?;
    let approval_file = approval_dir.join(&hash);

    if approval_file.exists() {
        fs::remove_file(&approval_file).with_context(|| {
            format!(
                "Failed to remove approval file: {}",
                approval_file.display()
            )
        })?;
    }

    Ok(())
}

/// List all approved books
#[allow(dead_code)] // Used by CLI binary (main.rs), not library
pub fn list_approved() -> Result<Vec<String>> {
    let approval_dir = get_approval_dir()?;

    if !approval_dir.exists() {
        return Ok(vec![]);
    }

    let mut approved = Vec::new();
    for entry in fs::read_dir(&approval_dir).with_context(|| {
        format!(
            "Failed to read approval directory: {}",
            approval_dir.display()
        )
    })? {
        let entry = entry?;
        if entry.path().is_file() {
            if let Ok(path_content) = fs::read_to_string(entry.path()) {
                approved.push(path_content);
            }
        }
    }

    Ok(approved)
}
