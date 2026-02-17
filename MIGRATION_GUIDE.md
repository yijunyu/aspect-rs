# Migration Guide

This guide helps you migrate to aspect-rs from various patterns and approaches.

## Table of Contents

- [From Manual Cross-Cutting Code](#from-manual-cross-cutting-code)
- [From Decorator Pattern](#from-decorator-pattern)
- [From Middleware Pattern](#from-middleware-pattern)
- [From Procedural Macros](#from-procedural-macros)
- [From Other Languages (Java/AspectJ, C#)](#from-other-languages)

---

## From Manual Cross-Cutting Code

### Before: Scattered Logging

```rust
fn fetch_user(id: u64) -> Result<User, Error> {
    println!("→ Entering fetch_user");
    let start = std::time::Instant::now();

    let result = database::query_user(id);

    let elapsed = start.elapsed();
    println!("← Exiting fetch_user ({:?})", elapsed);

    result
}

fn create_user(name: String) -> Result<User, Error> {
    println!("→ Entering create_user");
    let start = std::time::Instant::now();

    let result = database::insert_user(name);

    let elapsed = start.elapsed();
    println!("← Exiting create_user ({:?})", elapsed);

    result
}

// Repeated for every function... 😰
```

### After: aspect-rs

```rust
use aspect_std::{LoggingAspect, TimingAspect};

#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn fetch_user(id: u64) -> Result<User, Error> {
    database::query_user(id)
}

#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn create_user(name: String) -> Result<User, Error> {
    database::insert_user(name)
}
```

**Benefits**:
- ✅ No code duplication
- ✅ Easy to enable/disable
- ✅ Centralized aspect logic
- ✅ Clean business logic

---

## From Decorator Pattern

### Before: Manual Decorators

```rust
struct LoggingDecorator<F> {
    inner: F,
}

impl<F, T> LoggingDecorator<F>
where
    F: Fn(u64) -> Result<T, Error>,
{
    fn call(&self, id: u64) -> Result<T, Error> {
        println!("→ Entering");
        let result = (self.inner)(id);
        println!("← Exiting");
        result
    }
}

// Usage - verbose and type-heavy
let fetch = LoggingDecorator {
    inner: |id| database::query_user(id),
};
let user = fetch.call(42)?;
```

### After: aspect-rs

```rust
#[aspect(LoggingAspect::new())]
fn fetch_user(id: u64) -> Result<User, Error> {
    database::query_user(id)
}

// Direct usage - clean and natural
let user = fetch_user(42)?;
```

**Benefits**:
- ✅ No manual decorator wrapping
- ✅ Natural function call syntax
- ✅ Works with regular functions
- ✅ Type inference preserved

---

## From Middleware Pattern

### Before: Tower/Actix Middleware

```rust
// In web frameworks
use actix_web::middleware::Logger;

App::new()
    .wrap(Logger::default())
    .wrap(metrics_middleware())
    .wrap(auth_middleware())
    .service(my_handler);

// But only works in web context!
```

### After: aspect-rs (Universal)

```rust
// Works everywhere - not just web handlers
use aspect_std::{LoggingAspect, MetricsAspect, AuthorizationAspect};

#[aspect(LoggingAspect::new())]
#[aspect(MetricsAspect::new())]
#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
fn my_handler(req: Request) -> Response {
    process_request(req)
}

#[aspect(LoggingAspect::new())]
#[aspect(MetricsAspect::new())]
fn background_job() -> Result<(), Error> {
    // Aspects work here too!
    process_job()
}
```

**Benefits**:
- ✅ Not limited to web frameworks
- ✅ Works with any function
- ✅ Consistent across codebase
- ✅ Framework-agnostic

---

## From Procedural Macros

### Before: Custom Macros for Each Concern

```rust
// Separate macro for each concern
#[log_entry_exit]
fn fetch_user(id: u64) -> User { /* ... */ }

#[measure_time]
fn process_data(data: &str) -> String { /* ... */ }

#[with_cache]
fn expensive_calc(n: u64) -> u64 { /* ... */ }

// Each macro needs separate implementation
```

### After: aspect-rs (Composable)

```rust
use aspect_std::*;

// Compose multiple concerns with one pattern
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(CachingAspect::new())]
fn comprehensive_function(x: i32) -> i32 {
    expensive_computation(x)
}
```

**Benefits**:
- ✅ Unified approach
- ✅ Easy to compose
- ✅ Reusable aspects
- ✅ Standard patterns

---

## From Other Languages

### From Java/AspectJ

**AspectJ:**
```java
@Aspect
public class LoggingAspect {
    @Before("execution(* com.example.service.*.*(..))")
    public void logBefore(JoinPoint joinPoint) {
        System.out.println("→ " + joinPoint.getSignature().getName());
    }
}
```

**aspect-rs:**
```rust
use aspect_core::prelude::*;

#[derive(Default)]
struct LoggingAspect;

impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ {}", ctx.function_name);
    }
}

// Pointcut support
#[aspect(LoggingAspect::new())]
fn my_service_method() { }

// Automatic matching
// #[advice(pointcut = "execution(pub fn *(..)) && within(crate::service)")]
```

**Differences**:
- ✅ Similar concepts (JoinPoint, Aspect, Advice)
- ✅ Type-safe at compile time
- ✅ Zero runtime overhead

### From C#/PostSharp

**PostSharp:**
```csharp
[Log]
public void MyMethod()
{
    // Business logic
}

[Serializable]
public class LogAttribute : OnMethodBoundaryAspect
{
    public override void OnEntry(MethodExecutionArgs args)
    {
        Console.WriteLine("→ " + args.Method.Name);
    }
}
```

**aspect-rs:**
```rust
#[aspect(LoggingAspect::new())]
fn my_method() {
    // Business logic
}

#[derive(Default)]
struct LoggingAspect;

impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ {}", ctx.function_name);
    }
}
```

**Differences**:
- ✅ Attribute-based syntax similar
- ✅ No runtime reflection needed
- ✅ Compile-time code generation
- ✅ Better performance (no IL weaving overhead)

---

## Step-by-Step Migration

### Step 1: Identify Cross-Cutting Concerns

Look for patterns like:
- Logging scattered across functions
- Repeated timing/performance code
- Manual caching logic
- Authorization checks in every handler
- Transaction management boilerplate

### Step 2: Choose Appropriate Aspects

| Concern | Aspect | Example |
|---------|--------|---------|
| Logging | `LoggingAspect` | Entry/exit logs |
| Performance | `TimingAspect` | Execution time |
| Caching | `CachingAspect` | Memoization |
| Metrics | `MetricsAspect` | Call counts |
| Rate limiting | `RateLimitAspect` | API throttling |
| Reliability | `CircuitBreakerAspect` | Fault tolerance |
| Security | `AuthorizationAspect` | Access control |
| Validation | `ValidationAspect` | Input checks |

### Step 3: Apply Aspects

**Start small:**
```rust
// 1. Add dependencies
use aspect_std::LoggingAspect;

// 2. Apply to one function
#[aspect(LoggingAspect::new())]
fn test_function() {
    println!("Business logic");
}

// 3. Verify it works
test_function(); // Should see entry/exit logs
```

**Then expand:**
```rust
// Apply to more functions
#[aspect(LoggingAspect::new())]
fn function1() { /* ... */ }

#[aspect(LoggingAspect::new())]
fn function2() { /* ... */ }
```

### Step 4: Remove Old Code

```rust
// Before
fn fetch_user(id: u64) -> Result<User, Error> {
    println!("→ fetch_user");  // ← Remove this
    let result = database::query_user(id);
    println!("← fetch_user");  // ← Remove this
    result
}

// After
#[aspect(LoggingAspect::new())]
fn fetch_user(id: u64) -> Result<User, Error> {
    database::query_user(id)  // Clean!
}
```

### Step 5: Compose Multiple Aspects

```rust
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(CachingAspect::new())]
fn comprehensive_function(x: i32) -> i32 {
    expensive_computation(x)
}
```

---

## Common Migration Patterns

### Pattern 1: Extract Logging

**Before:**
```rust
fn process_order(order: Order) -> Result<(), Error> {
    log::info!("Processing order {}", order.id);
    let result = business_logic(order);
    match &result {
        Ok(_) => log::info!("Order {} processed", order.id),
        Err(e) => log::error!("Order {} failed: {}", order.id, e),
    }
    result
}
```

**After:**
```rust
#[aspect(LoggingAspect::new())]
fn process_order(order: Order) -> Result<(), Error> {
    business_logic(order)
}
```

### Pattern 2: Extract Performance Monitoring

**Before:**
```rust
fn slow_operation() -> Result<Data, Error> {
    let start = Instant::now();
    let result = expensive_work();
    let elapsed = start.elapsed();

    if elapsed > Duration::from_millis(100) {
        log::warn!("Slow operation: {:?}", elapsed);
    }

    result
}
```

**After:**
```rust
#[aspect(TimingAspect::with_threshold(Duration::from_millis(100)))]
fn slow_operation() -> Result<Data, Error> {
    expensive_work()
}
```

### Pattern 3: Extract Authorization

**Before:**
```rust
fn delete_user(user_id: u64) -> Result<(), Error> {
    let current_user = get_current_user()?;
    if !current_user.has_role("admin") {
        return Err(Error::Unauthorized);
    }

    database::delete_user(user_id)
}
```

**After:**
```rust
#[aspect(AuthorizationAspect::require_role("admin", get_current_user_roles))]
fn delete_user(user_id: u64) -> Result<(), Error> {
    database::delete_user(user_id)
}
```

---

## Troubleshooting Migration

### Issue: Type Inference Breaks

**Problem:**
```rust
// Before
let result = fetch_user(42);  // Type inferred

// After with aspect - type unclear?
#[aspect(LoggingAspect::new())]
fn fetch_user(id: u64) -> impl SomeTrait { /* ... */ }
```

**Solution:** Use concrete return types:
```rust
#[aspect(LoggingAspect::new())]
fn fetch_user(id: u64) -> User { /* ... */ }
```

### Issue: Async Functions

**Problem:**
```rust
#[aspect(LoggingAspect::new())]
async fn fetch_user(id: u64) -> Result<User, Error> {
    // Does this work?
}
```

**Solution:** Yes! Aspects work with async:
```rust
#[aspect(LoggingAspect::new())]
async fn fetch_user(id: u64) -> Result<User, Error> {
    database::async_query(id).await
}
```

### Issue: Generic Functions

**Problem:**
```rust
#[aspect(LoggingAspect::new())]
fn process<T: Display>(value: T) -> T {
    // Does this work?
    value
}
```

**Solution:** Yes! Generics are preserved:
```rust
#[aspect(LoggingAspect::new())]
fn process<T: Display>(value: T) -> T {
    println!("Processing: {}", value);
    value
}
```

---

## Performance Considerations

### Before Migration: Measure Baseline

```bash
cargo bench
```

### After Migration: Verify Overhead

aspect-rs overhead is typically:
- **No-op aspects**: 0ns (optimized away)
- **Simple aspects**: <10ns
- **Complex aspects**: Comparable to hand-written

### If Performance Critical

1. **Profile first**: Use `perf` or `flamegraph`
2. **Use `#[inline]`** in custom aspects
3. **Selective application**: Only where needed
4. **Measure impact**: Benchmark before/after

---

## Best Practices 

1. **Centralize Aspect Configuration**
   ```rust
   // aspects.rs
   pub fn default_service_aspects() -> Vec<Box<dyn Aspect>> {
       vec![
           Box::new(LoggingAspect::new()),
           Box::new(TimingAspect::new()),
       ]
   }
   ```

2. **Document Aspect Usage**
   ```rust
   /// Fetches user from database.
   ///
   /// Aspects: Logging, Caching
   #[aspect(LoggingAspect::new())]
   #[aspect(CachingAspect::new())]
   fn fetch_user(id: u64) -> User { /* ... */ }
   ```

3. **Test Aspects Independently**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_aspect_behavior() {
           let aspect = LoggingAspect::new();
           // Test aspect logic
       }
   }
   ```

---

## Need Help?

- **Examples**: See `aspect-examples/` directory
- **Documentation**: https://docs.rs/aspect-core
- **Issues**: https://github.com/yourusername/aspect-rs/issues
- **Discussions**: https://github.com/yourusername/aspect-rs/discussions

---

**Happy migrating!** 🚀
