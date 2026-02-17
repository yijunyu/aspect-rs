# Using Pre-built Aspects

The `aspect-std` crate provides 8 battle-tested, production-ready aspects that cover common use cases.

## Installation

```toml
[dependencies]
aspect-std = "0.1"
```

Import all aspects:

```rust
use aspect_std::*;
```

## 1. LoggingAspect

Automatically log function entry and exit with timestamps.

```rust
use aspect_std::LoggingAspect;

#[aspect(LoggingAspect::new())]
fn process_order(order_id: u64) -> Result<(), Error> {
    database::process(order_id)
}
```

**Features:**
- Structured logging with timestamps
- Function name and location
- Configurable log levels

## 2. TimingAspect

Measure function execution time and warn on slow operations.

```rust
use aspect_std::TimingAspect;
use std::time::Duration;

// Warn if execution > 100ms
#[aspect(TimingAspect::with_threshold(Duration::from_millis(100)))]
fn fetch_data() -> Result<Data, Error> {
    api::get_data()
}
```

**Features:**
- Nanosecond precision
- Configurable thresholds
- Slow function warnings

## 3. CachingAspect

Memoize expensive computations automatically.

```rust
use aspect_std::CachingAspect;

#[aspect(CachingAspect::new())]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}
```

**Features:**
- LRU cache with TTL
- Cache hit/miss metrics
- Thread-safe

## 4. MetricsAspect

Collect call counts and latency distributions.

```rust
use aspect_std::MetricsAspect;

#[aspect(MetricsAspect::new())]
fn api_endpoint() -> Response {
    handle_request()
}
```

**Features:**
- Call counters
- Latency percentiles (p50, p95, p99)
- Prometheus export

## 5. RateLimitAspect

Prevent API abuse with token bucket rate limiting.

```rust
use aspect_std::RateLimitAspect;

// 100 requests per minute
#[aspect(RateLimitAspect::new(100, Duration::from_secs(60)))]
fn api_call() -> Response {
    handle()
}
```

**Features:**
- Token bucket algorithm
- Per-function limits
- Returns error when exceeded

## 6. CircuitBreakerAspect

Handle service failures gracefully with circuit breaker pattern.

```rust
use aspect_std::CircuitBreakerAspect;

// Open after 5 failures, retry after 30s
#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
fn external_service() -> Result<Data, Error> {
    api::call()
}
```

**Features:**
- Automatic failure detection
- Configurable thresholds
- Half-open state for retries

## 7. AuthorizationAspect

Enforce role-based access control (RBAC).

```rust
use aspect_std::AuthorizationAspect;

fn get_user_roles() -> HashSet<String> {
    // Fetch from session
    current_user().roles()
}

#[aspect(AuthorizationAspect::require_role("admin", get_user_roles))]
fn delete_user(id: u64) -> Result<(), Error> {
    database::delete(id)
}
```

**Features:**
- Role-based permissions
- Custom role providers
- Returns Unauthorized error

## 8. ValidationAspect

Validate function arguments before execution.

```rust
use aspect_std::{ValidationAspect, validators};

fn validate_age() -> ValidationAspect {
    ValidationAspect::new(vec![
        Box::new(validators::RangeValidator::new(0, 0, 120)),
    ])
}

#[aspect(validate_age())]
fn set_age(age: i64) -> Result<(), Error> {
    database::update_age(age)
}
```

**Features:**
- Pre-built validators (range, regex, custom)
- Composable validation rules
- Clear error messages

## Combining Pre-built Aspects

All aspects can be combined:

```rust
#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
#[aspect(RateLimitAspect::new(10, Duration::from_secs(60)))]
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(MetricsAspect::new())]
fn sensitive_operation() -> Result<(), Error> {
    // Protected by:
    // - Authorization check
    // - Rate limiting (10/min)
    // - Comprehensive logging
    // - Performance monitoring
    // - Metrics collection
    perform_action()
}
```

## Next Steps

Now that you can use pre-built aspects, dive deeper into [Core Concepts](../ch04-core-concepts/README.md) to understand how they work internally.
