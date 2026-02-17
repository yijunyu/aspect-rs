# Hello World

The simplest possible aspect-rs program.

## The Code

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;

// Step 1: Define an aspect
#[derive(Default)]
struct HelloAspect;

impl Aspect for HelloAspect {
    fn before(&self, _ctx: &JoinPoint) {
        println!("Hello from aspect!");
    }
}

// Step 2: Apply it to a function
#[aspect(HelloAspect)]
fn my_function() {
    println!("Hello from function!");
}

// Step 3: Call the function
fn main() {
    my_function();
}
```

## Output

```
Hello from aspect!
Hello from function!
```

## How It Works

1. **Define**: `HelloAspect` implements the `Aspect` trait with a `before` method
2. **Apply**: The `#[aspect(HelloAspect)]` macro weaves the aspect into `my_function`
3. **Execute**: When `my_function()` is called, the aspect runs before the function body

## What Gets Generated

The `#[aspect(...)]` macro generates code like this (simplified):

```rust
fn my_function() {
    // Generated aspect code
    let aspect = HelloAspect;
    let ctx = JoinPoint {
        function_name: "my_function",
        module_path: module_path!(),
        file: file!(),
        line: line!(),
    };

    aspect.before(&ctx);  // Runs before function

    // Original function body
    println!("Hello from function!");
}
```

All of this happens at **compile time** - zero runtime overhead!

## Next Steps

This is the simplest example. For a more comprehensive introduction, see [Quick Start Guide](quick-start.md).
