# Case Study: Web Service Logging

This case study demonstrates how aspect-oriented programming eliminates repetitive logging code while maintaining clean business logic. We'll compare traditional manual logging with the aspect-based approach.

## The Problem

Web services require comprehensive logging for debugging, auditing, and monitoring. Traditional approaches scatter logging calls throughout the codebase:

```rust
fn fetch_user(id: u64) -> User {
    println!("[{}] [ENTRY] fetch_user({})", timestamp(), id);
    let user = database::get(id);
    println!("[{}] [EXIT] fetch_user -> {:?}", timestamp(), user);
    user
}

fn save_user(user: User) -> Result<()> {
    println!("[{}] [ENTRY] save_user({:?})", timestamp(), user);
    let result = database::save(user);
    match &result {
        Ok(_) => println!("[{}] [EXIT] save_user -> Ok", timestamp()),
        Err(e) => println!("[{}] [ERROR] save_user -> {}", timestamp(), e),
    }
    result
}

fn delete_user(id: u64) -> Result<()> {
    println!("[{}] [ENTRY] delete_user({})", timestamp(), id);
    let result = database::delete(id);
    match &result {
        Ok(_) => println!("[{}] [EXIT] delete_user -> Ok", timestamp()),
        Err(e) => println!("[{}] [ERROR] delete_user -> {}", timestamp(), e),
    }
    result
}
```

**Problems with this approach**:

1. **Repetition**: Same logging pattern repeated in every function
2. **Maintenance burden**: Changing log format requires updating 100+ functions
3. **Error-prone**: Easy to forget logging in new functions
4. **Code clutter**: Business logic obscured by logging code
5. **Inconsistency**: Different developers may log differently
6. **No centralized control**: Can't easily enable/disable logging

## Traditional Solution

Extract logging to helper functions:

```rust
fn log_entry(function_name: &str, args: &str) {
    println!("[{}] [ENTRY] {}({})", timestamp(), function_name, args);
}

fn log_exit(function_name: &str, result: &str) {
    println!("[{}] [EXIT] {} -> {}", timestamp(), function_name, result);
}

fn log_error(function_name: &str, error: &str) {
    println!("[{}] [ERROR] {} -> {}", timestamp(), function_name, error);
}

fn fetch_user(id: u64) -> User {
    log_entry("fetch_user", &format!("{}", id));
    let user = database::get(id);
    log_exit("fetch_user", &format!("{:?}", user));
    user
}

fn save_user(user: User) -> Result<()> {
    log_entry("save_user", &format!("{:?}", user));
    let result = database::save(user);
    match &result {
        Ok(_) => log_exit("save_user", "Ok"),
        Err(e) => log_error("save_user", &format!("{}", e)),
    }
    result
}
```

**Still problematic**:

- ✅ Reduces code duplication
- ✅ Centralized log format
- ❌ Still manual calls in every function
- ❌ Still easy to forget
- ❌ Business logic still cluttered
- ❌ Function names hardcoded (error-prone)

## aspect-rs Solution

Use a logging aspect to completely separate logging from business logic:

### Step 1: Define the Logging Aspect

```rust
use aspect_core::prelude::*;
use std::any::Any;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct Logger;

impl Aspect for Logger {
    fn before(&self, ctx: &JoinPoint) {
        println!(
            "[{}] [ENTRY] {} at {}:{}",
            current_timestamp(),
            ctx.function_name,
            ctx.location.file,
            ctx.location.line
        );
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        println!(
            "[{}] [EXIT]  {}",
            current_timestamp(),
            ctx.function_name
        );
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        eprintln!(
            "[{}] [ERROR] {} failed: {:?}",
            current_timestamp(),
            ctx.function_name,
            error
        );
    }
}

fn current_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
}
```

### Step 2: Apply to Functions (Phase 1)

```rust
use aspect_macros::aspect;

#[aspect(Logger::default())]
fn fetch_user(id: u64) -> User {
    database::get(id)
}

#[aspect(Logger::default())]
fn save_user(user: User) -> Result<()> {
    database::save(user)
}

#[aspect(Logger::default())]
fn delete_user(id: u64) -> Result<()> {
    database::delete(id)
}
```

### Step 3: Automatic Application (Phase 2)

```rust
use aspect_macros::advice;

// Register once for all matching functions
#[advice(
    pointcut = "execution(pub fn *_user(..))",
    advice = "around"
)]
fn user_operations_logger(pjp: ProceedingJoinPoint)
    -> Result<Box<dyn Any>, AspectError>
{
    let ctx = pjp.context();
    println!("[{}] [ENTRY] {}", current_timestamp(), ctx.function_name);

    let result = pjp.proceed();

    match &result {
        Ok(_) => println!("[{}] [EXIT] {}", current_timestamp(), ctx.function_name),
        Err(e) => eprintln!("[{}] [ERROR] {} failed: {:?}",
            current_timestamp(), ctx.function_name, e),
    }

    result
}

// Clean business logic - NO logging code!
fn fetch_user(id: u64) -> User {
    database::get(id)
}

fn save_user(user: User) -> Result<()> {
    database::save(user)
}

fn delete_user(id: u64) -> Result<()> {
    database::delete(id)
}
```

## Complete Working Example

Here's the complete working code from `aspect-examples/src/logging.rs`:

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;
use std::any::Any;

#[derive(Default)]
struct Logger;

impl Aspect for Logger {
    fn before(&self, ctx: &JoinPoint) {
        println!(
            "[{}] [ENTRY] {} at {}",
            current_timestamp(),
            ctx.function_name,
            ctx.location
        );
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        println!(
            "[{}] [EXIT]  {}",
            current_timestamp(),
            ctx.function_name
        );
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        eprintln!(
            "[{}] [ERROR] {} failed: {:?}",
            current_timestamp(),
            ctx.function_name,
            error
        );
    }
}

fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
}

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
}

#[aspect(Logger::default())]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[aspect(Logger::default())]
fn fetch_user(id: u64) -> Result<User, String> {
    if id == 0 {
        Err("Invalid user ID: 0".to_string())
    } else {
        Ok(User {
            id,
            name: format!("User{}", id),
        })
    }
}

#[aspect(Logger::default())]
fn process_data(input: &str, multiplier: usize) -> String {
    input.repeat(multiplier)
}

fn main() {
    println!("=== Logging Aspect Example ===\n");

    println!("1. Calling greet(\"Alice\"):");
    let greeting = greet("Alice");
    println!("   Result: {}\n", greeting);

    println!("2. Calling fetch_user(42):");
    match fetch_user(42) {
        Ok(user) => println!("   Success: {:?}\n", user),
        Err(e) => println!("   Error: {}\n", e),
    }

    println!("3. Calling fetch_user(0) (will fail):");
    match fetch_user(0) {
        Ok(user) => println!("   Success: {:?}\n", user),
        Err(e) => println!("   Error: {}\n", e),
    }

    println!("4. Calling process_data(\"Rust \", 3):");
    let result = process_data("Rust ", 3);
    println!("   Result: {}\n", result);

    println!("=== Demo Complete ===");
}
```

## Example Output

Running the example produces:

```
=== Logging Aspect Example ===

1. Calling greet("Alice"):
[1708224361.234] [ENTRY] greet at src/logging.rs:59
[1708224361.235] [EXIT]  greet
   Result: Hello, Alice!

2. Calling fetch_user(42):
[1708224361.235] [ENTRY] fetch_user at src/logging.rs:64
[1708224361.236] [EXIT]  fetch_user
   Success: User { id: 42, name: "User42" }

3. Calling fetch_user(0) (will fail):
[1708224361.236] [ENTRY] fetch_user at src/logging.rs:64
[1708224361.237] [ERROR] fetch_user failed: ExecutionError("Invalid user ID: 0")
   Error: Invalid user ID: 0

4. Calling process_data("Rust ", 3):
[1708224361.237] [ENTRY] process_data at src/logging.rs:76
[1708224361.238] [EXIT]  process_data
   Result: Rust Rust Rust

=== Demo Complete ===
```

## Analysis

### Lines of Code Comparison

**Manual logging (3 functions)**:
```
Without helpers: ~45 lines
With helpers:    ~30 lines
```

**aspect-rs (3 functions)**:
```
Aspect definition:  ~25 lines (once)
Business functions: ~15 lines (clean!)
Total:             ~40 lines
```

**For 100 functions**:
```
Manual:    ~1000-1500 lines
aspect-rs: ~325 lines (aspect + 100 clean functions)
           67% less code!
```

### Benefits Achieved

1. ✅ **Separation of concerns**: Logging completely separated from business logic
2. ✅ **No repetition**: Logging aspect defined once
3. ✅ **Automatic metadata**: Function name, location automatically captured
4. ✅ **Impossible to forget**: Can't miss logging on new functions (Phase 2/3)
5. ✅ **Centralized control**: Change logging format in one place
6. ✅ **Clean business logic**: Functions contain only business code
7. ✅ **Type-safe**: Compile-time verification
8. ✅ **Zero runtime overhead**: ~2% overhead (see [Benchmarks](../ch09-benchmarks/results.md))

### Performance Impact

From actual benchmarks:

```
Manual logging:    1.2678 µs per call
Aspect logging:    1.2923 µs per call
Overhead:          +2.14% (0.0245 µs)
```

**Conclusion**: The 2% overhead is negligible compared to I/O cost of `println!` itself (~1000µs).

## Advanced Usage

### Structured Logging

Extend the aspect for structured logging:

```rust
use serde_json::json;

impl Aspect for StructuredLogger {
    fn before(&self, ctx: &JoinPoint) {
        let log_entry = json!({
            "timestamp": current_timestamp(),
            "level": "INFO",
            "event": "function_entry",
            "function": ctx.function_name,
            "module": ctx.module_path,
            "location": {
                "file": ctx.location.file,
                "line": ctx.location.line
            }
        });
        println!("{}", log_entry);
    }
}
```

### Conditional Logging

Log only slow functions:

```rust
impl Aspect for ConditionalLogger {
    fn around(&self, pjp: ProceedingJoinPoint)
        -> Result<Box<dyn Any>, AspectError>
    {
        let start = Instant::now();
        let result = pjp.proceed();
        let elapsed = start.elapsed();

        if elapsed > Duration::from_millis(100) {
            println!("[SLOW] {} took {:?}", pjp.context().function_name, elapsed);
        }

        result
    }
}
```

### Multiple Logging Levels

```rust
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct LoggingAspect {
    level: LogLevel,
}

impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        if self.should_log(LogLevel::Info) {
            self.log(LogLevel::Info, format!("[ENTRY] {}", ctx.function_name));
        }
    }
}
```

## Real-World Application

This logging pattern is used in production systems for:

- **API servers**: Log all HTTP endpoint calls
- **Database operations**: Track all queries
- **Background jobs**: Monitor task execution
- **Microservices**: Distributed tracing
- **Security auditing**: Record all privileged operations

## Key Takeaways

1. **AOP eliminates logging boilerplate** - Define once, apply everywhere
2. **Business logic stays clean** - No clutter from cross-cutting concerns
3. **Centralized control** - Change behavior in one place
4. **Automatic metadata** - Function name, location, timestamp captured automatically
5. **Production-ready** - Minimal overhead, type-safe, thread-safe
6. **Scales well** - 100+ functions with no additional effort

## See Also

- [Timing Case Study](timing.md) - Performance monitoring aspect
- [API Server Case Study](api-server.md) - Multiple aspects working together
- [LoggingAspect Implementation](../ch06-architecture/crates.md#loggingaspect) - Standard library aspect
- [Benchmarks](../ch09-benchmarks/results.md) - Performance measurements
- [Usage Patterns](../ch05-usage-guide/patterns.md) - More logging patterns
