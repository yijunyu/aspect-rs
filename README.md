# Aspect-RS: Aspect-Oriented Programming for Rust

[![Crates.io](https://img.shields.io/crates/v/aspect-core.svg)](https://crates.io/crates/aspect-core)
[![Documentation](https://docs.rs/aspect-core/badge.svg)](https://docs.rs/aspect-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A comprehensive Aspect-Oriented Programming (AOP) framework for Rust, bringing the power of cross-cutting concerns modularization to the Rust ecosystem.

📖 **[Read the Book](https://yijunyu.github.io/aspect-rs/)** - Comprehensive guide with 11 chapters covering motivation, usage, architecture, and advanced topics.

## What is AOP?

Aspect-Oriented Programming helps you modularize cross-cutting concerns - functionality that cuts across multiple parts of your application:

- **Logging**: Track function entry/exit across your codebase
- **Performance Monitoring**: Measure execution time automatically
- **Caching**: Add memoization without cluttering business logic
- **Security**: Enforce authorization declaratively
- **Transactions**: Manage database transactions transparently
- **Retry Logic**: Add resilience patterns without boilerplate
- **Validation**: Enforce constraints before function execution

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aspect-core = "0.1"
aspect-macros = "0.1"
aspect-std = "0.1"  # Optional: production-ready aspects
```

**Minimum Supported Rust Version (MSRV):** 1.70+

## Production-Ready Aspects (`aspect-std`)

The `aspect-std` crate provides 8 battle-tested aspects ready for production use:

| Aspect | Purpose | Example Use Case |
|--------|---------|------------------|
| **LoggingAspect** | Entry/exit logging | Trace function calls with timestamps |
| **TimingAspect** | Performance monitoring | Measure execution time, warn on slow functions |
| **CachingAspect** | Memoization | Cache expensive computations |
| **MetricsAspect** | Metrics collection | Track call counts, latency distributions |
| **RateLimitAspect** | Rate limiting | Prevent API abuse with token bucket |
| **CircuitBreakerAspect** | Fault tolerance | Handle service failures gracefully |
| **AuthorizationAspect** | Access control | Enforce role-based permissions (RBAC) |
| **ValidationAspect** | Input validation | Validate function arguments with custom rules |

### Quick Examples

```rust
use aspect_std::*;

// Logging with timestamps
#[aspect(LoggingAspect::new())]
fn process_order(order: Order) -> Result<Receipt, Error> {
    // Business logic
}

// Rate limiting (100 calls per minute)
#[aspect(RateLimitAspect::new(100, Duration::from_secs(60)))]
pub fn api_endpoint(request: Request) -> Response {
    handle_request(request)
}

// Authorization with role-based access control
#[aspect(AuthorizationAspect::require_role("admin", get_current_user_roles))]
pub fn delete_user(user_id: u64) -> Result<(), Error> {
    database::delete_user(user_id)
}

// Circuit breaker (opens after 5 failures, retries after 30s)
#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
pub fn call_external_service(url: &str) -> Result<Response, Error> {
    reqwest::blocking::get(url)?.json()
}
```

## Quick Start

Define an aspect and apply it to your functions:

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;

// Define an aspect
#[derive(Default)]
struct Logger;

impl Aspect for Logger {
    fn before(&self, ctx: &JoinPoint) {
        println!("[ENTRY] {}", ctx.function_name);
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn std::any::Any) {
        println!("[EXIT] {}", ctx.function_name);
    }
}

// Apply it to your functions
#[aspect(Logger)]
fn fetch_user(id: u64) -> Result<User, Error> {
    // Your business logic here
    database::query_user(id)
}
```

## Features

- **✅ Procedural Macros**: Works with stable Rust, no custom toolchain needed
- **✅ Multiple Advice Types**: `before`, `after`, `after_error`, `around`
- **✅ Aspect Composition**: Stack multiple aspects on a single function
- **✅ Async Support**: Full support for async functions
- **✅ Generic Functions**: Preserves generics and lifetimes
- **✅ Zero Cost Abstractions**: <10ns overhead for simple aspects
- **✅ Production-Ready Aspects**: 8 battle-tested aspects in `aspect-std`
- **✅ Well-Tested**: 108+ tests across all crates
- **✅ Comprehensive Benchmarks**: Performance validated with criterion

## Core Concepts

### Aspects

An **aspect** is a module that encapsulates a cross-cutting concern. Implement the `Aspect` trait to define your aspect's behavior:

```rust
pub trait Aspect: Send + Sync {
    fn before(&self, ctx: &JoinPoint) {}
    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {}
    fn after_error(&self, ctx: &JoinPoint, error: &dyn Error) {}
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, Box<dyn Error>> {
        pjp.proceed()
    }
}
```

### JoinPoints

A **joinpoint** represents a point in your program's execution where an aspect can be applied (e.g., function call). The `JoinPoint` struct provides context:

```rust
pub struct JoinPoint {
    pub function_name: &'static str,
    pub module_path: &'static str,
    pub location: Location,
}
```

### Advice

**Advice** is the action taken by an aspect at a joinpoint:

- **`before`**: Runs before the function executes
- **`after`**: Runs after successful execution
- **`after_error`**: Runs if the function returns an error
- **`around`**: Wraps the entire function execution

## Examples

### Logging

```rust
#[derive(Default)]
struct Logger;

impl Aspect for Logger {
    fn before(&self, ctx: &JoinPoint) {
        println!("[{}:{}] Entering {}",
            ctx.location.file,
            ctx.location.line,
            ctx.function_name);
    }
}

#[aspect(Logger)]
fn process_data(input: &str) -> Result<String, Error> {
    Ok(input.to_uppercase())
}
```

### Performance Monitoring

```rust
struct Timer;

impl Aspect for Timer {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, Box<dyn Error>> {
        let start = Instant::now();
        let result = pjp.proceed();
        let elapsed = start.elapsed();
        println!("{} took {:?}", pjp.context().function_name, elapsed);
        result
    }
}

#[aspect(Timer)]
fn expensive_computation(n: u64) -> u64 {
    // Heavy computation
    fibonacci(n)
}
```

### Multiple Aspects

Stack aspects to compose behavior:

```rust
#[aspect(Logger)]
#[aspect(Timer)]
#[aspect(Cache)]
fn fetch_data(key: String) -> Result<Data, Error> {
    // Business logic
}
```

Aspects execute in order: Cache → Timer → Logger → function → Logger → Timer → Cache

## Comprehensive Examples

See the [`aspect-examples/`](aspect-examples/) directory for complete working examples:

### Basic Examples
- **[logging.rs](aspect-examples/src/logging.rs)**: Entry/exit logging with timestamps
- **[timing.rs](aspect-examples/src/timing.rs)**: Performance measurement and slow function warnings
- **[caching.rs](aspect-examples/src/caching.rs)**: Memoization and cache monitoring

### Advanced Examples
- **[advanced_aspects.rs](aspect-examples/src/advanced_aspects.rs)**: Rate limiting, circuit breakers, authorization, validation
- **[api_server.rs](aspect-examples/src/api_server.rs)**: Complete REST API with CRUD operations
- **More patterns coming soon**: Distributed tracing, async aspects, custom pointcuts

### Run Examples

```bash
# Run the API server example
cargo run --example api_server

# Run advanced aspects demo
cargo run --example advanced_aspects

# Run all examples
cargo run --example logging
cargo run --example timing
cargo run --example caching
```

## Architecture

```
aspect-rs/
├── aspect-core/       # Core traits and types (Aspect, JoinPoint, etc.)
├── aspect-macros/     # Procedural macros (#[aspect] attribute)
├── aspect-std/        # Production-ready aspects library (8 aspects)
├── aspect-runtime/    # Runtime utilities and registry
├── aspect-examples/   # Comprehensive examples and patterns
├── aspect-driver/     # rustc-driver integration
└── cargo-aspect/      # Cargo plugin for automatic weaving
```

### How It Works

The `#[aspect]` macro transforms your function at compile time:

```rust
// You write:
#[aspect(LoggingAspect::new())]
fn process_data(input: &str) -> Result<String, Error> {
    Ok(input.to_uppercase())
}

// Macro expands to:
fn __aspect_original_process_data(input: &str) -> Result<String, Error> {
    Ok(input.to_uppercase())
}

fn process_data(input: &str) -> Result<String, Error> {
    let ctx = JoinPoint {
        function_name: "process_data",
        module_path: module_path!(),
        location: Location { file: file!(), line: line!() },
    };

    // Before advice
    LoggingAspect::new().before(&ctx);

    // Execute original function
    let result = __aspect_original_process_data(input);

    // After advice
    LoggingAspect::new().after(&ctx, &result);

    result
}
```

This approach provides:
- **Zero runtime overhead** when aspects are no-ops
- **Compile-time safety** - all aspect code is type-checked
- **Clean generated code** - easy to debug with `cargo expand`

## Performance

Benchmarked on AMD Ryzen 9 5950X (see [`BENCHMARKS.md`](BENCHMARKS.md)):

| Scenario | Overhead | Target |
|----------|----------|--------|
| No-op aspect | ~2ns | <10ns ✅ |
| Simple logging | ~8ns | <10ns ✅ |
| Multiple aspects | ~15ns | <20ns ✅ |
| Complex caching | Variable | Depends on cache hit rate |

**All performance targets met!**

## 📚 Documentation

### Getting Started
- **[Quick Start Guide](QUICK_START.md)** - Get up and running in 5 minutes
- **[API Documentation](https://docs.rs/aspect-core)** - Complete API reference
- **[Examples](aspect-examples/)** - 10 real-world code examples

### Migration & Comparison
- **[Migration Guide](MIGRATION_GUIDE.md)** - Migrate from manual code, decorators, middleware, or other AOP frameworks
- **[Comparison Guide](COMPARISON.md)** - Compare aspect-rs with AspectJ, PostSharp, and alternative approaches

### Advanced Topics
- **[Benchmarks Guide](BENCHMARKS.md)** - Performance analysis and optimization techniques
- **[Optimization Guide](OPTIMIZATION_GUIDE.md)** - Strategies for achieving <5% overhead

### Project Information
- **[Changelog](CHANGELOG.md)** - Complete version history
- **[Contributing](CONTRIBUTING.md)** - Contribution guidelines
- **[Release Checklist](RELEASE_CHECKLIST.md)** - Publication and release process

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Inspiration

Inspired by AspectJ (Java), Spring AOP, and the Rust community's desire for clean, modular code.
