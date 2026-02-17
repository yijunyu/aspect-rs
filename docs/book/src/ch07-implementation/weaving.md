# Code Weaving Process

The aspect weaving process is where the magic happens - transforming your annotated code into executable Rust that seamlessly integrates aspect behavior. This chapter explores how aspect-rs performs compile-time code weaving through AST transformation.

## What is Weaving?

Weaving is the process of integrating aspect code with your business logic. In aspect-rs, this happens at compile time through procedural macros that transform your source code's Abstract Syntax Tree (AST).

```rust
// Before weaving (what you write):
#[aspect(LoggingAspect::new())]
fn fetch_user(id: u64) -> User {
    database::get(id)
}

// After weaving (what the compiler sees):
fn fetch_user(id: u64) -> User {
    let __aspect_ctx = JoinPoint {
        function_name: "fetch_user",
        module_path: module_path!(),
        location: Location { file: file!(), line: line!() },
    };

    let __aspect_instance = LoggingAspect::new();
    __aspect_instance.before(&__aspect_ctx);

    let __aspect_result = (|| { database::get(id) })();

    __aspect_instance.after(&__aspect_ctx, &__aspect_result);
    __aspect_result
}
```

## Weaving Strategies

aspect-rs supports two main weaving strategies:

### 1. Inline Weaving (Phase 1-2)

The aspect code is directly inserted into the function body:

**Advantages**:
- Simple implementation
- Easy to debug (use `cargo expand`)
- Direct control over execution order
- No runtime overhead

**Disadvantages**:
- Increases code size
- Manual annotation required
- Can't be toggled at runtime

### 2. Wrapper Weaving (Alternative)

The original function is renamed and wrapped:

```rust
// Original function renamed
fn __aspect_original_fetch_user(id: u64) -> User {
    database::get(id)
}

// New wrapper with aspect logic
#[inline(always)]
fn fetch_user(id: u64) -> User {
    let ctx = JoinPoint { /* ... */ };
    let aspect = LoggingAspect::new();
    aspect.before(&ctx);
    let result = __aspect_original_fetch_user(id);
    aspect.after(&ctx, &result);
    result
}
```

**Advantages**:
- Original function preserved
- Easier to test in isolation
- Can be inlined by optimizer
- Clean separation

**Disadvantages**:
- More complex transformation
- Potential visibility issues
- Extra function in symbol table

## AST Transformation Process

### Step 1: Parse the Attribute

```rust
#[aspect(LoggingAspect::new())]
//      ^^^^^^^^^^^^^^^^^^^
//      This expression is parsed
```

The macro receives:
- `attr`: The aspect expression (`LoggingAspect::new()`)
- `item`: The function being annotated

```rust
#[proc_macro_attribute]
pub fn aspect(attr: TokenStream, item: TokenStream) -> TokenStream {
    let aspect_expr: Expr = syn::parse(attr)?;
    let mut func: ItemFn = syn::parse(item)?;
    // ...
}
```

### Step 2: Extract Function Metadata

```rust
let fn_name = &func.sig.ident;          // "fetch_user"
let fn_vis = &func.vis;                  // pub/pub(crate)/private
let fn_inputs = &func.sig.inputs;        // Parameters
let fn_output = &func.sig.output;        // Return type
let fn_asyncness = &func.sig.asyncness;  // async or sync
let fn_generics = &func.sig.generics;    // Generic parameters
```

### Step 3: Generate JoinPoint

```rust
let ctx_init = quote! {
    let __aspect_ctx = ::aspect_core::JoinPoint {
        function_name: stringify!(#fn_name),
        module_path: module_path!(),
        location: ::aspect_core::Location {
            file: file!(),
            line: line!(),
            column: 0,
        },
    };
};
```

### Step 4: Transform Function Body

For synchronous functions:

```rust
let original_body = &func.block;
let new_body = quote! {
    {
        #ctx_init

        let __aspect_instance = #aspect_expr;
        __aspect_instance.before(&__aspect_ctx);

        let __aspect_result = (|| #original_body)();

        __aspect_instance.after(&__aspect_ctx, &__aspect_result);
        __aspect_result
    }
};
```

For async functions:

```rust
let new_body = quote! {
    {
        #ctx_init

        let __aspect_instance = #aspect_expr;
        __aspect_instance.before(&__aspect_ctx);

        let __aspect_result = async #original_body.await;

        __aspect_instance.after(&__aspect_ctx, &__aspect_result);
        __aspect_result
    }
};
```

### Step 5: Handle Return Types

Special handling for `Result<T, E>`:

```rust
let new_body = quote! {
    {
        #ctx_init
        let __aspect_instance = #aspect_expr;
        __aspect_instance.before(&__aspect_ctx);

        let __aspect_result: #return_type = (|| #original_body)();

        match &__aspect_result {
            Ok(val) => __aspect_instance.after(&__aspect_ctx, val),
            Err(err) => {
                let aspect_err = ::aspect_core::AspectError::execution(
                    format!("{:?}", err)
                );
                __aspect_instance.after_error(&__aspect_ctx, &aspect_err);
            }
        }

        __aspect_result
    }
};
```

### Step 6: Reconstruct Function

```rust
func.block = Box::new(syn::parse2(new_body)?);
let output = quote! { #func };
output.into()
```

## Multiple Aspects

When multiple aspects are applied:

```rust
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn fetch_user(id: u64) -> User {
    database::get(id)
}
```

They are applied from **bottom to top** (inner to outer):

```rust
fn fetch_user(id: u64) -> User {
    // TimingAspect (outer)
    let ctx1 = JoinPoint { /* ... */ };
    let timing = TimingAspect::new();
    timing.before(&ctx1);

    // LoggingAspect (inner)
    let ctx2 = JoinPoint { /* ... */ };
    let logging = LoggingAspect::new();
    logging.before(&ctx2);

    // Original function
    let result = database::get(id);

    logging.after(&ctx2, &result);
    timing.after(&ctx1, &result);

    result
}
```

**Execution order**:
1. TimingAspect::before()
2. LoggingAspect::before()
3. Original function
4. LoggingAspect::after()
5. TimingAspect::after()

## Error Handling Integration

aspect-rs integrates with Rust's error handling:

```rust
#[aspect(ErrorHandlingAspect::new())]
fn risky_operation() -> Result<Data, Error> {
    might_fail()?;
    Ok(data)
}

// Weaved code:
fn risky_operation() -> Result<Data, Error> {
    let ctx = JoinPoint { /* ... */ };
    let aspect = ErrorHandlingAspect::new();
    aspect.before(&ctx);

    let result = (|| {
        might_fail()?;
        Ok(data)
    })();

    match &result {
        Ok(val) => aspect.after(&ctx, val),
        Err(e) => {
            let err = AspectError::execution(format!("{:?}", e));
            aspect.after_error(&ctx, &err);
        }
    }

    result
}
```

## Generic Functions

Weaving works seamlessly with generic functions:

```rust
#[aspect(LoggingAspect::new())]
fn process<T: Debug>(item: T) -> T {
    item
}

// Weaved to:
fn process<T: Debug>(item: T) -> T {
    let ctx = JoinPoint {
        function_name: "process",
        module_path: module_path!(),
        location: Location { file: file!(), line: line!() },
    };

    let aspect = LoggingAspect::new();
    aspect.before(&ctx);

    let result = (|| item)();

    aspect.after(&ctx, &result);
    result
}
```

## Macro Expansion

Use `cargo expand` to see the weaved code:

```bash
# Install cargo-expand
cargo install cargo-expand

# Expand a specific function
cargo expand --lib my_module::my_function

# Expand an entire module
cargo expand --lib my_module
```

Example output:

```rust
// Original:
#[aspect(LoggingAspect::new())]
fn example() -> i32 { 42 }

// Expanded:
fn example() -> i32 {
    let __aspect_ctx = ::aspect_core::JoinPoint {
        function_name: "example",
        module_path: "my_crate::my_module",
        location: ::aspect_core::Location {
            file: "src/my_module.rs",
            line: 10u32,
            column: 0u32,
        },
    };
    let __aspect_instance = LoggingAspect::new();
    __aspect_instance.before(&__aspect_ctx);
    let __aspect_result = (|| { 42 })();
    __aspect_instance.after(&__aspect_ctx, &__aspect_result);
    __aspect_result
}
```

## Best Practices

### 1. Keep Aspects Lightweight

```rust
// Good: Lightweight logging
impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        log::debug!("Entering {}", ctx.function_name);
    }
}

// Bad: Heavy computation
impl Aspect for BadAspect {
    fn before(&self, ctx: &JoinPoint) {
        let expensive_data = compute_analytics();
        send_to_monitoring_service(expensive_data);
    }
}
```

### 2. Minimize Allocations

```rust
// Good: Stack allocation
let ctx = JoinPoint {
    function_name: "example",
    module_path: module_path!(),
    location: Location { /* stack allocated */ },
};

// Bad: Heap allocation
let ctx = Box::new(JoinPoint { /* ... */ });
```

### 3. Test Both Paths

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_without_aspect() {
        let result = core_logic(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_aspect() {
        let result = aspected_function(input);
        assert_eq!(result, expected);
    }
}
```

## Summary

The weaving process in aspect-rs:

1. **Parses** the `#[aspect(...)]` attribute at compile time
2. **Extracts** function metadata (name, parameters, return type)
3. **Generates** JoinPoint context with compile-time constants
4. **Transforms** the function body to integrate aspect calls
5. **Preserves** generics, lifetimes, and async/await
6. **Optimizes** through inlining and constant folding

The result is zero-runtime-overhead aspect integration that maintains Rust's performance guarantees.

**Next**: [Performance Optimizations](optimizations.md) - Techniques to minimize aspect overhead.
