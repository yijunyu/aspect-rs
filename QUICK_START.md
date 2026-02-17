# Quick Start Guide - aspect-rs

Get up and running with aspect-rs in under 5 minutes!

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aspect-core = "0.1"
aspect-macros = "0.1"
aspect-std = "0.1"  # Optional: pre-built aspects
```

## Your First Aspect

### Step 1: Create a Simple Aspect

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

### Step 2: Use Pre-Built Aspects

```rust
use aspect_macros::aspect;
use aspect_std::*;

// Add timing to any function
#[aspect(TimingAspect::new())]
fn slow_operation() -> Result<String, Box<dyn std::error::Error>> {
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok("Done!".to_string())
}

// Stack multiple aspects
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn important_function(x: i32) -> i32 {
    x * 2
}
```

## Common Use Cases

### 1. Logging

```rust
use aspect_std::LoggingAspect;

#[aspect(LoggingAspect::new())]
fn process_order(order_id: u64) -> Result<(), Error> {
    // Your business logic
    database::process(order_id)
}
```

### 2. Performance Monitoring

```rust
use aspect_std::TimingAspect;
use std::time::Duration;

#[aspect(TimingAspect::with_threshold(Duration::from_millis(100)))]
fn fetch_data(url: &str) -> Result<String, Error> {
    // Warns if execution > 100ms
    reqwest::blocking::get(url)?.text()
}
```

### 3. Caching

```rust
use aspect_std::CachingAspect;

#[aspect(CachingAspect::new())]
fn expensive_calculation(n: u64) -> u64 {
    // Result is cached automatically
    fibonacci(n)
}
```

### 4. Rate Limiting

```rust
use aspect_std::RateLimitAspect;
use std::time::Duration;

// Limit to 100 calls per minute
#[aspect(RateLimitAspect::new(100, Duration::from_secs(60)))]
fn api_endpoint(request: Request) -> Response {
    handle_request(request)
}
```

### 5. Authorization

```rust
use aspect_std::AuthorizationAspect;
use std::collections::HashSet;

fn get_user_roles() -> HashSet<String> {
    // Fetch from session/database
    vec!["admin".to_string()].into_iter().collect()
}

#[aspect(AuthorizationAspect::require_role("admin", get_user_roles))]
fn delete_user(user_id: u64) -> Result<(), Error> {
    // Only admins can execute this
    database::delete_user(user_id)
}
```

### 6. Circuit Breaker

```rust
use aspect_std::CircuitBreakerAspect;
use std::time::Duration;

// Opens circuit after 5 failures, retries after 30s
#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
fn call_external_service(url: &str) -> Result<Response, Error> {
    reqwest::blocking::get(url)?.json()
}
```

### 7. Validation

```rust
use aspect_std::{ValidationAspect, validators};

fn validate_age() -> ValidationAspect {
    ValidationAspect::new(vec![
        Box::new(validators::RangeValidator::new(0, 0, 120)),
    ])
}

#[aspect(validate_age())]
fn set_user_age(age: i64) -> Result<(), Error> {
    // Validates age is 0-120 before executing
    database::update_age(age)
}
```

## Custom Aspects

Create your own aspects for specific needs:

```rust
use aspect_core::prelude::*;
use std::time::Instant;

struct PerformanceMonitor {
    threshold: Duration,
}

impl Aspect for PerformanceMonitor {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let start = Instant::now();
        let result = pjp.proceed();
        let elapsed = start.elapsed();

        if elapsed > self.threshold {
            eprintln!("⚠️  SLOW: {} took {:?}", pjp.context().function_name, elapsed);
        }

        result
    }
}

#[aspect(PerformanceMonitor { threshold: Duration::from_millis(50) })]
fn critical_operation() -> Result<(), Error> {
    // Your code here
}
```

## Combining Aspects

Stack multiple aspects for complex behavior:

```rust
use aspect_std::*;

#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(MetricsAspect::new())]
fn admin_operation(action: &str) -> Result<(), Error> {
    // All aspects applied in order:
    // 1. Check authorization
    // 2. Log entry/exit
    // 3. Measure time
    // 4. Record metrics
    perform_action(action)
}
```

## Async Functions

Aspects work with async functions too:

```rust
use aspect_std::LoggingAspect;

#[aspect(LoggingAspect::new())]
async fn fetch_user(id: u64) -> Result<User, Error> {
    database::async_query_user(id).await
}
```

## Best Practices

### ✅ DO

- Use aspects for cross-cutting concerns (logging, timing, security)
- Keep aspect logic simple and focused
- Test aspects independently
- Document aspect behavior
- Use pre-built aspects from `aspect-std` when possible

### ❌ DON'T

- Put business logic in aspects
- Create aspects with side effects that depend on execution order
- Use aspects for one-off functionality (just write normal code)
- Over-apply aspects (be selective)

## Performance Tips

1. **Aspects are zero-cost when removed** - if you remove the `#[aspect]` attribute, there's no overhead

2. **Simple aspects have minimal overhead** - logging/timing typically <10ns

3. **Use `#[inline]` in performance-critical aspects**:
   ```rust
   impl Aspect for FastAspect {
       #[inline(always)]
       fn before(&self, ctx: &JoinPoint) {
           // Fast path
       }
   }
   ```

4. **Profile before optimizing** - use benchmarks to measure actual impact

## Troubleshooting

### "Cannot find type `Aspect` in this scope"

Make sure you import the prelude:
```rust
use aspect_core::prelude::*;
```

### Aspect not being called

1. Verify the `#[aspect(...)]` attribute is present
2. Check that the aspect implements the `Aspect` trait
3. Ensure you're calling the function (not just defining it)

### Compilation errors with generics

Aspects work with generic functions, but ensure your aspect can handle `Box<dyn Any>`:

```rust
#[aspect(LoggingAspect::new())]
fn generic_function<T: Display>(value: T) -> T {
    value
}
```

## Next Steps

- **Read the [examples](aspect-examples/)** for real-world patterns
- **Check [BENCHMARKS.md](BENCHMARKS.md)** for performance analysis
- **See [CONTRIBUTING.md](CONTRIBUTING.md)** to contribute

## Support

- **Documentation**: https://docs.rs/aspect-core
- **GitHub Issues**: https://github.com/yourusername/aspect-rs/issues
- **Discussions**: https://github.com/yourusername/aspect-rs/discussions

---

**Happy aspect-oriented programming!** 🚀
