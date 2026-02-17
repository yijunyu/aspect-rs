# Basic Patterns

Common aspect patterns for everyday use. These patterns are simple to implement and cover the most frequent use cases.

## Logging Pattern

The most common aspect - automatically log function entry and exit.

### Simple Logging

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;

#[derive(Default)]
struct SimpleLogger;

impl Aspect for SimpleLogger {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ Entering: {}", ctx.function_name);
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        println!("← Exiting: {}", ctx.function_name);
    }
}

#[aspect(SimpleLogger)]
fn process_data(data: &str) -> String {
    data.to_uppercase()
}

fn main() {
    let result = process_data("hello");
    println!("Result: {}", result);
}
```

**Output:**
```
→ Entering: process_data
← Exiting: process_data
Result: HELLO
```

### Logging with Timestamps

```rust
use chrono::Utc;

struct TimestampLogger;

impl Aspect for TimestampLogger {
    fn before(&self, ctx: &JoinPoint) {
        println!("[{}] → {}", Utc::now().format("%H:%M:%S%.3f"), ctx.function_name);
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        println!("[{}] ← {}", Utc::now().format("%H:%M:%S%.3f"), ctx.function_name);
    }
}
```

**Output:**
```
[14:32:15.123] → fetch_user
[14:32:15.456] ← fetch_user
```

### Structured Logging

```rust
use log::{info, Level};

struct StructuredLogger {
    level: Level,
}

impl Aspect for StructuredLogger {
    fn before(&self, ctx: &JoinPoint) {
        info!(
            target: "aspect",
            "function = {}, module = {}, file = {}:{}",
            ctx.function_name,
            ctx.module_path,
            ctx.file,
            ctx.line
        );
    }
}

#[aspect(StructuredLogger { level: Level::Info })]
fn important_operation() {
    // Business logic
}
```

## Timing Pattern

Measure execution time of functions automatically.

### Basic Timing

```rust
use std::time::Instant;

struct Timer;

impl Aspect for Timer {
    fn around(&self, mut pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let start = Instant::now();
        let result = pjp.proceed()?;
        let elapsed = start.elapsed();

        println!("{} took {:?}", pjp.function_name, elapsed);

        Ok(result)
    }
}

#[aspect(Timer)]
fn expensive_operation(n: u64) -> u64 {
    // Simulate expensive work
    std::thread::sleep(std::time::Duration::from_millis(100));
    n * 2
}
```

**Output:**
```
expensive_operation took 100.234ms
```

### Timing with Threshold Warnings

```rust
use std::time::Duration;

struct ThresholdTimer {
    threshold: Duration,
}

impl Aspect for ThresholdTimer {
    fn around(&self, mut pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let start = Instant::now();
        let result = pjp.proceed()?;
        let elapsed = start.elapsed();

        if elapsed > self.threshold {
            eprintln!(
                "⚠️  SLOW: {} took {:?} (threshold: {:?})",
                pjp.function_name, elapsed, self.threshold
            );
        } else {
            println!("✓ {} took {:?}", pjp.function_name, elapsed);
        }

        Ok(result)
    }
}

#[aspect(ThresholdTimer {
    threshold: Duration::from_millis(50)
})]
fn database_query(sql: &str) -> Vec<Row> {
    // Execute query
}
```

## Call Counting Pattern

Track how many times functions are called.

### Simple Counter

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct CallCounter {
    count: AtomicU64,
}

impl CallCounter {
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
        }
    }

    fn get_count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

impl Aspect for CallCounter {
    fn before(&self, ctx: &JoinPoint) {
        let count = self.count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("{} called {} times", ctx.function_name, count);
    }
}

static COUNTER: CallCounter = CallCounter {
    count: AtomicU64::new(0),
};

#[aspect(&COUNTER)]
fn api_endpoint() {
    // Handle request
}
```

### Per-Function Counters

```rust
use std::collections::HashMap;
use std::sync::Mutex;

struct GlobalCounter {
    counts: Mutex<HashMap<String, u64>>,
}

impl GlobalCounter {
    fn new() -> Self {
        Self {
            counts: Mutex::new(HashMap::new()),
        }
    }
}

impl Aspect for GlobalCounter {
    fn before(&self, ctx: &JoinPoint) {
        let mut counts = self.counts.lock().unwrap();
        let count = counts.entry(ctx.function_name.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        println!("{} called {} times", ctx.function_name, count);
    }
}
```

## Tracing Pattern

Trace function execution with indentation for call hierarchy.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

struct Tracer {
    depth: AtomicUsize,
}

impl Tracer {
    fn new() -> Self {
        Self {
            depth: AtomicUsize::new(0),
        }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.depth.load(Ordering::Relaxed))
    }
}

impl Aspect for Tracer {
    fn before(&self, ctx: &JoinPoint) {
        let depth = self.depth.fetch_add(1, Ordering::Relaxed);
        println!("{}→ {}", "  ".repeat(depth), ctx.function_name);
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        let depth = self.depth.fetch_sub(1, Ordering::Relaxed) - 1;
        println!("{}← {}", "  ".repeat(depth), ctx.function_name);
    }
}

#[aspect(Tracer::new())]
fn outer() {
    inner();
}

#[aspect(Tracer::new())]
fn inner() {
    leaf();
}

#[aspect(Tracer::new())]
fn leaf() {
    println!("    Executing leaf");
}
```

**Output:**
```
→ outer
  → inner
    → leaf
      Executing leaf
    ← leaf
  ← inner
← outer
```

## Key Takeaways

Basic patterns are:
- ✅ **Simple to implement** - Just a few lines of code
- ✅ **Reusable** - Define once, apply everywhere
- ✅ **Non-invasive** - Business logic stays clean
- ✅ **Composable** - Can be combined with other aspects

See [Production Patterns](production.md) for more advanced use cases.
