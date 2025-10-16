# mdbook-check-code

A configuration-driven mdBook preprocessor that validates code blocks by compiling them with user-specified compilers.

## Quick Start

### Using Nix

```bash
# Enter development environment with all compilers
nix develop

# Build the preprocessor
nix build
```

### Using Cargo

```bash
cargo install --path .
```

Requires separate installation of compilers for enabled languages (gcc, clang, tsc, etc.).

## Usage

Add the preprocessor and language configurations to `book.toml`:

```toml
[preprocessor.check-code]

# C language configuration
[preprocessor.check-code.languages.c]
enabled = true
compiler = "gcc"
flags = ["-fsyntax-only"]

# Parasol variant for FHE code
[preprocessor.check-code.languages.c.variants.parasol]
compiler = "${CLANG}"
flags = ["-target", "parasol", "-O2"]
preamble = "#include <parasol.h>"
```

Write code blocks with fence markers:

````markdown
```c,variant=parasol
[[clang::fhe_program]] uint8_t add(uint8_t a, uint8_t b) {
    return a + b;
}
```
````

Build the book:

```bash
mdbook build
```

The preprocessor validates all code blocks during the build process and reports compilation errors.

### Code Block Flags

- `ignore` - Skip compilation for a block
- `propagate` - Make code available to subsequent blocks in the same file

## Configuration

All language behavior is configured in `book.toml`. Each language requires:
- `enabled` (bool) - Whether to check this language
- `compiler` (string) - Compiler executable (supports `${VAR}` env var expansion)
- `flags` (array) - Compiler flags

Optional:
- `preamble` (string) - Code prepended to all blocks
- `fence_markers` (array) - Custom fence identifiers

## Testing

Test with included fixtures:

```bash
# With all compilers (Nix)
nix develop --command bash -c "cd tests/fixtures && mdbook build"

# Or with Cargo
cargo build --release
cd tests/fixtures && mdbook build
```

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPLv3).

## Related Projects

- [mdBook](https://github.com/rust-lang/mdBook) - The book generator
- [Sunscreen LLVM](https://github.com/Sunscreen-tech/sunscreen-llvm) - The Parasol compiler
- [Sunscreen](https://github.com/Sunscreen-tech/Sunscreen) - FHE library and runtime
