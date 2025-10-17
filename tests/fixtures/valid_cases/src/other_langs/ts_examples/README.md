# TypeScript Examples

This section demonstrates TypeScript code validation at depth level 2.

TypeScript type checking works seamlessly with the preprocessor:

```typescript
interface Config {
  host: string;
  port: number;
  secure?: boolean;
}

function getUrl(config: Config): string {
  const protocol = config.secure ? "https" : "http";
  return `${protocol}://${config.host}:${config.port}`;
}
```
