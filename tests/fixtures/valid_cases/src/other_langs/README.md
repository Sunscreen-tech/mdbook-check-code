# Other Languages

This section demonstrates code validation for non-Parasol languages at depth level 1.

Code validation works with multiple language configurations. Here is a simple C example:

```c
struct Point {
    int x;
    int y;
};

int add_points(struct Point p1, struct Point p2) {
    return p1.x + p2.x + p1.y + p2.y;
}
```
