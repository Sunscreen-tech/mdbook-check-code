# mdbook-check-code

A configuration-driven mdBook preprocessor that validates code blocks in multiple languages by compiling them with user-specified compilers.

## Overview

`mdbook-check-code` is a preprocessor for [mdBook](https://github.com/rust-lang/mdBook) that automatically extracts and validates code blocks from your documentation. All language behavior is configured in `book.toml` - no languages are built-in. This helps catch errors early and ensures your documentation stays in sync with working code.

## Features

- **Multi-language support**: Parasol C, plain C, TypeScript, or any language you configure
- **Configuration-driven**: All compiler behavior specified in `book.toml` - no hardcoded defaults
- **Environment variable expansion**: Use `${VAR}` syntax in compiler paths
- **No regex**: Uses `pulldown-cmark` for clean markdown parsing
- **Selective compilation**: Supports `ignore` flag to skip specific code blocks
- **Code propagation**: Use the `propagate` flag to share code between blocks (useful for structs and helper functions)
- **mdBook integration**: Works seamlessly with mdBook's build process
- **Clear error messages**: Shows compilation errors with context
- **Extensible**: Add new languages without writing Rust code

## Installation

### Using Nix (Recommended)

This project provides a Nix flake that includes the preprocessor and compilers (Sunscreen LLVM, gcc, TypeScript):

```bash
# Enter development environment
cd mdbook-check-code
nix develop

# Build the preprocessor
nix build

# Install to your profile
nix profile install
```

### Using Cargo

```bash
cargo install --path .
```

**Note**: You'll need to separately install compilers for the languages you want to check:
- Parasol C: [Sunscreen LLVM compiler](https://github.com/Sunscreen-tech/sunscreen-llvm)
- Plain C: gcc or clang
- TypeScript: Node.js and `npm install -g typescript`

## Usage

### Configure your mdBook

Add the preprocessor and language configurations to your `book.toml`:

```toml
[book]
title = "My Documentation"
authors = ["Your Name"]

[preprocessor.check-code]
command = "mdbook-check-code"

# Configure Parasol C (FHE code)
[preprocessor.check-code.languages.parasol-c]
enabled = true
compiler = "${CLANG}"                           # Environment variable expansion
flags = ["-target", "parasol", "-fsyntax-only"]
preamble = "#include <parasol.h>"               # Automatically included
fence_markers = ["parasol-c", "parasol"]

# Configure plain C
[preprocessor.check-code.languages.c]
enabled = true
compiler = "gcc"
flags = ["-fsyntax-only"]
fence_markers = ["c"]

# Configure TypeScript
[preprocessor.check-code.languages.typescript]
enabled = true
compiler = "tsc"
flags = ["--noEmit", "--skipLibCheck"]
fence_markers = ["typescript", "ts"]

[output.html]
```

### Write your documentation

Use standard markdown with code blocks. The fence marker determines which language configuration is used:

````markdown
# Parasol C Example

```parasol-c
[[clang::fhe_program]] uint8_t add(
    [[clang::encrypted]] uint8_t a,
    [[clang::encrypted]] uint8_t b
) {
    return a + b;
}
```

# Plain C Example

```c
#include <stdint.h>

uint32_t multiply(uint32_t a, uint32_t b) {
    return a * b;
}
```

# TypeScript Example

```typescript
function greet(name: string): string {
    return `Hello, ${name}!`;
}
```
````

### Build your book

```bash
# Set environment variables (if not using Nix)
export CLANG=/path/to/sunscreen-llvm/bin/clang

# Build the book
mdbook build
```

The preprocessor will automatically validate all configured code blocks and report any errors.

## Code Block Flags

### `ignore` - Skip compilation

Use this flag for code that shouldn't be compiled (e.g., pseudocode, incomplete examples):

````markdown
```parasol-c,ignore
// This won't be compiled
incomplete_function() {
```
````

### `propagate` - Share code between blocks

Use this flag to make definitions available to subsequent code blocks in the same file:

````markdown
Define a struct:

```parasol-c,propagate
typedef struct Point {
    uint16_t x;
    uint16_t y;
} Point;
```

Use the struct in later blocks:

```parasol-c
[[clang::fhe_program]] void move_point(
    [[clang::encrypted]] Point *p,
    uint16_t dx,
    uint16_t dy
) {
    p->x = p->x + dx;
    p->y = p->y + dy;
}
```
````

## Configuration

### Language Configuration

Each language requires a full configuration in `book.toml`:

**Required fields**:
- `enabled` (bool): Whether to check this language
- `compiler` (string): Compiler executable (supports `${VAR}` env var expansion)
- `flags` (array): Compiler flags
- `fence_markers` (array): Which markdown fence markers identify this language

**Optional fields**:
- `preamble` (string): Code prepended to all blocks (e.g., includes)

### Adding New Languages

Add any language by configuring it in `book.toml`. Example for Python with mypy:

```toml
[preprocessor.check-code.languages.python]
enabled = true
compiler = "mypy"
flags = ["--ignore-missing-imports"]
fence_markers = ["python", "py"]
```

### Environment Variables

Use `${VAR}` syntax in compiler paths and flags. Common variables:
- `${CLANG}`: Path to Sunscreen LLVM clang binary

### Nix Development Environment

When using `nix develop`, the environment is automatically configured with:
- The Sunscreen LLVM compiler
- Standard gcc compiler
- TypeScript (node + tsc)
- `CLANG` environment variable
- mdBook and other development tools

## How It Works

1. **Integration**: mdBook calls the preprocessor before rendering
2. **Configuration**: Preprocessor loads language configs from `book.toml` and expands env vars
3. **Extraction**: Uses `pulldown-cmark` to extract code blocks (no regex!)
4. **Language matching**: Fence markers matched via simple string equality checks
5. **Compilation**: Each code block is validated with the configured compiler and flags
6. **Error Reporting**: Compilation failures are reported with file names and error messages
7. **Success**: If all code blocks compile, mdBook continues with rendering

## Testing

Test the preprocessor on the included fixtures (includes Parasol C, plain C, and TypeScript examples):

```bash
# Using Nix (recommended - includes all compilers)
nix develop --command bash -c "cd tests/fixtures && mdbook build"

# Or with Cargo (requires compilers to be installed separately)
cargo build --release
cd tests/fixtures
export CLANG=/path/to/sunscreen-llvm/bin/clang
mdbook build
```

## Development

```bash
# Enter development environment (includes all compilers)
nix develop

# Build
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Run clippy
cargo clippy

# Build with Nix
nix build
```

## Architecture

- **No regex**: Uses `pulldown-cmark` for markdown parsing and simple string operations
- **Configuration-driven**: All language behavior from `book.toml`, no hardcoded languages
- **Modular**: Clean separation between markdown extraction, configuration, and compilation
- **Extensible**: Add languages via config without touching Rust code

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPLv3).

See the [LICENSE](LICENSE) file for details.

## Related Projects

- [mdBook](https://github.com/rust-lang/mdBook) - The book generator
- [Sunscreen LLVM](https://github.com/Sunscreen-tech/sunscreen-llvm) - The Parasol compiler
- [Sunscreen](https://github.com/Sunscreen-tech/Sunscreen) - FHE library and runtime
