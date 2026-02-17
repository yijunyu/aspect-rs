# Testing Aspects

Comprehensive strategies for testing custom aspects and aspect-enhanced functions.

## Unit Testing Aspects

Test aspects in isolation to verify their behavior.

### Testing Before/After Advice

```rust
use aspect_core::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_aspect_before() {
        let aspect = LoggingAspect::new();
        let ctx = JoinPoint {
            function_name: "test_function",
            module_path: "test::module",
            location: Location {
                file: "test.rs",
                line: 42,
                column: 10,
            },
        };

        // Capture output
        let output = capture_output(|| {
            aspect.before(&ctx);
        });

        assert!(output.contains("test_function"));
        assert!(output.contains("[ENTRY]"));
    }

    #[test]
    fn test_logging_aspect_after() {
        let aspect = LoggingAspect::new();
        let ctx = JoinPoint {
            function_name: "test_function",
            module_path: "test::module",
            location: Location {
                file: "test.rs",
                line: 42,
                column: 10,
            },
        };

        let result: i32 = 42;
        let boxed_result: Box<dyn Any> = Box::new(result);

        let output = capture_output(|| {
            aspect.after(&ctx, boxed_result.as_ref());
        });

        assert!(output.contains("test_function"));
        assert!(output.contains("[EXIT]"));
    }

    #[test]
    fn test_logging_aspect_error() {
        let aspect = LoggingAspect::new();
        let ctx = JoinPoint {
            function_name: "test_function",
            module_path: "test::module",
            location: Location {
                file: "test.rs",
                line: 42,
                column: 10,
            },
        };

        let error = AspectError::execution("test error");

        let output = capture_stderr(|| {
            aspect.after_error(&ctx, &error);
        });

        assert!(output.contains("test_function"));
        assert!(output.contains("test error"));
        assert!(output.contains("[ERROR]"));
    }
}
```

### Testing Around Advice

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_aspect_success_first_try() {
        let aspect = RetryAspect::new(3, Duration::from_millis(10));

        let mut call_count = 0;
        let pjp = create_test_pjp(|| {
            call_count += 1;
            Ok(Box::new(42) as Box<dyn Any>)
        });

        let result = aspect.around(pjp);

        assert!(result.is_ok());
        assert_eq!(call_count, 1); // Success on first try
    }

    #[test]
    fn test_retry_aspect_success_after_retries() {
        let aspect = RetryAspect::new(3, Duration::from_millis(10));

        let mut call_count = 0;
        let pjp = create_test_pjp(|| {
            call_count += 1;
            if call_count < 3 {
                Err(AspectError::execution("temporary failure"))
            } else {
                Ok(Box::new(42) as Box<dyn Any>)
            }
        });

        let result = aspect.around(pjp);

        assert!(result.is_ok());
        assert_eq!(call_count, 3); // Success on third try
    }

    #[test]
    fn test_retry_aspect_all_attempts_fail() {
        let aspect = RetryAspect::new(3, Duration::from_millis(10));

        let mut call_count = 0;
        let pjp = create_test_pjp(|| {
            call_count += 1;
            Err(AspectError::execution("permanent failure"))
        });

        let result = aspect.around(pjp);

        assert!(result.is_err());
        assert_eq!(call_count, 3); // All attempts exhausted
    }

    // Helper to create test ProceedingJoinPoint
    fn create_test_pjp<F>(f: F) -> ProceedingJoinPoint
    where
        F: FnOnce() -> Result<Box<dyn Any>, AspectError> + 'static,
    {
        ProceedingJoinPoint::new(
            f,
            &JoinPoint {
                function_name: "test",
                module_path: "test",
                location: Location {
                    file: "test.rs",
                    line: 1,
                    column: 1,
                },
            },
        )
    }
}
```

### Testing Stateful Aspects

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_timing_aspect_records_duration() {
        let timings = Arc::new(Mutex::new(Vec::new()));
        let aspect = TimingAspect::with_callback({
            let timings = timings.clone();
            move |duration| {
                timings.lock().unwrap().push(duration);
            }
        });

        let ctx = create_test_joinpoint();

        aspect.before(&ctx);
        std::thread::sleep(Duration::from_millis(10));
        aspect.after(&ctx, &Box::new(()));

        let recorded = timings.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert!(recorded[0] >= Duration::from_millis(10));
    }

    #[test]
    fn test_metrics_aspect_counts_calls() {
        let aspect = MetricsAspect::new();
        let ctx = create_test_joinpoint();

        // Simulate multiple calls
        for _ in 0..5 {
            aspect.before(&ctx);
            aspect.after(&ctx, &Box::new(()));
        }

        let stats = aspect.get_stats("test_function");
        assert_eq!(stats.call_count, 5);
        assert_eq!(stats.success_count, 5);
        assert_eq!(stats.error_count, 0);
    }

    #[test]
    fn test_metrics_aspect_tracks_errors() {
        let aspect = MetricsAspect::new();
        let ctx = create_test_joinpoint();

        // Successful calls
        aspect.before(&ctx);
        aspect.after(&ctx, &Box::new(()));

        aspect.before(&ctx);
        aspect.after(&ctx, &Box::new(()));

        // Failed call
        aspect.before(&ctx);
        aspect.after_error(&ctx, &AspectError::execution("error"));

        let stats = aspect.get_stats("test_function");
        assert_eq!(stats.call_count, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.error_count, 1);
    }
}
```

## Integration Testing

Test functions with aspects applied.

### Testing Aspect-Enhanced Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[aspect(LoggingAspect::new())]
    fn function_under_test(x: i32) -> Result<i32, String> {
        if x < 0 {
            Err("Negative input".to_string())
        } else {
            Ok(x * 2)
        }
    }

    #[test]
    fn test_function_success_case() {
        let output = capture_output(|| {
            let result = function_under_test(5);
            assert_eq!(result, Ok(10));
        });

        // Verify logging occurred
        assert!(output.contains("[ENTRY]"));
        assert!(output.contains("[EXIT]"));
        assert!(output.contains("function_under_test"));
    }

    #[test]
    fn test_function_error_case() {
        let stderr = capture_stderr(|| {
            let result = function_under_test(-5);
            assert!(result.is_err());
        });

        // Verify error logging occurred
        assert!(stderr.contains("[ERROR]"));
        assert!(stderr.contains("Negative input"));
    }
}
```

### Testing Multiple Aspects

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[aspect(LoggingAspect::new())]
    #[aspect(TimingAspect::new())]
    #[aspect(MetricsAspect::new())]
    fn multi_aspect_function(x: i32) -> i32 {
        x * 2
    }

    #[test]
    fn test_all_aspects_execute() {
        // Capture all outputs
        let (stdout, stderr, result) = capture_all(|| {
            multi_aspect_function(21)
        });

        assert_eq!(result, 42);

        // Verify logging
        assert!(stdout.contains("[ENTRY]"));
        assert!(stdout.contains("[EXIT]"));

        // Verify timing
        assert!(stdout.contains("took"));

        // Verify metrics (would need metrics API to check)
    }
}
```

### Testing Aspect Ordering

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Track execution order
    static EXECUTION_ORDER: Mutex<Vec<String>> = Mutex::new(Vec::new());

    struct OrderTrackingAspect {
        name: String,
    }

    impl Aspect for OrderTrackingAspect {
        fn before(&self, _ctx: &JoinPoint) {
            EXECUTION_ORDER.lock().unwrap().push(format!("{}_before", self.name));
        }

        fn after(&self, _ctx: &JoinPoint, _result: &dyn Any) {
            EXECUTION_ORDER.lock().unwrap().push(format!("{}_after", self.name));
        }
    }

    #[aspect(OrderTrackingAspect { name: "A".into() })]
    #[aspect(OrderTrackingAspect { name: "B".into() })]
    #[aspect(OrderTrackingAspect { name: "C".into() })]
    fn ordered_function() -> i32 {
        EXECUTION_ORDER.lock().unwrap().push("function".into());
        42
    }

    #[test]
    fn test_aspect_execution_order() {
        EXECUTION_ORDER.lock().unwrap().clear();

        let result = ordered_function();
        assert_eq!(result, 42);

        let order = EXECUTION_ORDER.lock().unwrap();
        assert_eq!(
            *order,
            vec![
                "A_before",
                "B_before",
                "C_before",
                "function",
                "C_after",
                "B_after",
                "A_after",
            ]
        );
    }
}
```

## Mock Aspects for Testing

Create mock aspects for testing without side effects.

### Mock Logging Aspect

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct MockLoggingAspect {
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

struct LogEntry {
    level: LogLevel,
    message: String,
    function_name: String,
}

impl MockLoggingAspect {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }
}

impl Aspect for MockLoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        self.logs.lock().unwrap().push(LogEntry {
            level: LogLevel::Info,
            message: format!("Entering {}", ctx.function_name),
            function_name: ctx.function_name.to_string(),
        });
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        self.logs.lock().unwrap().push(LogEntry {
            level: LogLevel::Info,
            message: format!("Exiting {}", ctx.function_name),
            function_name: ctx.function_name.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_mock_logging() {
        let mock = MockLoggingAspect::new();

        #[aspect(mock.clone())]
        fn test_function(x: i32) -> i32 {
            x * 2
        }

        let result = test_function(21);
        assert_eq!(result, 42);

        let logs = mock.get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message, "Entering test_function");
        assert_eq!(logs[1].message, "Exiting test_function");
    }
}
```

### Mock Circuit Breaker

```rust
struct MockCircuitBreaker {
    should_fail: Arc<AtomicBool>,
    call_count: Arc<AtomicUsize>,
}

impl MockCircuitBreaker {
    fn new() -> Self {
        Self {
            should_fail: Arc::new(AtomicBool::new(false)),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn set_should_fail(&self, fail: bool) {
        self.should_fail.store(fail, Ordering::Relaxed);
    }

    fn get_call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }
}

impl Aspect for MockCircuitBreaker {
    fn before(&self, ctx: &JoinPoint) {
        self.call_count.fetch_add(1, Ordering::Relaxed);

        if self.should_fail.load(Ordering::Relaxed) {
            panic!("Circuit breaker: {} - Circuit open", ctx.function_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_allows_when_closed() {
        let cb = MockCircuitBreaker::new();
        cb.set_should_fail(false);

        #[aspect(cb.clone())]
        fn test_fn() -> i32 {
            42
        }

        let result = test_fn();
        assert_eq!(result, 42);
        assert_eq!(cb.get_call_count(), 1);
    }

    #[test]
    #[should_panic(expected = "Circuit open")]
    fn test_circuit_breaker_fails_when_open() {
        let cb = MockCircuitBreaker::new();
        cb.set_should_fail(true);

        #[aspect(cb.clone())]
        fn test_fn() -> i32 {
            42
        }

        test_fn(); // Should panic
    }
}
```

## Property-Based Testing

Use property-based testing for comprehensive coverage.

### Testing Aspect Invariants

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_timing_aspect_always_positive(delay_ms in 0u64..100) {
        let aspect = TimingAspect::new();
        let ctx = create_test_joinpoint();

        aspect.before(&ctx);
        std::thread::sleep(Duration::from_millis(delay_ms));
        aspect.after(&ctx, &Box::new(()));

        let duration = aspect.get_last_duration();
        prop_assert!(duration >= Duration::from_millis(delay_ms));
    }

    #[test]
    fn test_retry_aspect_never_exceeds_max_attempts(
        max_attempts in 1usize..10,
        should_succeed_at in prop::option::of(0usize..10)
    ) {
        let aspect = RetryAspect::new(max_attempts, Duration::from_millis(1));
        let mut actual_attempts = 0;

        let pjp = create_test_pjp(|| {
            actual_attempts += 1;
            if let Some(succeed_at) = should_succeed_at {
                if actual_attempts >= succeed_at {
                    return Ok(Box::new(42) as Box<dyn Any>);
                }
            }
            Err(AspectError::execution("fail"))
        });

        let _ = aspect.around(pjp);

        prop_assert!(actual_attempts <= max_attempts);
    }
}
```

## Testing Async Aspects

Test aspects with async functions.

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use tokio::test;

    #[aspect(LoggingAspect::new())]
    async fn async_function(x: i32) -> Result<i32, String> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(x * 2)
    }

    #[tokio::test]
    async fn test_async_function_with_aspect() {
        let output = capture_output(|| async {
            let result = async_function(21).await;
            assert_eq!(result, Ok(42));
        }).await;

        assert!(output.contains("[ENTRY]"));
        assert!(output.contains("[EXIT]"));
    }

    #[tokio::test]
    async fn test_concurrent_async_calls() {
        let handles: Vec<_> = (0..10)
            .map(|i| {
                tokio::spawn(async move {
                    async_function(i).await
                })
            })
            .collect();

        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert_eq!(result, Ok(i * 2));
        }
    }
}
```

## Test Helpers

Utility functions for testing aspects.

```rust
// Helper to create test JoinPoint
fn create_test_joinpoint() -> JoinPoint {
    JoinPoint {
        function_name: "test_function",
        module_path: "test::module",
        location: Location {
            file: "test.rs",
            line: 42,
            column: 10,
        },
    }
}

// Helper to capture stdout
fn capture_output<F, R>(f: F) -> (String, R)
where
    F: FnOnce() -> R,
{
    use std::sync::Mutex;
    static CAPTURED: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    let result = f();
    let captured = CAPTURED.lock().unwrap();
    let output = String::from_utf8_lossy(&captured).to_string();

    (output, result)
}

// Helper to capture stderr
fn capture_stderr<F, R>(f: F) -> (String, R)
where
    F: FnOnce() -> R,
{
    // Similar implementation for stderr
}
```

## Best Practices

1. **Test Aspects in Isolation**: Unit test aspect behavior separately
2. **Test Integration**: Verify aspects work with real functions
3. **Use Mocks**: Create mock aspects for testing without side effects
4. **Test Ordering**: Verify aspect execution order
5. **Test Error Cases**: Ensure error handling works correctly
6. **Property-Based Testing**: Use proptest for comprehensive coverage
7. **Async Testing**: Test async functions with aspects
8. **Test Performance**: Benchmark aspect overhead

## Summary

Testing strategies covered:

1. **Unit Testing**: Test aspects in isolation
2. **Integration Testing**: Test with real functions
3. **Mock Aspects**: Testing without side effects
4. **Property-Based Testing**: Comprehensive coverage
5. **Async Testing**: Testing async functions
6. **Test Helpers**: Utility functions for testing

**Key Takeaways:**
- Always test aspects in isolation first
- Use mocks to avoid side effects in tests
- Test aspect ordering explicitly
- Property-based testing finds edge cases
- Async aspects need special test handling

**Next Steps:**
- See [Case Studies](../ch08-case-studies/README.md) for real-world examples
- Review [Production Patterns](production.md) for best practices
- Check [Advanced Patterns](advanced.md) for complex scenarios
