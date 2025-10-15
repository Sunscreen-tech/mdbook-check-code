# Parasol C Examples

This section demonstrates Parasol C code validation at depth level 1.

The following examples show how to write FHE programs using Parasol C.

Here is a simple function to test code validation at this depth:

```parasol-c
[[clang::fhe_program]] uint8_t simple_add(uint8_t x, uint8_t y) {
    return x + y;
}
```
