# Quick Start Guide

This comprehensive guide will have you productive with aspect-rs in 5 minutes. We'll cover the most common use cases with working code examples.

## Step 1: Create a Simple Aspect

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;

// Define an aspect
#[derive(Default)]
struct Logger;

impl Aspect for Logger {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ Entering: {}", ctx.function_name);
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn std::any::Any) {
        println!("← Exiting: {}", ctx.function_name);
    }
}

// Apply it to any function
#[aspect(Logger)]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    let greeting = greet("World");
    println!("{}", greeting);
}
```

**Output:**
```
→ Entering: greet
← Exiting: greet
Hello, World!
```

## Step 2: Use Pre-Built Aspects

aspect-rs includes 8 production-ready aspects in `aspect-std`:

### Logging
```rust
use aspect_std::LoggingAspect;

#[aspect(LoggingAspect::new())]
fn process_order(order_id: u64) -> Result<(), Error> {
    database::process(order_id)
}
```

### Performance Monitoring
```rust
use aspect_std::TimingAspect;
use std::time::Duration;

#[aspect(TimingAspect::with_threshold(Duration::from_millis(100)))]
fn fetch_data(url: &str) -> Result<String, Error> {
    reqwest::blocking::get(url)?.text()
}
```

### Caching
```rust
use aspect_std::CachingAspect;

#[aspect(CachingAspect::new())]
fn expensive_calculation(n: u64) -> u64 {
    fibonacci(n)  // Result cached automatically!
}
```

### Rate Limiting
```rust
use aspect_std::RateLimitAspect;

// 100 calls per minute
#[aspect(RateLimitAspect::new(100, Duration::from_secs(60)))]
fn api_endpoint(request: Request) -> Response {
    handle_request(request)
}
```

### Authorization
```rust
use aspect_std::AuthorizationAspect;

fn get_user_roles() -> HashSet<String> {
    vec!["admin".to_string()].into_iter().collect()
}

#[aspect(AuthorizationAspect::require_role("admin", get_user_roles))]
fn delete_user(user_id: u64) -> Result<(), Error> {
    database::delete_user(user_id)
}
```

### Circuit Breaker
```rust
use aspect_std::CircuitBreakerAspect;

// Opens after 5 failures, retries after 30s
#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
fn call_external_service(url: &str) -> Result<Response, Error> {
    reqwest::blocking::get(url)?.json()
}
```

## Step 3: Combine Multiple Aspects

Stack aspects for complex behavior:

```rust
use aspect_std::*;

#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(MetricsAspect::new())]
fn admin_operation(action: &str) -> Result<(), Error> {
    perform_action(action)
}
```

**Execution order** (outermost first):
1. Check authorization
2. Log entry
3. Start timer
4. Record metrics
5. Execute function
6. Record metrics (after)
7. Stop timer
8. Log exit
9. Return result

## Step 4: Create Custom Aspects

For specific needs, create your own aspects:

```rust
use aspect_core::prelude::*;
use std::time::{Instant, Duration};

struct PerformanceMonitor {
    threshold: Duration,
}

impl Aspect for PerformanceMonitor {
    fn around(&self, mut pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let start = Instant::now();
        let result = pjp.proceed()?;
        let elapsed = start.elapsed();

        if elapsed > self.threshold {
            eprintln!("⚠️ SLOW: {} took {:?}", pjp.function_name, elapsed);
        }

        Ok(result)
    }
}

#[aspect(PerformanceMonitor { threshold: Duration::from_millis(50) })]
fn critical_operation() -> Result<(), Error> {
    // Your code here
}
```

## Async Functions

Aspects work seamlessly with async functions:

```rust
use aspect_std::LoggingAspect;

#[aspect(LoggingAspect::new())]
async fn fetch_user(id: u64) -> Result<User, Error> {
    database::async_query_user(id).await
}
```

## Best Practices

### ✅ DO
- Use aspects for crosscutting concerns (logging, metrics, security)
- Keep aspect logic simple and focused
- Use `aspect-std` pre-built aspects when possible
- Test aspects independently
- Document aspect behavior

### ❌ DON'T
- Put business logic in aspects
- Use aspects for one-off functionality
- Create aspects with hidden side effects
- Over-apply aspects everywhere

## Next Steps

- Learn about [pre-built aspects](prebuilt.md) in detail
- Explore [Core Concepts](../ch04-core-concepts/README.md) for deeper understanding
- See [Case Studies](../ch08-case-studies/README.md) for real-world examples
- Check [Performance Benchmarks](../ch09-benchmarks/README.md) for overhead analysis
