# Macro Code Generation

This chapter details how the `#[aspect]` procedural macro transforms annotated functions to weave aspect behavior at compile time.

## Overview

The aspect-rs macro system performs **compile-time code transformation** to weave aspects into your functions. This approach provides:

- **Zero runtime overhead** - All aspect setup happens at compile time
- **Type safety** - Compiler verifies all generated code
- **Transparent integration** - Works with existing Rust tooling
- **No reflection** - No runtime introspection needed

## Macro Architecture

The `#[aspect]` macro follows a standard procedural macro pipeline:

```
Input Source Code
    ↓
Macro Attribute Parser
    ↓
Function AST Analysis
    ↓
Code Generator
    ↓
Output TokenStream
    ↓
Rust Compiler
```

### Component Breakdown

#### 1. Entry Point (`aspect-macros/src/lib.rs`)

```rust
#[proc_macro_attribute]
pub fn aspect(attr: TokenStream, item: TokenStream) -> TokenStream {
    let aspect_expr = parse_macro_input!(attr as Expr);
    let func = parse_macro_input!(item as ItemFn);

    aspect_attr::transform(aspect_expr, func)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
```

**What happens here**:
1. Parse the aspect expression (e.g., `LoggingAspect::new()`)
2. Parse the function being annotated
3. Transform the function with aspect weaving
4. Convert errors to compiler errors if transformation fails

#### 2. Parser (`aspect-macros/src/parsing.rs`)

```rust
pub struct AspectInfo {
    pub aspect_expr: Expr,
}

impl AspectInfo {
    pub fn parse(expr: Expr) -> Result<Self> {
        // Validate the aspect expression
        Ok(AspectInfo { aspect_expr: expr })
    }
}
```

**Validation includes**:
- Aspect expression is valid Rust syntax
- Expression evaluates to a type implementing `Aspect`
- Type checking deferred to Rust compiler

#### 3. Transformer (`aspect-macros/src/aspect_attr.rs`)

```rust
pub fn transform(aspect_expr: Expr, func: ItemFn) -> Result<TokenStream> {
    let aspect_info = AspectInfo::parse(aspect_expr)?;
    let output = generate_aspect_wrapper(&aspect_info, &func);
    Ok(output)
}
```

**Transformation strategy**:
1. Extract function metadata (name, parameters, return type)
2. Generate renamed original function
3. Create wrapper function with aspect calls
4. Preserve all function signatures and attributes

## Code Generation Process

### Step 1: Rename Original Function

The original function is preserved with a mangled name:

**Input**:
```rust
#[aspect(Logger)]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

**Generated (Step 1)**:
```rust
fn __aspect_original_greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

**Why rename?**
- Preserves original business logic unchanged
- Allows wrapper to call original
- Prevents name collision
- Enables clean separation

### Step 2: Extract Function Metadata

```rust
let fn_name = &func.sig.ident;           // "greet"
let fn_vis = &func.vis;                  // pub/private
let fn_inputs = &func.sig.inputs;        // Parameters
let fn_output = &func.sig.output;        // Return type
let fn_generics = &func.sig.generics;    // Generic params
let fn_asyncness = &func.sig.asyncness;  // async keyword
```

### Step 3: Create JoinPoint Context

```rust
let __context = JoinPoint {
    function_name: "greet",
    module_path: "my_crate::api",
    location: Location {
        file: "src/api.rs",
        line: 42,
    },
};
```

**Metadata captured**:
- `function_name` - From `fn_name.to_string()`
- `module_path` - From `module_path!()` macro
- `file` - From `file!()` macro
- `line` - From `line!()` macro

All captured at **compile time** with zero runtime cost.

### Step 4: Create ProceedingJoinPoint

```rust
let __pjp = ProceedingJoinPoint::new(
    || {
        let __result = __aspect_original_greet(name);
        Ok(Box::new(__result) as Box<dyn Any>)
    },
    __context,
);
```

**ProceedingJoinPoint wraps**:
- Original function as a closure
- Execution context
- Provides `proceed()` method for aspect

### Step 5: Call Aspect's Around Method

```rust
let __aspect = Logger;

match __aspect.around(__pjp) {
    Ok(__boxed_result) => {
        *__boxed_result
            .downcast::<String>()
            .expect("aspect around() returned wrong type")
    }
    Err(__err) => {
        panic!("aspect around() failed: {:?}", __err);
    }
}
```

### Step 6: Generate Wrapper Function

**Final generated code**:
```rust
// Original function (renamed, private)
fn __aspect_original_greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Wrapper function (original name, public)
pub fn greet(name: &str) -> String {
    use ::aspect_core::prelude::*;
    use ::std::any::Any;

    let __aspect = Logger;
    let __context = JoinPoint {
        function_name: "greet",
        module_path: module_path!(),
        location: Location {
            file: file!(),
            line: line!(),
        },
    };

    let __pjp = ProceedingJoinPoint::new(
        || {
            let __result = __aspect_original_greet(name);
            Ok(Box::new(__result) as Box<dyn Any>)
        },
        __context,
    );

    match __aspect.around(__pjp) {
        Ok(__boxed_result) => {
            *__boxed_result
                .downcast::<String>()
                .expect("aspect around() returned wrong type")
        }
        Err(__err) => {
            panic!("aspect around() failed: {:?}", __err);
        }
    }
}
```

## Handling Different Function Types

### Non-Result Return Types

For functions returning concrete types (not `Result`):

```rust
#[aspect(Timer)]
fn calculate(x: i32) -> i32 {
    x * 2
}
```

**Generated wrapper**:
```rust
pub fn calculate(x: i32) -> i32 {
    // ... setup ...

    let __pjp = ProceedingJoinPoint::new(
        || {
            let __result = __aspect_original_calculate(x);
            Ok(Box::new(__result) as Box<dyn Any>)
        },
        __context,
    );

    match __aspect.around(__pjp) {
        Ok(__boxed_result) => {
            *__boxed_result
                .downcast::<i32>()
                .expect("type mismatch")
        }
        Err(__err) => {
            panic!("aspect failed: {:?}", __err);
        }
    }
}
```

### Result Return Types

For functions returning `Result<T, E>`:

```rust
#[aspect(Logger)]
fn fetch_user(id: u64) -> Result<User, DbError> {
    database::get(id)
}
```

**Generated wrapper**:
```rust
pub fn fetch_user(id: u64) -> Result<User, DbError> {
    // ... setup ...

    let __pjp = ProceedingJoinPoint::new(
        || {
            match __aspect_original_fetch_user(id) {
                Ok(__val) => Ok(Box::new(__val) as Box<dyn Any>),
                Err(__err) => Err(AspectError::execution(format!("{:?}", __err))),
            }
        },
        __context,
    );

    match __aspect.around(__pjp) {
        Ok(__boxed_result) => {
            let __inner = *__boxed_result
                .downcast::<User>()
                .expect("type mismatch");
            Ok(__inner)
        }
        Err(__err) => {
            Err(format!("{:?}", __err).into())
        }
    }
}
```

**Key difference**: Errors converted to `AspectError` and back.

### Async Functions

For `async fn`:

```rust
#[aspect(Logger)]
async fn fetch_data(url: &str) -> Result<String, Error> {
    reqwest::get(url).await?.text().await
}
```

**Generated wrapper**:
```rust
pub async fn fetch_data(url: &str) -> Result<String, Error> {
    use ::aspect_core::prelude::*;
    use ::std::any::Any;

    let __aspect = Logger;
    let __context = JoinPoint {
        function_name: "fetch_data",
        module_path: module_path!(),
        location: Location { file: file!(), line: line!() },
    };

    // Before advice
    __aspect.before(&__context);

    // Execute original function
    let __result = __aspect_original_fetch_data(url).await;

    // After advice
    match &__result {
        Ok(__val) => {
            __aspect.after(&__context, __val as &dyn Any);
        }
        Err(__err) => {
            let __aspect_err = AspectError::execution(format!("{:?}", __err));
            __aspect.after_error(&__context, &__aspect_err);
        }
    }

    __result
}
```

**Note**: Async functions use before/after instead of around (no stable async traits yet).

### Generic Functions

For functions with type parameters:

```rust
#[aspect(Logger)]
fn identity<T: Debug>(value: T) -> T {
    println!("{:?}", value);
    value
}
```

**Generated wrapper preserves generics**:
```rust
fn __aspect_original_identity<T: Debug>(value: T) -> T {
    println!("{:?}", value);
    value
}

pub fn identity<T: Debug>(value: T) -> T {
    use ::aspect_core::prelude::*;

    let __aspect = Logger;
    let __context = JoinPoint { /* ... */ };

    let __pjp = ProceedingJoinPoint::new(
        || {
            let __result = __aspect_original_identity(value);
            Ok(Box::new(__result) as Box<dyn Any>)
        },
        __context,
    );

    match __aspect.around(__pjp) {
        Ok(__boxed_result) => {
            *__boxed_result.downcast::<T>().expect("type mismatch")
        }
        Err(__err) => panic!("{:?}", __err),
    }
}
```

**Challenge**: Type erasure via `Box<dyn Any>` works because `T: 'static` implied by `Any`.

## Advanced Generation Techniques

### Multiple Aspects

When multiple `#[aspect]` macros applied:

```rust
#[aspect(Logger)]
#[aspect(Timer)]
fn my_function() { }
```

**Processing order** (bottom-up):
1. `Timer` macro applied first
2. `Logger` macro wraps Timer's output

**Generated nesting**:
```rust
// After Timer
fn __aspect_original_my_function() { }
fn __timer_my_function() {
    // Timer aspect wrapping original
}

// After Logger
fn __logger___timer_my_function() {
    // Logger aspect wrapping Timer
}

pub fn my_function() {
    // Logger wrapper calling Timer wrapper
}
```

### Preserving Attributes

Non-aspect attributes are preserved:

```rust
#[inline]
#[cold]
#[aspect(Logger)]
fn rare_function() { }
```

**Generated**:
```rust
fn __aspect_original_rare_function() { }

#[inline]
#[cold]
pub fn rare_function() {
    // Aspect wrapper
}
```

**Attributes copied to**:
- Wrapper function (visible to callers)
- NOT original (internal implementation)

### Capturing Closure Variables

For closures in aspect expressions:

```rust
let prefix = "[LOG]";
#[aspect(LoggerWithPrefix::new(prefix))]
fn my_func() { }
```

**Generated**:
```rust
pub fn my_func() {
    let prefix = "[LOG]";  // Captured at call site
    let __aspect = LoggerWithPrefix::new(prefix);
    // ... rest of wrapper ...
}
```

## Optimization Strategies

### Inline Hints

Generated wrappers marked for inlining:

```rust
#[inline(always)]
pub fn my_function() {
    // Aspect wrapper
}
```

**Result**: Compiler may inline entire aspect chain.

### Const Evaluation

JoinPoint data as constants:

```rust
const __JOINPOINT_DATA: &str = "my_function";

pub fn my_function() {
    let __context = JoinPoint {
        function_name: __JOINPOINT_DATA,  // No allocation!
        // ...
    };
}
```

### Dead Code Elimination

For no-op aspects:

```rust
impl Aspect for NoOpAspect {
    fn before(&self, _: &JoinPoint) { }
    fn after(&self, _: &JoinPoint, _: &dyn Any) { }
}
```

**Compiler optimizes**:
```rust
pub fn my_function() {
    // Empty before() inlined away
    let result = __aspect_original_my_function();
    // Empty after() inlined away
    result
}
```

**Final code**: Identical to no aspect!

## Error Handling

### Compilation Errors

Macro generates compiler errors for:

**Invalid aspect expression**:
```rust
#[aspect(NotAnAspect)]
fn my_func() { }
```

**Error**: `NotAnAspect` does not implement `Aspect`.

**Type mismatch**:
```rust
#[aspect(Logger)]
fn my_func() -> i32 {
    "not an i32"  // Type error
}
```

**Error**: Expected `i32`, found `&str`.

### Runtime Type Safety

Downcasting validates types:

```rust
*__boxed_result
    .downcast::<String>()
    .expect("aspect around() returned wrong type")
```

**Panic if**: Aspect returns wrong type (programmer error).

## Expansion Examples

### Simple Function

**Input**:
```rust
#[aspect(Logger)]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Expanded** (via `cargo expand`):
```rust
fn __aspect_original_add(a: i32, b: i32) -> i32 {
    a + b
}

fn add(a: i32, b: i32) -> i32 {
    use ::aspect_core::prelude::*;
    use ::std::any::Any;

    let __aspect = Logger;
    let __context = JoinPoint {
        function_name: "add",
        module_path: "my_crate",
        location: Location {
            file: "src/main.rs",
            line: 10u32,
        },
    };

    let __pjp = ProceedingJoinPoint::new(
        || {
            let __result = __aspect_original_add(a, b);
            Ok(Box::new(__result) as Box<dyn Any>)
        },
        __context,
    );

    match __aspect.around(__pjp) {
        Ok(__boxed_result) => {
            *__boxed_result
                .downcast::<i32>()
                .expect("aspect around() returned wrong type")
        }
        Err(__err) => panic!("aspect around() failed: {:?}", __err),
    }
}
```

### Result Function

**Input**:
```rust
#[aspect(Logger)]
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```

**Expanded**:
```rust
fn __aspect_original_divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn divide(a: i32, b: i32) -> Result<i32, String> {
    use ::aspect_core::prelude::*;
    use ::std::any::Any;

    let __aspect = Logger;
    let __context = JoinPoint {
        function_name: "divide",
        module_path: "my_crate",
        location: Location {
            file: "src/main.rs",
            line: 20u32,
        },
    };

    let __pjp = ProceedingJoinPoint::new(
        || match __aspect_original_divide(a, b) {
            Ok(__val) => Ok(Box::new(__val) as Box<dyn Any>),
            Err(__err) => Err(AspectError::execution(format!("{:?}", __err))),
        },
        __context,
    );

    match __aspect.around(__pjp) {
        Ok(__boxed_result) => {
            let __inner = *__boxed_result
                .downcast::<i32>()
                .expect("aspect around() returned wrong type");
            Ok(__inner)
        }
        Err(__err) => Err(format!("{:?}", __err).into()),
    }
}
```

## Testing Generated Code

### Viewing Expansions

```bash
# Install cargo-expand
cargo install cargo-expand

# View expanded macros
cargo expand --lib
cargo expand --example logging

# View specific function
cargo expand my_function
```

### Unit Testing Macros

```rust
#[test]
fn test_aspect_macro() {
    #[aspect(TestAspect)]
    fn test_func() -> i32 {
        42
    }

    let result = test_func();
    assert_eq!(result, 42);
}
```

### Integration Testing

See `aspect-macros/tests/` for comprehensive tests.

## Performance Characteristics

### Compile-Time Cost

- **Macro expansion**: ~10ms per function
- **Type checking**: Standard Rust cost
- **Code generation**: Minimal impact

**Total overhead**: Negligible for typical projects.

### Runtime Cost

- **Wrapper overhead**: 0-5ns (inline eliminated)
- **JoinPoint creation**: ~2ns (stack allocation)
- **Virtual dispatch**: ~1-2ns (aspect.around() call)

**Total**: <10ns for simple aspects.

See [Benchmarks](../ch09-benchmarks/results.md) for details.

## Limitations and Workarounds

### Cannot Intercept Method Calls

**Limitation**: Macro works on function definitions only.

```rust
#[aspect(Logger)]
impl MyStruct {
    fn method(&self) { }  // ❌ Not supported
}
```

**Workaround**: Apply to individual methods:

```rust
impl MyStruct {
    #[aspect(Logger)]
    fn method(&self) { }  // ✅ Works
}
```

### Cannot Modify External Code

**Limitation**: Must control source code.

**Workaround**: Use Phase 3 automatic weaving (see [Chapter 10](../ch10-phase3/README.md)).

### Async Traits Unsupported

**Limitation**: No stable async trait support yet.

**Current approach**: Use before/after instead of around for async.

**Future**: Async traits in development (RFC pending).

## Debugging Macros

### Common Issues

**Issue**: "Cannot find type `JoinPoint`"

**Solution**: Add dependency:
```toml
[dependencies]
aspect-core = "0.1"
```

**Issue**: "Type mismatch in downcast"

**Solution**: Ensure aspect returns correct type:
```rust
fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    let result = pjp.proceed()?;
    // Don't modify result type!
    Ok(result)
}
```

### Debugging Techniques

1. **View expansion**: `cargo expand`
2. **Check compiler errors**: Read full error messages
3. **Simplify**: Remove aspect, verify function works
4. **Test aspect separately**: Unit test aspect implementation

## Best Practices

### DO

✅ Use `cargo expand` to verify generated code
✅ Keep aspect expressions simple
✅ Test aspects independently
✅ Use type inference where possible
✅ Prefer const expressions for aspects

### DON'T

❌ Rely on side effects in aspect expressions
❌ Mutate captured variables
❌ Use expensive computations in aspect constructor
❌ Return wrong types from around advice
❌ Panic in aspects (use Result)

## Summary

The `#[aspect]` macro provides:

1. **Compile-time code transformation** - No runtime magic
2. **Type-safe weaving** - Compiler verifies everything
3. **Transparent integration** - Works with all Rust tools
4. **Zero-cost abstractions** - Optimizes to hand-written code

**Key insight**: Procedural macros enable aspect-oriented programming in Rust while maintaining the language's core principles of zero-cost abstractions and type safety.

## See Also

- [Pointcut Matching](pointcuts.md) - How functions are selected
- [Code Weaving Process](weaving.md) - Complete weaving pipeline
- [Performance Optimizations](optimizations.md) - Optimization techniques
- [Usage Guide](../ch05-usage-guide/README.md) - Practical usage patterns
