# AOP Terminology

Understanding AOP requires learning a few key terms. This section defines the core vocabulary used throughout aspect-rs.

## Core Concepts

### 1. Aspect

**Definition**: A module that encapsulates a crosscutting concern.

**In aspect-rs**: Any type implementing the `Aspect` trait.

```rust
use aspect_core::Aspect;

#[derive(Default)]
pub struct LoggingAspect;

impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ {}", ctx.function_name);
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        println!("← {}", ctx.function_name);
    }
}
```

**Examples**: LoggingAspect, TimingAspect, CachingAspect, AuthorizationAspect

**Analogy**: An aspect is like a **middleware** that wraps function execution.

---

### 2. Join Point

**Definition**: A point in program execution where an aspect can be applied.

**In aspect-rs**: Currently, **function calls** are join points. Future versions may support field access, method calls, etc.

**Context**: The `JoinPoint` struct provides metadata about the execution point:

```rust
pub struct JoinPoint {
    pub function_name: &'static str,  // e.g., "fetch_user"
    pub module_path: &'static str,     // e.g., "myapp::user::repository"
    pub file: &'static str,            // e.g., "src/user/repository.rs"
    pub line: u32,                     // e.g., 42
}
```

**Examples**:
- Entering `fetch_user()` function
- Returning from `process_payment()` function
- Throwing error in `validate_email()` function

**Analogy**: A join point is like a **breakpoint** in a debugger where your aspect can inject code.

---

### 3. Pointcut

**Definition**: A predicate that selects which join points an aspect should apply to.

**In aspect-rs**:
- **Phase 1-2**: Per-function annotation `#[aspect(...)]`
- **Phase 3**: Pattern-based expressions (like AspectJ)

**Examples**:

```rust
// Phase 2 - Explicit annotation (current)
#[aspect(LoggingAspect::new())]
fn fetch_user(id: u64) -> User { ... }

// Phase 3 - Pattern matching (future)
#[advice(
    pointcut = "execution(pub fn *(..)) && within(crate::api)",
    //         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //         This is a pointcut expression
    advice = "before"
)]
static LOGGER: LoggingAspect = LoggingAspect::new();
```

**Pointcut expressions** (Phase 3):
- `execution(pub fn *(..))` - All public functions
- `within(crate::api)` - Inside the `api` module
- `@annotation(#[cached])` - Functions with `#[cached]` attribute

**Analogy**: A pointcut is like a **CSS selector** that matches DOM elements, but for code.

---

### 4. Advice

**Definition**: The action taken by an aspect at a join point.

**In aspect-rs**: Four advice types:

#### Before Advice
Runs **before** the function executes.

```rust
fn before(&self, ctx: &JoinPoint) {
    println!("About to call {}", ctx.function_name);
}
```

**Use cases**: Logging entry, validation, authorization checks

#### After Advice
Runs **after** successful execution.

```rust
fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
    println!("Successfully completed {}", ctx.function_name);
}
```

**Use cases**: Logging exit, metrics collection, cleanup

#### After Throwing Advice
Runs when function **panics or returns Err**.

```rust
fn after_throwing(&self, ctx: &JoinPoint, error: &dyn Any) {
    eprintln!("Error in {}: {:?}", ctx.function_name, error);
}
```

**Use cases**: Error logging, alerting, circuit breaker logic

#### Around Advice
**Wraps** the entire function execution.

```rust
fn around(&self, ctx: &mut ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    println!("Before execution");

    let result = ctx.proceed()?;  // Execute the function

    println!("After execution");
    Ok(result)
}
```

**Use cases**: Timing, caching (skip execution if cached), transactions, retry logic

**Analogy**: Advice is like **event handlers** (onClick, onSubmit), but for function execution.

---

### 5. Weaving

**Definition**: The process of inserting aspect code into join points.

**Three types of weaving**:

#### Compile-Time Weaving (aspect-rs)
Code is generated at compile time via procedural macros.

```rust
// Source code
#[aspect(LoggingAspect::new())]
fn my_function() { println!("Hello"); }

// Generated code (simplified)
fn my_function() {
    LoggingAspect::new().before(&ctx);
    println!("Hello");
    LoggingAspect::new().after(&ctx, &());
}
```

**Pros**: Zero runtime overhead, type-safe
**Cons**: Must recompile to change aspects

#### Load-Time Weaving (AspectJ)
Bytecode is modified when classes are loaded into JVM.

**Pros**: Can weave into third-party libraries
**Cons**: Requires AspectJ agent, runtime overhead

#### Runtime Weaving (Spring AOP)
Uses dynamic proxies at runtime.

**Pros**: Very flexible
**Cons**: Significant overhead, only works with interfaces

**aspect-rs uses compile-time weaving** for maximum performance.

**Analogy**: Weaving is like a **compiler pass** that transforms your code.

---

### 6. ProceedingJoinPoint

**Definition**: A special join point for `around` advice that can control function execution.

**In aspect-rs**:

```rust
pub struct ProceedingJoinPoint<'a> {
    function_name: &'static str,
    proceed_fn: Box<dyn FnOnce() -> Box<dyn Any> + 'a>,
}

impl<'a> ProceedingJoinPoint<'a> {
    pub fn proceed(self) -> Result<Box<dyn Any>, AspectError> {
        (self.proceed_fn)()  // Execute original function
    }
}
```

**Key capability**: Can choose **whether** and **when** to execute the function.

```rust
impl Aspect for CachingAspect {
    fn around(&self, ctx: &mut ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        if let Some(cached) = self.cache.get(ctx.function_name) {
            return Ok(cached);  // Skip execution!
        }

        let result = ctx.proceed()?;  // Execute if not cached
        self.cache.insert(ctx.function_name, result.clone());
        Ok(result)
    }
}
```

**Analogy**: `ProceedingJoinPoint` is like a **callback** you can choose to invoke.

---

## Summary Table

| Term | Definition | aspect-rs Equivalent |
|------|------------|---------------------|
| **Aspect** | Crosscutting concern module | `impl Aspect for T` |
| **Join Point** | Execution point | Function call |
| **Pointcut** | Selection predicate | `#[aspect(...)]` or pointcut expression |
| **Advice** | Action at join point | `before`, `after`, `after_throwing`, `around` |
| **Weaving** | Code insertion process | Procedural macro expansion |
| **Target** | Object being advised | The function with `#[aspect(...)]` |

## Visual Model

```
┌──────────────────────────────────────────────────┐
│              Aspect (LoggingAspect)              │
│  ┌────────────────────────────────────────────┐  │
│  │ before() → Log "entering function"         │  │
│  │ after() → Log "exiting function"           │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
                        ↓ Weaving (compile-time)
┌──────────────────────────────────────────────────┐
│          Join Point (function execution)         │
│  ┌────────────────────────────────────────────┐  │
│  │ fn fetch_user(id: u64) -> User {           │  │
│  │     database::query_user(id)               │  │
│  │ }                                          │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
                        ↓ Pointcut matches?
                      ✅ Yes
                        ↓
         Generated code with advice woven
```

## Common Misconceptions

### ❌ "Aspects run at runtime"
**Truth**: In aspect-rs, aspects are woven at **compile time**. The generated code has zero aspect framework overhead.

### ❌ "Pointcuts use regex"
**Truth**: Pointcuts use a **structured expression language**, not regex. They understand Rust syntax semantically.

### ❌ "Aspects violate encapsulation"
**Truth**: Aspects are **declared explicitly** with `#[aspect(...)]`. They don't secretly modify code.

### ❌ "AOP is only for logging"
**Truth**: AOP is powerful for **any crosscutting concern**: caching, security, transactions, metrics, etc.

## Next Steps

Now that you understand AOP terminology, let's explore [What aspect-rs Can Do](capabilities.md) with concrete examples.
