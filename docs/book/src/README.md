# Introduction

Welcome to **aspect-rs**, a comprehensive Aspect-Oriented Programming (AOP) framework for Rust that brings the power of cross-cutting concerns modularization to the Rust ecosystem.

## What You'll Learn

This book is your complete guide to aspect-rs, from first principles to advanced techniques. Whether you're new to AOP or coming from AspectJ, you'll find:

- **Clear explanations** of AOP concepts in a Rust context
- **Practical examples** you can use in production today
- **Deep technical insights** into how aspect-rs works under the hood
- **Performance analysis** showing the zero-cost abstraction story
- **Real-world case studies** demonstrating measurable value

## Who This Book Is For

- **Rust developers** curious about AOP and how it can reduce boilerplate
- **AspectJ users** wanting to understand how aspect-rs differs from traditional AOP
- **Library authors** looking to add declarative functionality to their crates
- **Systems programmers** interested in compile-time code generation techniques
- **Contributors** wanting to understand aspect-rs internals

## What is aspect-rs?

aspect-rs is a **compile-time AOP framework** that allows you to modularize cross-cutting concerns without runtime overhead:

```rust
use aspect_std::*;

// Automatically log all function calls
#[aspect(LoggingAspect::new())]
fn process_order(order: Order) -> Result<Receipt, Error> {
    // Your business logic here
    // Logging happens transparently
}
```

## Key Features

- ✅ **Zero runtime overhead** - All weaving happens at compile time
- ✅ **Type-safe** - Full Rust type checking and ownership verification
- ✅ **8 production-ready aspects** - Logging, timing, caching, rate limiting, and more
- ✅ **Automatic weaving** - enables annotation-free AOP
- ✅ **108+ passing tests** - Comprehensive test coverage with benchmarks
- ✅ **9,100+ lines of production code** - Battle-tested and documented

## How to Use This Book

The book is organized into five main sections:

### Getting Started (Chapters 1-3)
Understand the motivation for AOP, learn core concepts, and write your first aspect in 5 minutes.

### User Guide (Chapters 4-5, 8)
Master the Aspect trait, explore common patterns, and study real-world case studies.

### Technical Reference (Chapters 6-7, 9)
Dive deep into architecture, implementation details, and performance characteristics.

### Advanced Topics (Chapter 10)
Explore automatic weaving - the breakthrough that enables annotation-free AOP.

### Community (Chapter 11)
Discover the roadmap, learn how to contribute, and join the aspect-rs community.

## Quick Navigation

New to AOP? Start with [Motivation](ch01-motivation/README.md) to understand why AOP matters.

Want to try it right now? Jump to [Getting Started](ch03-getting-started/README.md) for a 5-minute quickstart.

Coming from AspectJ? Read the [AspectJ Legacy](ch01-motivation/aspectj.md) comparison.

Building a library? Check out [Architecture](ch06-architecture/README.md) for design patterns.

Optimizing performance? See [Performance Benchmarks](ch09-benchmarks/README.md).

## Example: What AOP Looks Like

**Before aspect-rs** (scattered logging):
```rust
fn transfer_funds(from: Account, to: Account, amount: u64) -> Result<(), Error> {
    log::info!("transfer_funds called with from={}, to={}, amount={}", from.id, to.id, amount);
    let start = Instant::now();

    // Actual business logic
    let result = perform_transfer(from, to, amount);

    log::info!("transfer_funds completed in {:?}", start.elapsed());
    result
}
```

**With aspect-rs** (clean separation):
```rust
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn transfer_funds(from: Account, to: Account, amount: u64) -> Result<(), Error> {
    // Pure business logic - no cross-cutting concerns!
    perform_transfer(from, to, amount)
}
```

The logging and timing code is **automatically woven** at compile time, with zero runtime overhead.

## Let's Begin!

Ready to learn aspect-rs? Let's start with [Chapter 1: Motivation](ch01-motivation/README.md) to understand why AOP is a game-changer for Rust development.

---

**Note**: This book documents aspect-rs version 0.1.x. For the latest updates, see the [GitHub repository](https://github.com/yijunyu/aspect-rs).
