# Basic Example

This file tests basic Parasol C code compilation.

## Simple addition

Here's a simple FHE program that adds two numbers:

```c,variant=parasol
[[clang::fhe_program]] uint8_t add(
    [[clang::encrypted]] uint8_t a,
    [[clang::encrypted]] uint8_t b
) {
    return a + b;
}
```

## This should be ignored

```c,variant=parasol,ignore
This code should not be compiled because it has the ignore flag.
This is not valid C code.
```
