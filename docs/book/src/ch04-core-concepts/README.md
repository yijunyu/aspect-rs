# Core Concepts

Deep dive into the fundamental building blocks of aspect-rs.

## What You'll Learn

- The `Aspect` trait and its four advice methods
- `JoinPoint` context and metadata
- `ProceedingJoinPoint` for around advice
- Advice types and when to use each
- Error handling with `AspectError`

## The Aspect Trait

Every aspect implements this trait:

```rust
pub trait Aspect: Send + Sync {
    fn before(&self, ctx: &JoinPoint) {}
    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {}
    fn after_throwing(&self, ctx: &JoinPoint, error: &dyn Any) {}
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        pjp.proceed()
    }
}
```

**Key points:**
- All methods have default implementations
- Must be `Send + Sync` for thread safety
- Implement only the advice you need

## Chapter Sections

- **[The Aspect Trait](aspect-trait.md)** - Detailed API reference
- **[JoinPoint Context](joinpoint.md)** - Accessing execution metadata
- **[Advice Types](advice-types.md)** - Comparison and use cases
- **[Error Handling](error-handling.md)** - Working with errors and panics

See [The Aspect Trait](aspect-trait.md) to continue.
