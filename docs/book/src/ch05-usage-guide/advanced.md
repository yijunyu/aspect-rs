# Advanced Patterns

This chapter covers advanced aspect composition, ordering, conditional application, and async patterns.

## Aspect Composition

Multiple aspects can be stacked on a single function. Understanding how they compose is critical for correct behavior.

### Basic Composition

```rust
use aspect_std::{LoggingAspect, TimingAspect, MetricsAspect};

#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(MetricsAspect::new())]
fn process_request(data: RequestData) -> Result<Response, Error> {
    handle_request(data)
}
```

**Execution Order** (outermost to innermost):
1. LoggingAspect::before()
2. TimingAspect::before()
3. MetricsAspect::before()
4. **function execution**
5. MetricsAspect::after()
6. TimingAspect::after()
7. LoggingAspect::after()

### Order Matters

Different orderings produce different behavior:

```rust
// Timing includes logging overhead
#[aspect(TimingAspect::new())]
#[aspect(LoggingAspect::new())]
fn example1() { }

// Timing excludes logging overhead (more accurate)
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn example2() { }
```

**Best Practice**: Place aspects in this order (outer to inner):
1. Authorization (fail fast)
2. Rate limiting (prevent abuse)
3. Circuit breakers (fail fast on known issues)
4. Caching (skip work if possible)
5. Transactions (only for valid requests)
6. Logging (record actual execution)
7. Timing (measure core logic)
8. Metrics (collect statistics)

### Practical Example: Complete API Handler

```rust
use aspect_std::*;
use std::time::Duration;

#[aspect(AuthorizationAspect::require_role("user", get_roles))]
#[aspect(RateLimitAspect::new(100, Duration::from_secs(60)))]
#[aspect(CachingAspect::with_ttl(Duration::from_secs(300)))]
#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::with_threshold(Duration::from_millis(100)))]
#[aspect(MetricsAspect::new())]
fn get_user_data(user_id: u64) -> Result<UserData, Error> {
    // Clean business logic - all infrastructure handled by aspects
    external_service::fetch_user_data(user_id)
}
```

**Execution Flow:**
1. Check authorization → reject if unauthorized
2. Check rate limit → reject if exceeded
3. Check cache → return if hit
4. Check circuit breaker → reject if open
5. Log entry
6. Start timer
7. Record metrics (start)
8. Execute function (call external service)
9. Record metrics (end)
10. Stop timer, warn if > 100ms
11. Log exit
12. Cache result (if success)
13. Return result

## Conditional Aspect Application

Sometimes you want aspects to apply only under certain conditions.

### Runtime Conditions

Use conditional logic within aspect implementation:

```rust
use aspect_core::prelude::*;

struct ConditionalLogger {
    enabled: Arc<AtomicBool>,
}

impl Aspect for ConditionalLogger {
    fn before(&self, ctx: &JoinPoint) {
        if self.enabled.load(Ordering::Relaxed) {
            println!("→ {}", ctx.function_name);
        }
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        if self.enabled.load(Ordering::Relaxed) {
            println!("← {}", ctx.function_name);
        }
    }
}

// Can enable/disable at runtime
#[aspect(ConditionalLogger { enabled: LOGGER_ENABLED.clone() })]
fn monitored_function() -> Result<(), Error> {
    // Logging only occurs if enabled flag is true
    Ok(())
}
```

### Environment-Based Application

Use feature flags or environment variables:

```rust
use aspect_std::LoggingAspect;

// Different aspects for different environments
#[cfg_attr(debug_assertions, aspect(LoggingAspect::verbose()))]
#[cfg_attr(not(debug_assertions), aspect(LoggingAspect::new()))]
fn debug_sensitive_function() -> Result<(), Error> {
    // Verbose logging in debug builds
    // Standard logging in release builds
    Ok(())
}

// Only apply in production
#[cfg_attr(not(debug_assertions), aspect(MetricsAspect::new()))]
fn production_only_metrics() -> Result<(), Error> {
    Ok(())
}
```

### Feature Flag Pattern

```rust
use std::sync::Arc;
use aspect_core::prelude::*;

struct FeatureGatedAspect {
    feature_name: &'static str,
    inner: Arc<dyn Aspect>,
}

impl Aspect for FeatureGatedAspect {
    fn before(&self, ctx: &JoinPoint) {
        if feature_flags::is_enabled(self.feature_name) {
            self.inner.before(ctx);
        }
    }

    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
        if feature_flags::is_enabled(self.feature_name) {
            self.inner.after(ctx, result);
        }
    }
}

#[aspect(FeatureGatedAspect {
    feature_name: "new_metrics_system",
    inner: Arc::new(MetricsAspect::new()),
})]
fn gradually_rolled_out_feature() -> Result<(), Error> {
    // Metrics only collected if feature flag enabled
    Ok(())
}
```

## Aspect Ordering with Dependencies

When aspects depend on each other, explicit ordering is crucial.

### Transaction + Logging Pattern

```rust
// Correct: Logging outside transaction
#[aspect(LoggingAspect::new())]
#[aspect(TransactionalAspect::new())]
fn correct_order(data: Data) -> Result<(), Error> {
    // Logs show:
    // - Transaction start
    // - Business logic
    // - Commit/rollback
    database::save(data)
}

// Incorrect: Transaction outside logging
#[aspect(TransactionalAspect::new())]
#[aspect(LoggingAspect::new())]
fn incorrect_order(data: Data) -> Result<(), Error> {
    // Transaction committed before exit log
    // Rollback information not logged properly
    database::save(data)
}
```

### Caching + Authorization Pattern

```rust
// Correct: Authorization before cache
#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
#[aspect(CachingAspect::new())]
fn secure_cached_data(id: u64) -> Result<SensitiveData, Error> {
    // 1. Check authorization first
    // 2. Only cache for authorized users
    // Prevents unauthorized users from benefiting from cache
    database::fetch_sensitive(id)
}

// Security Issue: Cache before authorization
#[aspect(CachingAspect::new())]
#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
fn insecure_order(id: u64) -> Result<SensitiveData, Error> {
    // BAD: Unauthorized users can populate cache
    // Then authorized users get the cached data
    // Authorization check is ineffective
    database::fetch_sensitive(id)
}
```

## Async Patterns

aspect-rs works seamlessly with async functions. All aspects handle async transparently.

### Basic Async Usage

```rust
use aspect_std::{LoggingAspect, TimingAspect};

#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
async fn fetch_user(id: u64) -> Result<User, Error> {
    database::async_query_user(id).await
}

#[aspect(LoggingAspect::new())]
async fn parallel_operations() -> Result<Vec<Data>, Error> {
    let future1 = fetch_user(1);
    let future2 = fetch_user(2);
    let future3 = fetch_user(3);

    let results = tokio::join!(future1, future2, future3);
    Ok(vec![results.0?, results.1?, results.2?])
}
```

### Async with Caching

```rust
use aspect_std::CachingAspect;
use std::time::Duration;

#[aspect(CachingAspect::with_ttl(Duration::from_secs(60)))]
async fn cached_async_call(key: String) -> Result<Value, Error> {
    // Cache works across async boundaries
    expensive_async_operation(key).await
}
```

### Async with Circuit Breaker

```rust
use aspect_std::CircuitBreakerAspect;
use std::time::Duration;

#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
async fn protected_async_call(url: String) -> Result<Response, Error> {
    // Circuit breaker protects async calls
    reqwest::get(&url).await?.json().await
}
```

## Custom Aspect Composition

Create reusable aspect bundles for common patterns.

### Aspect Bundle Pattern

```rust
use aspect_core::prelude::*;
use std::sync::Arc;

struct WebServiceAspectBundle {
    aspects: Vec<Arc<dyn Aspect>>,
}

impl WebServiceAspectBundle {
    fn new() -> Self {
        Self {
            aspects: vec![
                Arc::new(AuthorizationAspect::require_role("user", get_roles)),
                Arc::new(RateLimitAspect::new(100, Duration::from_secs(60))),
                Arc::new(LoggingAspect::new()),
                Arc::new(TimingAspect::new()),
                Arc::new(MetricsAspect::new()),
            ],
        }
    }
}

impl Aspect for WebServiceAspectBundle {
    fn before(&self, ctx: &JoinPoint) {
        for aspect in &self.aspects {
            aspect.before(ctx);
        }
    }

    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
        for aspect in self.aspects.iter().rev() {
            aspect.after(ctx, result);
        }
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        for aspect in self.aspects.iter().rev() {
            aspect.after_error(ctx, error);
        }
    }
}

// Use bundle on multiple functions
#[aspect(WebServiceAspectBundle::new())]
fn endpoint1(data: Data1) -> Result<Response, Error> {
    handle1(data)
}

#[aspect(WebServiceAspectBundle::new())]
fn endpoint2(data: Data2) -> Result<Response, Error> {
    handle2(data)
}
```

## Summary

Advanced patterns covered:

1. **Composition**: Understanding execution order
2. **Conditional Application**: Runtime and compile-time conditions
3. **Ordering**: Correct aspect ordering for dependencies
4. **Async**: Seamless async/await support
5. **Custom Composition**: Reusable aspect bundles

**Key Takeaways:**
- Aspect order significantly impacts behavior
- Authorization and validation should be outermost
- Async works transparently with aspects
- Custom aspect bundles reduce duplication
- Always measure performance impact

**Next Steps:**
- Review [Configuration](configuration.md) for environment settings
- See [Testing](testing.md) for aspect testing strategies
- Check [Case Studies](../ch08-case-studies/README.md) for real examples
