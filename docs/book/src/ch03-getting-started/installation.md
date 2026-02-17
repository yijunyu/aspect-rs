# Installation

## Prerequisites

- **Rust 1.70 or later** - aspect-rs uses modern proc macro features
- **Cargo** - Rust's package manager (comes with rustc)

Check your Rust version:

```bash
rustc --version
# Should show: rustc 1.70.0 or higher
```

If you need to install or update Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

## Add Dependencies

Add aspect-rs to your `Cargo.toml`:

```toml
[dependencies]
aspect-core = "0.1"      # Core traits and types
aspect-macros = "0.1"    # #[aspect] macro
aspect-std = "0.1"       # Optional: 8 production-ready aspects
```

### Minimal Installation

If you want to write custom aspects without using the standard library:

```toml
[dependencies]
aspect-core = "0.1"
aspect-macros = "0.1"
```

### Full Installation (Recommended)

For production use with pre-built aspects:

```toml
[dependencies]
aspect-core = "0.1"
aspect-macros = "0.1"
aspect-std = "0.1"
```

## Verify Installation

Create a new project and test the installation:

```bash
cargo new aspect-test
cd aspect-test
```

Edit `Cargo.toml`:

```toml
[package]
name = "aspect-test"
version = "0.1.0"
edition = "2021"

[dependencies]
aspect-core = "0.1"
aspect-macros = "0.1"
aspect-std = "0.1"
```

Edit `src/main.rs`:

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;

#[derive(Default)]
struct TestAspect;

impl Aspect for TestAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("✅ aspect-rs is working! Function: {}", ctx.function_name);
    }
}

#[aspect(TestAspect)]
fn test_function() {
    println!("Hello from test_function!");
}

fn main() {
    test_function();
}
```

Run it:

```bash
cargo run
```

Expected output:

```
✅ aspect-rs is working! Function: test_function
Hello from test_function!
```

If you see this output, aspect-rs is installed correctly!

## Troubleshooting

### Error: "cannot find macro `aspect` in this scope"

**Solution**: Add `aspect-macros` to your dependencies:

```toml
[dependencies]
aspect-macros = "0.1"
```

### Error: "failed to resolve: use of undeclared type `Aspect`"

**Solution**: Import the prelude:

```rust
use aspect_core::prelude::*;
```

### Error: "no method named `before` found"

**Solution**: Implement the `Aspect` trait:

```rust
impl Aspect for YourAspect {
    fn before(&self, ctx: &JoinPoint) {
        // Your code here
    }
}
```

### Compiler Version Too Old

If you get errors about unstable features, update Rust:

```bash
rustup update stable
rustc --version  # Verify 1.70+
```

## Next Steps

Installation complete! Let's write your first aspect in [Hello World](hello-world.md).
