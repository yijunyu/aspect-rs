# The Aspect Trait

The `Aspect` trait is the foundation of all aspects in aspect-rs. Implementing this trait allows your type to be woven into functions using the `#[aspect(...)]` macro.

## Trait Definition

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

## Requirements

### Thread Safety (`Send + Sync`)

All aspects must be thread-safe because they may be used across multiple threads:

```rust
// ✅ Good - implements Send + Sync automatically
#[derive(Default)]
struct LoggingAspect;

// ❌ Bad - Rc is not Send + Sync
struct BadAspect {
    data: Rc<String>,  // Compile error!
}
```

Use `Arc` instead of `Rc` for shared data:

```rust
struct ThreadSafeAspect {
    data: Arc<Mutex<HashMap<String, String>>>,
}
```

## The Four Advice Methods

### 1. `before` - Runs Before Function

```rust
fn before(&self, ctx: &JoinPoint) {
    println!("About to call {}", ctx.function_name);
}
```

**Use cases:**
- Logging function entry
- Input validation
- Authorization checks
- Metrics start

### 2. `after` - Runs After Success

```rust
fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
    if let Some(num) = result.downcast_ref::<i32>() {
        println!("{} returned {}", ctx.function_name, num);
    }
}
```

**Use cases:**
- Logging function exit
- Result caching
- Metrics collection
- Cleanup

### 3. `after_throwing` - Runs On Error

```rust
fn after_throwing(&self, ctx: &JoinPoint, error: &dyn Any) {
    eprintln!("Error in {}: {:?}", ctx.function_name, error);
}
```

**Use cases:**
- Error logging
- Alerting
- Circuit breaker logic
- Rollback transactions

### 4. `around` - Wraps Entire Execution

```rust
fn around(&self, mut pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    println!("Before");
    let result = pjp.proceed()?;
    println!("After");
    Ok(result)
}
```

**Use cases:**
- Timing measurement
- Caching (skip execution if cached)
- Transaction management
- Retry logic

## Complete Example

```rust
use aspect_core::prelude::*;
use std::time::Instant;

struct ComprehensiveAspect;

impl Aspect for ComprehensiveAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ {} at {}:{}", ctx.function_name, ctx.file, ctx.line);
    }

    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
        println!("✓ {} succeeded", ctx.function_name);
    }

    fn after_throwing(&self, ctx: &JoinPoint, error: &dyn Any) {
        eprintln!("✗ {} failed: {:?}", ctx.function_name, error);
    }

    fn around(&self, mut pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let start = Instant::now();
        let result = pjp.proceed()?;
        println!("Took {:?}", start.elapsed());
        Ok(result)
    }
}
```

## Default Implementations

All methods have default (no-op) implementations, so you only implement what you need:

```rust
struct MinimalAspect;

impl Aspect for MinimalAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("Called {}", ctx.function_name);
    }
    // after, after_throwing, around use defaults
}
```

## Next: [JoinPoint Context](joinpoint.md)
