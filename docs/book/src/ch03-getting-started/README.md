# Getting Started

Get up and running with aspect-rs in under 5 minutes!

## What You'll Learn

By the end of this chapter, you'll be able to:

- Install aspect-rs in your Rust project
- Write your first aspect from scratch
- Use pre-built production-ready aspects
- Apply multiple aspects to functions
- Understand the execution model

## Prerequisites

- **Rust 1.70+** ([install rustup](https://rustup.rs/))
- Basic Rust knowledge (functions, traits, cargo)
- A text editor or IDE with Rust support

## Quick Example

Here's what aspect-rs looks like:

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;

// Define an aspect
#[derive(Default)]
struct Logger;

impl Aspect for Logger {
    fn before(&self, ctx: &JoinPoint) {
        println!("→ {}", ctx.function_name);
    }
}

// Apply it
#[aspect(Logger)]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    let msg = greet("World");
    // Prints: "→ greet"
    // Prints: "Hello, World!"
}
```

**That's it!** Logging automatically added with zero runtime overhead.

## Chapter Outline

1. **[Installation](installation.md)** - Add aspect-rs to your project
2. **[Hello World](hello-world.md)** - Simplest possible example
3. **[Quick Start Guide](quick-start.md)** - Comprehensive 5-minute tutorial
4. **[Using Pre-built Aspects](prebuilt.md)** - Leverage production-ready aspects

Let's begin with [Installation](installation.md)!
