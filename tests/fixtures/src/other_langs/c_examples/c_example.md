# Plain C Example

This file tests standard C code compilation.

## Simple function

Here's a basic C function:

```c
#include <stdio.h>
#include <stdint.h>

uint32_t multiply(uint32_t a, uint32_t b) {
    return a * b;
}
```

## Using structs

```c
#include <stdint.h>

typedef struct {
    int x;
    int y;
} Point2D;

Point2D create_point(int x, int y) {
    Point2D p;
    p.x = x;
    p.y = y;
    return p;
}
```

## Ignored code

```c,ignore
This should not compile!
invalid C syntax here...
```
