# mdbook-check-parasol

An mdBook preprocessor that validates Parasol C code blocks by compiling them with the Sunscreen LLVM compiler.

## Overview

`mdbook-check-parasol` is a preprocessor for [mdBook](https://github.com/rust-lang/mdBook) that automatically extracts and compiles C code blocks from your documentation to ensure they are valid Parasol FHE programs. This helps catch errors early and ensures your documentation stays in sync with working code.

## Features

- **Automatic validation**: Compiles all C code blocks during book build
- **Selective compilation**: Supports `ignore` flag to skip specific code blocks
- **Code propagation**: Use the `propagate` flag to share code between blocks (useful for structs and helper functions)
- **mdBook integration**: Works seamlessly with mdBook's build process
- **Clear error messages**: Shows compilation errors with context

## Installation

### Using Nix (Recommended)

This project provides a Nix flake that includes both the preprocessor and the Sunscreen LLVM compiler:

```bash
# Enter development environment
cd mdbook-check-parasol
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

**Note**: You'll need to separately install the [Sunscreen LLVM compiler](https://github.com/Sunscreen-tech/sunscreen-llvm) and set the `CLANG` environment variable to point to it.

## Usage

### Configure your mdBook

Add the preprocessor to your `book.toml`:

```toml
[book]
title = "My FHE Documentation"
authors = ["Your Name"]

[preprocessor.check-parasol]
command = "mdbook-check-parasol"

[output.html]
```

### Write your documentation

Use standard markdown with C code blocks:

````markdown
# My FHE Program

Here's a simple addition program:

```c
#include <parasol.h>

[[clang::fhe_program]] uint8_t add(
    [[clang::encrypted]] uint8_t a,
    [[clang::encrypted]] uint8_t b
) {
    return a + b;
}
```
````

### Build your book

```bash
# Set the compiler path (if not using Nix)
export CLANG=/path/to/sunscreen-llvm/bin/clang

# Build the book
mdbook build
```

The preprocessor will automatically compile all C code blocks and report any errors.

## Code Block Flags

### `ignore` - Skip compilation

Use this flag for code that shouldn't be compiled (e.g., pseudocode, incomplete examples):

````markdown
```c,ignore
// This won't be compiled
incomplete_function() {
```
````

### `propagate` - Share code between blocks

Use this flag to make definitions available to subsequent code blocks in the same file:

````markdown
Define a struct:

```c,propagate
#include <parasol.h>

typedef struct Point {
    uint16_t x;
    uint16_t y;
} Point;
```

Use the struct in later blocks:

```c
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

### Environment Variables

- `CLANG`: Path to the Sunscreen LLVM clang binary (defaults to `clang`)

### Nix Development Environment

When using `nix develop`, the environment is automatically configured with:
- The Sunscreen LLVM compiler
- `CLANG` environment variable pointing to the compiler
- mdBook and other development tools

## How It Works

1. **Integration**: mdBook calls the preprocessor before rendering
2. **Extraction**: The preprocessor extracts all C code blocks from each chapter
3. **Compilation**: Each code block is compiled with `clang -target parasol -O2 -c`
4. **Error Reporting**: Compilation failures are reported with file names and error messages
5. **Success**: If all code blocks compile, mdBook continues with rendering

## Testing

Test the preprocessor on the included fixtures:

```bash
# Using Nix
nix develop
cd tests/fixtures
mdbook build

# Or with Cargo
cargo build --release
cd tests/fixtures
CLANG=/path/to/clang ../../target/release/mdbook-check-parasol
```

## Development

```bash
# Enter development environment
nix develop

# Build
cargo build

# Format code
cargo fmt

# Run clippy
cargo clippy

# Build with Nix
nix build
```

## License

This project is licensed under the MIT License.

## Related Projects

- [mdBook](https://github.com/rust-lang/mdBook) - The book generator
- [Sunscreen LLVM](https://github.com/Sunscreen-tech/sunscreen-llvm) - The Parasol compiler
- [Sunscreen](https://github.com/Sunscreen-tech/Sunscreen) - FHE library and runtime
