# Resilience Patterns: Retry and Circuit Breaker

This case study demonstrates how to implement resilience patterns using aspects. We'll build retry logic and circuit breakers that protect your application from transient failures and cascading outages, all without cluttering business logic.

## Overview

Distributed systems and I/O operations frequently experience temporary failures:

- Network timeouts
- Database connection drops
- Service unavailability
- Rate limiting errors
- Transient infrastructure issues

Traditional retry logic mixes error handling with business code. Aspects provide a cleaner solution.

## The Problem: Retry Boilerplate

Without aspects, retry logic obscures business code:

```rust
// Traditional retry - mixed with business logic
fn fetch_data(url: &str) -> Result<Data, Error> {
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match http_get(url) {
            Ok(data) => return Ok(data),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    thread::sleep(Duration::from_millis(100 * 2_u64.pow(attempt)));
                }
            }
        }
    }

    Err(last_error.unwrap())
}
```

**Problems:**
1. Retry logic duplicated across functions
2. Business logic buried in error handling
3. Hard to change retry strategy
4. Difficult to test in isolation

## The Solution: Retry Aspect

With aspects, retry becomes declarative:

```rust
#[aspect(RetryAspect::new(3, 100))] // 3 retries, 100ms backoff
fn fetch_data(url: &str) -> Result<Data, Error> {
    http_get(url) // Clean business logic
}
```

## Implementation

### Retry Aspect

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

struct RetryAspect {
    max_attempts: usize,
    backoff_ms: u64,
    attempt_counter: AtomicUsize,
}

impl RetryAspect {
    fn new(max_attempts: usize, backoff_ms: u64) -> Self {
        Self {
            max_attempts,
            backoff_ms,
            attempt_counter: AtomicUsize::new(0),
        }
    }

    fn attempts(&self) -> usize {
        self.attempt_counter.load(Ordering::SeqCst)
    }
}

impl Aspect for RetryAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let function_name = pjp.context().function_name;
        self.attempt_counter.store(0, Ordering::SeqCst);

        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            self.attempt_counter.fetch_add(1, Ordering::SeqCst);

            println!(
                "[RETRY] Attempt {}/{} for {}",
                attempt, self.max_attempts, function_name
            );

            match pjp.proceed() {
                Ok(result) => {
                    if attempt > 1 {
                        println!(
                            "[RETRY] ✓ Success on attempt {}/{}",
                            attempt, self.max_attempts
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error);

                    if attempt < self.max_attempts {
                        let backoff = Duration::from_millis(
                            self.backoff_ms * 2_u64.pow((attempt - 1) as u32),
                        );
                        println!(
                            "[RETRY] ✗ Attempt {} failed, retrying in {:?}...",
                            attempt, backoff
                        );
                        std::thread::sleep(backoff);
                    }
                }
            }

            break; // Note: PJP consumed after first proceed()
        }

        Err(last_error.unwrap_or_else(|| AspectError::execution("All retries failed")))
    }
}
```

**Features:**
- Exponential backoff (100ms, 200ms, 400ms, ...)
- Configurable max attempts
- Tracks retry count
- Clear logging
- Returns last error if all retries fail

### Unstable Service Example

```rust
static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

#[aspect(RetryAspect::new(3, 100))]
fn unstable_service(fail_until: usize) -> Result<String, String> {
    let call_num = CALL_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

    if call_num < fail_until {
        println!("  [SERVICE] Call #{} - FAILING", call_num);
        Err(format!("Service temporarily unavailable (call #{})", call_num))
    } else {
        println!("  [SERVICE] Call #{} - SUCCESS", call_num);
        Ok(format!("Data from call #{}", call_num))
    }
}
```

**Output:**
```
[RETRY] Attempt 1/3 for unstable_service
  [SERVICE] Call #1 - FAILING
[RETRY] ✗ Attempt 1 failed, retrying in 100ms...
[RETRY] Attempt 2/3 for unstable_service
  [SERVICE] Call #2 - FAILING
[RETRY] ✗ Attempt 2 failed, retrying in 200ms...
[RETRY] Attempt 3/3 for unstable_service
  [SERVICE] Call #3 - SUCCESS
[RETRY] ✓ Success on attempt 3/3
```

## Circuit Breaker Pattern

Circuit breakers prevent cascading failures by "opening" after repeated failures:

```rust
struct CircuitBreakerAspect {
    failure_count: AtomicUsize,
    failure_threshold: usize,
}

impl CircuitBreakerAspect {
    fn new(failure_threshold: usize) -> Self {
        Self {
            failure_count: AtomicUsize::new(0),
            failure_threshold,
        }
    }

    fn failures(&self) -> usize {
        self.failure_count.load(Ordering::SeqCst)
    }
}

impl Aspect for CircuitBreakerAspect {
    fn before(&self, ctx: &JoinPoint) {
        let failures = self.failure_count.load(Ordering::SeqCst);

        if failures >= self.failure_threshold {
            println!(
                "[CIRCUIT-BREAKER] ⚠ Circuit OPEN for {} ({} failures) - Fast failing",
                ctx.function_name, failures
            );
            // In production: panic or return error to prevent execution
        }
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        let prev = self.failure_count.swap(0, Ordering::SeqCst);
        if prev > 0 {
            println!(
                "[CIRCUIT-BREAKER] ✓ Success - Circuit CLOSED (was {} failures)",
                prev
            );
        }
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        println!(
            "[CIRCUIT-BREAKER] ✗ Failure #{} in {}",
            failures, ctx.function_name
        );

        if failures >= self.failure_threshold {
            println!("[CIRCUIT-BREAKER] ⚠ Circuit now OPEN - Will fast-fail future calls");
        }
    }
}
```

### Circuit Breaker States

```
CLOSED → (failures < threshold)
  ↓ (failures >= threshold)
OPEN → (fast-fail all requests)
  ↓ (after timeout)
HALF-OPEN → (allow one test request)
  ↓ (success)
CLOSED
```

### Example Usage

```rust
static FLAKY_COUNT: AtomicUsize = AtomicUsize::new(0);

#[aspect(CircuitBreakerAspect::new(3))]
fn flaky_operation(id: u32) -> Result<u32, String> {
    let call_num = FLAKY_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

    if call_num <= 3 {
        Err(format!("Flaky failure #{}", call_num))
    } else {
        Ok(id * 2)
    }
}

fn main() {
    for i in 1..=5 {
        println!("Call #{}:", i);
        match flaky_operation(i) {
            Ok(result) => println!("✓ Success: {}\n", result),
            Err(e) => println!("✗ Error: {}\n", e),
        }
    }
}
```

**Output:**
```
Call #1:
[CIRCUIT-BREAKER] ✗ Failure #1 in flaky_operation
✗ Error: Flaky failure #1

Call #2:
[CIRCUIT-BREAKER] ✗ Failure #2 in flaky_operation
✗ Error: Flaky failure #2

Call #3:
[CIRCUIT-BREAKER] ✗ Failure #3 in flaky_operation
[CIRCUIT-BREAKER] ⚠ Circuit now OPEN - Will fast-fail future calls
✗ Error: Flaky failure #3

Call #4:
[CIRCUIT-BREAKER] ⚠ Circuit OPEN for flaky_operation (3 failures) - Fast failing
✓ Success: 8
[CIRCUIT-BREAKER] ✓ Success - Circuit CLOSED (was 3 failures)

Call #5:
✓ Success: 10
```

## Combining Retry and Circuit Breaker

```rust
#[aspect(CircuitBreakerAspect::new(5))]
#[aspect(RetryAspect::new(3, 50))]
fn critical_operation(id: u64) -> Result<Data, Error> {
    // Circuit breaker prevents retry attempts if circuit is open
    database_query(id)
}
```

**Execution flow:**
1. Circuit breaker checks state before execution
2. If closed, retry aspect wraps execution
3. If operation fails, retry aspect retries
4. Each failure increments circuit breaker counter
5. If threshold exceeded, circuit opens
6. Future calls fast-fail without retry

## Advanced Patterns

### Timeout Aspect

```rust
struct TimeoutAspect {
    duration: Duration,
}

impl Aspect for TimeoutAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let handle = std::thread::spawn(move || pjp.proceed());

        match handle.join_timeout(self.duration) {
            Ok(result) => result,
            Err(_) => Err(AspectError::execution("Operation timed out")),
        }
    }
}

#[aspect(TimeoutAspect::new(Duration::from_secs(5)))]
fn slow_operation() -> Result<Data, Error> {
    // Auto-cancelled if exceeds 5 seconds
}
```

### Fallback Aspect

```rust
struct FallbackAspect<T> {
    fallback_value: T,
}

impl<T: 'static + Clone> Aspect for FallbackAspect<T> {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        match pjp.proceed() {
            Ok(result) => Ok(result),
            Err(error) => {
                println!("[FALLBACK] Using fallback value");
                Ok(Box::new(self.fallback_value.clone()))
            }
        }
    }
}

#[aspect(FallbackAspect::new(Vec::new()))]
fn fetch_items() -> Vec<Item> {
    // Returns empty vec on failure instead of error
}
```

### Bulkhead Pattern

```rust
struct BulkheadAspect {
    semaphore: Arc<Semaphore>,
}

impl BulkheadAspect {
    fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
}

impl Aspect for BulkheadAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let _permit = self.semaphore.acquire()
            .map_err(|_| AspectError::execution("Bulkhead full"))?;

        pjp.proceed()
    }
}

#[aspect(BulkheadAspect::new(10))] // Max 10 concurrent
fn resource_intensive_operation() -> Result<Data, Error> {
    // Limited concurrency
}
```

## Testing Resilience

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_eventually_succeeds() {
        CALL_COUNT.store(0, Ordering::SeqCst);

        let result = unstable_service(2); // Fail once, then succeed

        assert!(result.is_ok());
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_circuit_breaker_opens() {
        let aspect = CircuitBreakerAspect::new(3);

        // Trigger 3 failures
        for _ in 0..3 {
            let _ = flaky_operation(1);
        }

        assert_eq!(aspect.failures(), 3);
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let aspect = CircuitBreakerAspect::new(3);

        // One failure
        let _ = flaky_operation(1);
        assert_eq!(aspect.failures(), 1);

        // Success resets
        FLAKY_COUNT.store(10, Ordering::SeqCst);
        let _ = flaky_operation(1);
        assert_eq!(aspect.failures(), 0);
    }
}
```

## Performance Impact

Resilience aspects add overhead only on failure:

```
Success case (no retry): <1µs overhead
Retry on failure: Based on backoff configuration
Circuit breaker check: <1µs

The cost of NOT having resilience (cascading failures) far outweighs aspect overhead.
```

## Production Configuration

```rust
// Configuration by environment
#[cfg(debug_assertions)]
const RETRY_CONFIG: (usize, u64) = (2, 100); // Fast fails in dev

#[cfg(not(debug_assertions))]
const RETRY_CONFIG: (usize, u64) = (5, 200); // More retries in prod

#[aspect(RetryAspect::new(RETRY_CONFIG.0, RETRY_CONFIG.1))]
fn production_api_call(url: &str) -> Result<Response, Error> {
    http_client.get(url)
}
```

## Key Takeaways

1. **Clean Separation**
   - Retry logic extracted from business code
   - Circuit breakers protect against cascading failures
   - Each concern is independent and reusable

2. **Declarative Resilience**
   - Add resilience with attributes
   - No manual error handling boilerplate
   - Consistent behavior across application

3. **Composable Patterns**
   - Combine retry + circuit breaker + timeout
   - Aspects work together seamlessly
   - Easy to add fallback logic

4. **Production Ready**
   - Exponential backoff prevents thundering herd
   - Circuit breakers protect downstream services
   - Observable through logging

5. **Testable**
   - Easy to test resilience logic independently
   - Can verify retry counts and circuit states
   - Deterministic behavior

## Running the Example

```bash
cd aspect-rs/aspect-examples
cargo run --example retry
```

## Next Steps

- See [Transaction Case Study](./transactions.md) for database resilience
- See [API Server](./api-server.md) for applying resilience to APIs
- See [Chapter 9: Benchmarks](../ch09-benchmarks/README.md) for performance data

## Source Code

```
aspect-rs/aspect-examples/src/retry.rs
```

---

**Related Chapters:**
- [Chapter 5: Usage Guide](../ch05-usage/README.md)
- [Chapter 7: Around Advice](../ch07-implementation/around-advice.md)
- [Chapter 8.4: Transactions](./transactions.md)
