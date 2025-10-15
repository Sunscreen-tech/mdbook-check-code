# C Examples

This section demonstrates plain C code validation at depth level 2.

Standard C code can be validated without FHE annotations:

```c
typedef struct {
    int value;
    char label[32];
} Item;

int compare_items(Item a, Item b) {
    return a.value - b.value;
}
```
