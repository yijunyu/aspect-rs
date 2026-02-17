# Core Design Principles

aspect-rs is built on five foundational principles that guide every design decision. These principles ensure the framework is both powerful and practical for production use.

## 1. Zero Runtime Overhead

**Goal**: Aspects should have near-zero performance impact after compiler optimizations.

### Implementation

Aspects are woven at **compile-time** using procedural macros and (in Phase 3) MIR-level transformations. This means:

- No runtime registration
- No dynamic dispatch (when possible)
- No reflection overhead
- Compiler can inline and optimize

### Example

Consider a simple logging aspect:

```rust
#[aspect(Logger::default())]
fn calculate(x: i32) -> i32 {
    x * 2
}
```

The macro expands to:

```rust
#[inline(always)]
fn calculate(x: i32) -> i32 {
    const CTX: JoinPoint = JoinPoint {
        function_name: "calculate",
        module_path: module_path!(),
        location: Location {
            file: file!(),
            line: line!(),
        },
    };

    Logger::default().before(&CTX);
    let __result = { x * 2 };
    Logger::default().after(&CTX, &__result);
    __result
}
```

With optimizations enabled:
- `#[inline(always)]` causes the wrapper to be inlined
- `const CTX` is stored in `.rodata` (no allocation)
- Empty `before`/`after` methods are eliminated by dead code elimination
- Final assembly is identical to hand-written code

### Benchmark Results

From `BENCHMARKS.md`:

| Aspect Type | Overhead | Target | Status |
|-------------|----------|--------|--------|
| No-op aspect | 0ns | 0ns | ✅ |
| Simple logging | ~2% | <5% | ✅ |
| Complex aspect | ~manual | ~manual | ✅ |

**Conclusion**: Aspects add negligible overhead in real-world use.

### Optimization Techniques

1. **Inline wrappers**: Mark all generated code as `#[inline(always)]`
2. **Const evaluation**: Use `const` for static JoinPoint data
3. **Dead code elimination**: Remove empty aspect methods at compile-time
4. **Static instances**: Reuse aspect instances via `static`
5. **Zero allocation**: Stack-only execution where possible

## 2. Type Safety

**Goal**: Leverage Rust's type system to catch errors at compile-time.

### Implementation

Every aspect interaction is type-checked:

#### Aspect Trait

```rust
pub trait Aspect: Send + Sync {
    fn before(&self, ctx: &JoinPoint) {}
    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {}
    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {}
}
```

**Type guarantees**:
- `ctx` is always a valid `JoinPoint`
- `result` is type-erased but safe
- `error` is always an `AspectError`
- Return types are checked at compile-time

#### Function Signature Preservation

The `#[aspect]` macro preserves the exact function signature:

```rust
#[aspect(Logger::default())]
fn fetch_user(id: u64) -> Result<User, DatabaseError> {
    // ...
}
```

The generated code maintains:
- Same parameter types
- Same return type
- Same error types
- Same visibility
- Same generics

**This prevents**:
- Accidental type coercion
- Lost error information
- Broken API contracts

### Type-Safe Context Access

JoinPoint provides compile-time known metadata:

```rust
pub struct JoinPoint {
    pub function_name: &'static str,  // Known at compile-time
    pub module_path: &'static str,     // Known at compile-time
    pub location: Location,            // Known at compile-time
}
```

All fields are `&'static str` - no runtime allocation or lifetime issues.

### Generic Aspects

Aspects can be generic while maintaining type safety:

```rust
pub struct CachingAspect<K: Hash + Eq, V: Clone> {
    cache: Arc<Mutex<HashMap<K, V>>>,
}

impl<K: Hash + Eq, V: Clone> Aspect for CachingAspect<K, V> {
    // Type-safe caching logic
}
```

The compiler ensures:
- `K` is hashable and comparable
- `V` is cloneable
- Cache operations are type-safe

### Compile-Time Errors

Type errors are caught early:

```rust
// ERROR: Logger is not an Aspect
#[aspect(String::new())]
fn my_function() { }

// ERROR: Wrong signature
impl Aspect for MyAspect {
    fn before(&self, ctx: String) { }  // Should be &JoinPoint
}
```

## 3. Thread Safety

**Goal**: All aspects must be safe to use across threads.

### Implementation

The `Aspect` trait requires `Send + Sync`:

```rust
pub trait Aspect: Send + Sync {
    // ...
}
```

**This guarantees**:
- Aspects can be sent between threads (`Send`)
- Aspects can be shared between threads (`Sync`)
- No data races possible
- Safe for concurrent execution

### Thread-Safe Aspects

Example timing aspect with thread-safe state:

```rust
pub struct TimingAspect {
    // Arc + Mutex ensures thread-safety
    start_times: Arc<Mutex<Vec<Instant>>>,
}

impl Aspect for TimingAspect {
    fn before(&self, _ctx: &JoinPoint) {
        // Lock is held only briefly
        self.start_times.lock().unwrap().push(Instant::now());
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        if let Some(start) = self.start_times.lock().unwrap().pop() {
            let elapsed = start.elapsed();
            println!("{} took {:?}", ctx.function_name, elapsed);
        }
    }
}
```

The compiler enforces:
- `Arc` for shared ownership
- `Mutex` for interior mutability
- No data races

### Concurrent Execution

Multiple threads can execute aspected functions simultaneously:

```rust
#[aspect(Logger::default())]
fn process(id: u64) -> Result<()> {
    // ...
}

// Safe: Logger implements Send + Sync
std::thread::scope(|s| {
    for i in 0..10 {
        s.spawn(|| process(i));
    }
});
```

### Lock-Free Aspects

For maximum performance, use atomic operations:

```rust
pub struct MetricsAspect {
    call_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
}

impl Aspect for MetricsAspect {
    fn before(&self, _ctx: &JoinPoint) {
        self.call_count.fetch_add(1, Ordering::Relaxed);
    }

    fn after_error(&self, _ctx: &JoinPoint, _error: &AspectError) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
}
```

No locks, no contention, perfect for high-concurrency scenarios.

## 4. Composability

**Goal**: Multiple aspects should compose cleanly without interference.

### Implementation

Aspects can be stacked on a single function:

```rust
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(AuthorizationAspect::new(Role::Admin))]
fn delete_user(id: u64) -> Result<()> {
    // ...
}
```

**Execution order** (outer to inner):
1. AuthorizationAspect::before
2. TimingAspect::before
3. LoggingAspect::before
4. **Function executes**
5. LoggingAspect::after
6. TimingAspect::after
7. AuthorizationAspect::after

### Explicit Ordering

Use the `order` parameter in Phase 2:

```rust
#[advice(
    pointcut = "execution(pub fn *(..))",
    order = 10  // Higher = outer
)]
fn security_check(pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    // Runs first
    pjp.proceed()
}

#[advice(
    pointcut = "execution(pub fn *(..))",
    order = 5  // Lower = inner
)]
fn logging(pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    // Runs second
    pjp.proceed()
}
```

### Aspect Independence

Aspects should not depend on each other's state:

```rust
// GOOD: Independent aspects
#[aspect(Logger::new())]
#[aspect(Timer::new())]
fn my_function() { }

// AVOID: Aspects that depend on execution order
// (Use explicit ordering instead)
```

### Composition Patterns

#### Chain of Responsibility

```rust
#[aspect(RateLimitAspect::new(100))]
#[aspect(AuthenticationAspect::new())]
#[aspect(AuthorizationAspect::new(Role::User))]
#[aspect(ValidationAspect::new())]
fn handle_request(req: Request) -> Response {
    // Each aspect can short-circuit by returning an error
}
```

#### Decorator Pattern

```rust
#[aspect(CachingAspect::new(Duration::from_secs(60)))]
#[aspect(TimingAspect::new())]
fn expensive_computation(x: i32) -> i32 {
    // Caching wraps timing wraps the function
}
```

## 5. Extensibility

**Goal**: Easy to create custom aspects for domain-specific concerns.

### Implementation

Creating a custom aspect requires implementing a single trait:

```rust
use aspect_core::prelude::*;
use std::any::Any;

struct MyCustomAspect {
    config: MyConfig,
}

impl Aspect for MyCustomAspect {
    fn before(&self, ctx: &JoinPoint) {
        // Custom pre-execution logic
        println!("[CUSTOM] Entering {}", ctx.function_name);
    }

    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
        // Custom post-execution logic
        println!("[CUSTOM] Exiting {}", ctx.function_name);
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        // Custom error handling
        eprintln!("[CUSTOM] Error in {}: {:?}", ctx.function_name, error);
    }
}
```

That's it! No registration, no boilerplate, just implement the trait.

### Extension Points

#### 1. Custom Aspects

Create domain-specific aspects:

```rust
// Database connection pooling
struct ConnectionPoolAspect {
    pool: Arc<Pool<PostgresConnectionManager>>,
}

// Distributed tracing
struct TracingAspect {
    tracer: Arc<dyn Tracer>,
}

// Feature flags
struct FeatureFlagAspect {
    flag: String,
}
```

#### 2. Custom Pointcuts (Phase 2+)

Extend pointcut matching:

```rust
// Custom pattern matching
pub enum CustomPattern {
    HasAttribute(String),
    ReturnsType(String),
    TakesParameter(String),
}

impl PointcutMatcher for CustomPattern {
    fn matches(&self, ctx: &JoinPoint) -> bool {
        // Custom matching logic
    }
}
```

#### 3. Custom Code Generation (Phase 3)

Extend the weaver for special cases:

```rust
pub trait AspectCodeGenerator {
    fn generate_before(&self, func: &ItemFn) -> TokenStream;
    fn generate_after(&self, func: &ItemFn) -> TokenStream;
    fn generate_around(&self, func: &ItemFn) -> TokenStream;
}
```

### Standard Library as Examples

The `aspect-std` crate provides 8 standard aspects that serve as examples:

1. **LoggingAspect** - Shows structured logging
2. **TimingAspect** - Demonstrates state management
3. **CachingAspect** - Generic aspects with caching
4. **MetricsAspect** - Lock-free concurrent aspects
5. **RateLimitAspect** - Complex logic with state
6. **RetryAspect** - Control flow modification
7. **TransactionAspect** - Resource management
8. **AuthorizationAspect** - Security concerns

Each can be studied and adapted for custom needs.

### Community Aspects

The framework is designed for easy third-party aspects:

```toml
[dependencies]
aspect-core = "0.1"
aspect-macros = "0.1"
my-custom-aspects = "1.0"  # Third-party crate
```

```rust
use my_custom_aspects::SpecializedAspect;

#[aspect(SpecializedAspect::new())]
fn my_function() { }
```

## Principle Interactions

These principles work together:

- **Zero Overhead + Type Safety**: Compile-time guarantees with no runtime cost
- **Thread Safety + Composability**: Safe concurrent aspect composition
- **Type Safety + Extensibility**: Easy to create type-safe custom aspects
- **Zero Overhead + Composability**: Multiple aspects with minimal impact
- **All principles**: Production-ready AOP in Rust

## Design Tradeoffs

### Choices Made

1. **Compile-time over runtime**: Sacrifices dynamic aspect loading for performance
2. **Proc macros over reflection**: Requires macro system but enables zero-cost
3. **Static typing over flexibility**: Less flexible than runtime AOP but safer
4. **Explicit over implicit**: Requires annotations (Phase 1-2) but clearer

### Phase 3 Improvements

Phase 3 addresses some limitations:

- **Annotation-free**: Automatic weaving via pointcuts
- **More powerful**: MIR-level transformations
- **Still zero-cost**: Compile-time weaving preserved

## Validation

These principles are validated through:

1. **Benchmarks**: Prove zero-overhead claim (see [Benchmarks](../ch09-benchmarks/results.md))
2. **Type system**: Compiler enforces type safety
3. **Tests**: 194 tests across all crates
4. **Examples**: 10+ real-world examples
5. **Production use**: Successfully deployed

## See Also

- [Crate Organization](crates.md) - How principles map to crates
- [Interactions](interactions.md) - How components implement principles
- [Benchmarks](../ch09-benchmarks/methodology.md) - Performance validation
- [Examples](../ch08-case-studies/) - Principles in practice
