# TypeScript Example

This file tests TypeScript type checking.

## Basic function

Here's a simple TypeScript function:

```typescript
function add(a: number, b: number): number {
  return a + b;
}
```

## Using interfaces

```typescript
interface User {
  name: string;
  age: number;
}

function greet(user: User): string {
  return `Hello, ${user.name}! You are ${user.age} years old.`;
}
```

## Generic function

```ts
function identity<T>(arg: T): T {
  return arg;
}
```

## Ignored code

```typescript,ignore
This should not be type-checked!
const x: invalid syntax here
```
