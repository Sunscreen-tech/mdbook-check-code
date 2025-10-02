# Propagate Example

This file tests the propagate feature.

## Define a struct

```parasol-c,propagate
typedef struct Point {
    uint16_t x;
    uint16_t y;
} Point;
```

## Use the struct

This code block should have access to the Point struct from the previous block:

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
