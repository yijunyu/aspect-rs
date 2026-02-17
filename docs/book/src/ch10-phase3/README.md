# Automatic Weaving

The breakthrough feature enabling AspectJ-style annotation-free AOP.

## The Vision

**Before**: Per-function annotation required
```rust
#[aspect(LoggingAspect::new())]  // Repeated for every function
fn my_function() { }
```

**After**: Pattern-based automatic weaving
```rust
#[advice(
    pointcut = "execution(pub fn *(..)) && within(crate::api)",
    advice = "before"
)]
static LOGGER: LoggingAspect = LoggingAspect::new();

// No annotation needed - automatically woven!
pub fn api_handler() { }
```

## Technical Achievement

Automated Weaving uses:
- **rustc-driver** to compile code
- **TyCtxt** to access type information
- **MIR extraction** to analyze functions
- **Function pointers** to register aspects globally
- **Pointcut matching** to determine which functions get woven

This is a **major breakthrough** - the first Rust AOP framework with automatic weaving!

See [The Vision](vision.md) for details.
