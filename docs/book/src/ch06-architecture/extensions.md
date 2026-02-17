# Extending the Framework

aspect-rs is designed for extensibility. This chapter shows how to create custom aspects, custom pointcuts, and extend the framework for specialized needs.

## Creating Custom Aspects

### Basic Custom Aspect

The simplest extension is creating a custom aspect:

```rust
use aspect_core::prelude::*;
use std::any::Any;

pub struct MyCustomAspect {
    config: MyConfig,
}

impl Aspect for MyCustomAspect {
    fn before(&self, ctx: &JoinPoint) {
        // Custom logic before function execution
        log_to_external_service(ctx.function_name, &self.config);
    }

    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
        // Custom logic after successful execution
        track_success_metric(ctx.function_name);
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        // Custom error handling
        alert_on_call(ctx, error);
    }
}
```

### Stateful Aspects

Aspects with internal state using thread-safe structures:

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct CallCounterAspect {
    counts: Arc<Mutex<HashMap<String, u64>>>,
}

impl Aspect for CallCounterAspect {
    fn before(&self, ctx: &JoinPoint) {
        let mut counts = self.counts.lock().unwrap();
        *counts.entry(ctx.function_name.to_string())
            .or_insert(0) += 1;
    }
}

impl CallCounterAspect {
    pub fn get_count(&self, function_name: &str) -> u64 {
        self.counts.lock().unwrap()
            .get(function_name)
            .copied()
            .unwrap_or(0)
    }
}
```

### Generic Aspects

Type-safe generic aspects:

```rust
pub struct ValidationAspect<T: Validator> {
    validator: T,
}

pub trait Validator: Send + Sync {
    fn validate(&self, ctx: &JoinPoint) -> Result<(), String>;
}

impl<T: Validator> Aspect for ValidationAspect<T> {
    fn before(&self, ctx: &JoinPoint) {
        if let Err(e) = self.validator.validate(ctx) {
            panic!("Validation failed: {}", e);
        }
    }
}
```

## Custom Pointcuts

### Implementing PointcutMatcher

Create custom pattern matching logic:

```rust
use aspect_core::pointcut::PointcutMatcher;

pub struct AnnotationPointcut {
    annotation_name: String,
}

impl PointcutMatcher for AnnotationPointcut {
    fn matches(&self, ctx: &JoinPoint) -> bool {
        // Custom matching logic
        // (In real implementation, would check function annotations)
        ctx.module_path.contains(&self.annotation_name)
    }
}
```

### Complex Pointcut Patterns

Combine multiple matching criteria:

```rust
pub struct ComplexPointcut {
    matchers: Vec<Box<dyn PointcutMatcher>>,
    combinator: Combinator,
}

pub enum Combinator {
    And,
    Or,
    Not,
}

impl PointcutMatcher for ComplexPointcut {
    fn matches(&self, ctx: &JoinPoint) -> bool {
        match self.combinator {
            Combinator::And => {
                self.matchers.iter().all(|m| m.matches(ctx))
            }
            Combinator::Or => {
                self.matchers.iter().any(|m| m.matches(ctx))
            }
            Combinator::Not => {
                !self.matchers[0].matches(ctx)
            }
        }
    }
}
```

## Extending Code Generation

### Custom Macro Attributes

Create domain-specific macro attributes:

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn monitored(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let block = &func.block;

    // Generate custom wrapper
    let output = quote! {
        fn #func_name() {
            let _guard = MonitorGuard::new(stringify!(#func_name));
            #block
        }
    };

    output.into()
}
```

### Custom Code Generators

Extend aspect-weaver for specialized code generation:

```rust
pub trait AspectCodeGenerator {
    fn generate_before(&self, func: &ItemFn) -> TokenStream {
        // Default implementation
        quote! {}
    }

    fn generate_after(&self, func: &ItemFn) -> TokenStream {
        quote! {}
    }

    fn generate_around(&self, func: &ItemFn) -> TokenStream {
        quote! {
            #func
        }
    }
}

pub struct OptimizingGenerator {
    inline_threshold: usize,
}

impl AspectCodeGenerator for OptimizingGenerator {
    fn generate_around(&self, func: &ItemFn) -> TokenStream {
        let should_inline = estimate_size(func) < self.inline_threshold;

        if should_inline {
            quote! {
                #[inline(always)]
                #func
            }
        } else {
            quote! {
                #[inline(never)]
                #func
            }
        }
    }
}
```

## Domain-Specific Extensions

### Database Aspects

Custom aspects for database operations:

```rust
pub struct TransactionAspect {
    isolation_level: IsolationLevel,
}

impl Aspect for TransactionAspect {
    fn around(&self, pjp: ProceedingJoinPoint)
        -> Result<Box<dyn Any>, AspectError>
    {
        let conn = get_connection()?;
        conn.begin_transaction(self.isolation_level)?;

        match pjp.proceed() {
            Ok(result) => {
                conn.commit()?;
                Ok(result)
            }
            Err(e) => {
                conn.rollback()?;
                Err(e)
            }
        }
    }
}
```

### HTTP/API Aspects

Aspects for web services:

```rust
pub struct RateLimitAspect {
    max_requests: usize,
    window: Duration,
    limiter: Arc<Mutex<RateLimiter>>,
}

impl Aspect for RateLimitAspect {
    fn before(&self, ctx: &JoinPoint) -> Result<(), AspectError> {
        let mut limiter = self.limiter.lock().unwrap();

        if !limiter.check_rate_limit(ctx.function_name) {
            return Err(AspectError::execution(
                format!("Rate limit exceeded for {}", ctx.function_name)
            ));
        }

        Ok(())
    }
}
```

### Security Aspects

Authorization and authentication:

```rust
pub struct AuthorizationAspect {
    required_roles: Vec<Role>,
}

impl Aspect for AuthorizationAspect {
    fn before(&self, ctx: &JoinPoint) -> Result<(), AspectError> {
        let current_user = get_current_user()?;

        if !current_user.has_any_role(&self.required_roles) {
            return Err(AspectError::execution(
                format!(
                    "User {} lacks required roles for {}",
                    current_user.id,
                    ctx.function_name
                )
            ));
        }

        Ok(())
    }
}
```

## Plugin Architecture

### Third-Party Aspect Crates

Structure for distributable aspects:

```rust
// my-custom-aspects/src/lib.rs
pub mod database;
pub mod monitoring;
pub mod security;

pub use database::TransactionAspect;
pub use monitoring::MetricsAspect;
pub use security::AuthAspect;

pub mod prelude {
    pub use super::*;
    pub use aspect_core::prelude::*;
}
```

Users can then:

```toml
[dependencies]
aspect-core = "0.1"
aspect-macros = "0.1"
my-custom-aspects = "1.0"
```

```rust
use my_custom_aspects::prelude::*;

#[aspect(TransactionAspect::new(IsolationLevel::ReadCommitted))]
fn update_balance(account_id: u64, amount: i64) -> Result<()> {
    // ...
}
```

## Integration Points

### Custom Backends

Integrate with external systems:

```rust
pub trait LogBackend: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}

pub struct CloudWatchBackend {
    client: CloudWatchClient,
}

impl LogBackend for CloudWatchBackend {
    fn log(&self, level: LogLevel, message: &str) {
        self.client.put_log_event(level, message);
    }
}

pub struct LoggingAspect<B: LogBackend> {
    backend: Arc<B>,
}

impl<B: LogBackend> Aspect for LoggingAspect<B> {
    fn before(&self, ctx: &JoinPoint) {
        self.backend.log(
            LogLevel::Info,
            &format!("[ENTRY] {}", ctx.function_name)
        );
    }
}
```

### Metrics Integration

Connect to monitoring systems:

```rust
pub trait MetricsReporter: Send + Sync {
    fn report_call(&self, function: &str, duration: Duration);
    fn report_error(&self, function: &str, error: &AspectError);
}

pub struct PrometheusReporter {
    registry: Registry,
}

impl MetricsReporter for PrometheusReporter {
    fn report_call(&self, function: &str, duration: Duration) {
        FUNCTION_DURATION
            .with_label_values(&[function])
            .observe(duration.as_secs_f64());
    }

    fn report_error(&self, function: &str, error: &AspectError) {
        ERROR_COUNTER
            .with_label_values(&[function])
            .inc();
    }
}
```

## Best Practices

### 1. Keep Aspects Focused

Each aspect should have a single responsibility:

```rust
// GOOD: Focused aspect
pub struct TimingAspect { ... }

// AVOID: Kitchen sink aspect
pub struct EverythingAspect {
    logger: Logger,
    timer: Timer,
    cache: Cache,
    metrics: Metrics,
}
```

### 2. Make Aspects Configurable

Use builder pattern for complex configuration:

```rust
pub struct RetryAspect {
    max_attempts: usize,
    backoff: BackoffStrategy,
    retry_on: Vec<ErrorKind>,
}

impl RetryAspect {
    pub fn builder() -> RetryAspectBuilder {
        RetryAspectBuilder::default()
    }
}

pub struct RetryAspectBuilder {
    max_attempts: usize,
    backoff: BackoffStrategy,
    retry_on: Vec<ErrorKind>,
}

impl RetryAspectBuilder {
    pub fn max_attempts(mut self, n: usize) -> Self {
        self.max_attempts = n;
        self
    }

    pub fn with_backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff = strategy;
        self
    }

    pub fn build(self) -> RetryAspect {
        RetryAspect {
            max_attempts: self.max_attempts,
            backoff: self.backoff,
            retry_on: self.retry_on,
        }
    }
}

// Usage
#[aspect(RetryAspect::builder()
    .max_attempts(3)
    .with_backoff(BackoffStrategy::Exponential)
    .build())]
fn fetch_data() -> Result<Data> { ... }
```

### 3. Document Performance Impact

Include performance characteristics in documentation:

```rust
/// Transaction aspect for database operations.
///
/// # Performance
///
/// - Overhead: ~50-100µs per transaction
/// - Memory: ~200 bytes per connection
/// - Allocations: 2 per begin/commit cycle
///
/// Use only on functions that perform database operations.
pub struct TransactionAspect { ... }
```

### 4. Provide Examples

Include usage examples in documentation:

```rust
/// # Examples
///
/// ```rust
/// use my_aspects::TransactionAspect;
///
/// #[aspect(TransactionAspect::new(IsolationLevel::ReadCommitted))]
/// fn transfer_funds(from: u64, to: u64, amount: i64) -> Result<()> {
///     // Database operations
/// }
/// ```
pub struct TransactionAspect { ... }
```

## Testing Extensions

### Unit Testing Aspects

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_aspect() {
        let aspect = MyCustomAspect::new();
        let ctx = JoinPoint {
            function_name: "test_function",
            module_path: "test::module",
            location: Location {
                file: "test.rs",
                line: 42,
            },
        };

        aspect.before(&ctx);
        // Assert expected behavior
    }
}
```

### Integration Testing

```rust
#[test]
fn test_aspect_integration() {
    #[aspect(CounterAspect::new())]
    fn test_func() -> i32 {
        42
    }

    let result = test_func();
    assert_eq!(result, 42);

    let count = COUNTER_ASPECT.get_count("test_func");
    assert_eq!(count, 1);
}
```

## See Also

- [Core Concepts](../ch04-core-concepts/) - Understanding the foundation
- [Standard Aspects](crates.md#aspect-std) - Examples to learn from
- [Implementation](../ch07-implementation/) - How code generation works
- [Case Studies](../ch08-case-studies/) - Real-world examples
