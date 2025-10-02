# Basic Example

This file tests basic Parasol code compilation.

## Simple addition

Here's a simple FHE program that adds two numbers:

```c
#include <parasol.h>

[[clang::fhe_program]] uint8_t add(
    [[clang::encrypted]] uint8_t a,
    [[clang::encrypted]] uint8_t b
) {
    return a + b;
}
```

## This should be ignored

```c,ignore
This code should not be compiled because it has the ignore flag.
This is not valid C code.
```
