# JoinPoint Context

The `JoinPoint` struct provides metadata about the function being executed.

## Definition

```rust
pub struct JoinPoint {
    pub function_name: &'static str,
    pub module_path: &'static str,
    pub file: &'static str,
    pub line: u32,
}
```

## Example

```rust
impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("[{}] {}::{} at {}:{}",
            chrono::Utc::now(),
            ctx.module_path,
            ctx.function_name,
            ctx.file,
            ctx.line
        );
    }
}
```

See [The Aspect Trait](aspect-trait.md) for more context.
