# Crate Organization

aspect-rs is architected as a modular workspace with clear separation of concerns. The framework consists of seven crates, each with specific responsibilities and dependencies. This chapter details the complete crate structure.

## Overview

```
aspect-rs/
├── aspect-core/           # Foundation (zero dependencies)
├── aspect-macros/         # Procedural macros
├── aspect-runtime/        # Global aspect registry
├── aspect-std/            # Standard aspects library
├── aspect-pointcut/       # Pointcut matching (Phase 2)
├── aspect-weaver/         # Code generation (Phase 2)
└── aspect-rustc-driver/   # Automatic weaving (Phase 3)
```

## aspect-core

**Purpose**: Foundation - Core traits and abstractions
**Version**: 0.1.0
**Dependencies**: None (zero-dependency core)
**Lines of Code**: ~800

### Responsibilities

- Define the `Aspect` trait
- Provide `JoinPoint` and `ProceedingJoinPoint` types
- Implement error handling (`AspectError`)
- Establish pointcut pattern matching foundation
- Export prelude for convenient imports

### Key Types

#### Aspect Trait

```rust
pub trait Aspect: Send + Sync {
    /// Runs before the target function
    fn before(&self, ctx: &JoinPoint) {}

    /// Runs after successful execution
    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {}

    /// Runs when an error occurs
    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {}

    /// Wraps the entire function execution
    fn around(&self, pjp: ProceedingJoinPoint)
        -> Result<Box<dyn Any>, AspectError>
    {
        pjp.proceed()
    }
}
```

#### JoinPoint

```rust
pub struct JoinPoint {
    pub function_name: &'static str,
    pub module_path: &'static str,
    pub location: Location,
}

pub struct Location {
    pub file: &'static str,
    pub line: u32,
}
```

#### ProceedingJoinPoint

```rust
pub struct ProceedingJoinPoint {
    proceed_fn: Box<dyn FnOnce() -> Result<Box<dyn Any>, AspectError>>,
    context: JoinPoint,
}

impl ProceedingJoinPoint {
    pub fn proceed(self) -> Result<Box<dyn Any>, AspectError> {
        (self.proceed_fn)()
    }

    pub fn context(&self) -> &JoinPoint {
        &self.context
    }
}
```

### API Surface

- **Public traits**: 1 (`Aspect`)
- **Public structs**: 3 (`JoinPoint`, `ProceedingJoinPoint`, `Location`)
- **Public enums**: 1 (`AspectError`)
- **Total tests**: 28

### Dependencies

None - completely standalone. This ensures:
- Fast compilation
- No version conflicts
- Easy to vendor
- Clear separation of concerns

## aspect-macros

**Purpose**: Compile-time aspect weaving
**Version**: 0.1.0
**Dependencies**: `syn`, `quote`, `proc-macro2`
**Lines of Code**: ~1,200

### Responsibilities

- Implement `#[aspect(Expr)]` attribute macro
- Implement `#[advice(...)]` attribute macro
- Parse function signatures and attributes
- Generate aspect wrapper code
- Preserve original function semantics
- Emit clean, readable code

### Macros

#### #[aspect] Macro

Transforms an annotated function into an aspect-wrapped version:

```rust
#[aspect(Logger::default())]
fn my_function(x: i32) -> i32 {
    x * 2
}

// Expands to:
fn my_function(x: i32) -> i32 {
    let __aspect = Logger::default();
    let __ctx = JoinPoint {
        function_name: "my_function",
        module_path: module_path!(),
        location: Location {
            file: file!(),
            line: line!(),
        },
    };

    __aspect.before(&__ctx);

    let __result = {
        let x = x;
        x * 2
    };

    __aspect.after(&__ctx, &__result);
    __result
}
```

#### #[advice] Macro

Registers aspects globally with pointcut patterns:

```rust
#[advice(
    pointcut = "execution(pub fn *(..)) && within(crate::api)",
    advice = "around",
    order = 10
)]
fn api_logger(pjp: ProceedingJoinPoint)
    -> Result<Box<dyn Any>, AspectError>
{
    println!("API call: {}", pjp.context().function_name);
    pjp.proceed()
}
```

### Code Generation Process

1. **Parse Input**: Use `syn` to parse function AST
2. **Extract Metadata**: Function name, parameters, return type
3. **Generate JoinPoint**: Create context with location info
4. **Generate Wrapper**: Insert aspect calls around original logic
5. **Preserve Signature**: Maintain exact function signature
6. **Handle Errors**: Wrap Result types appropriately

**Generated Code Quality**:
- Hygienic (no name collisions)
- Readable (useful for debugging)
- Optimizable (compiler can inline)
- Error-preserving (maintains error types)

### API Surface

- **Procedural macros**: 2 (`aspect`, `advice`)
- **Total tests**: 32

### Dependencies

- `syn ^2.0` - Rust parser
- `quote ^1.0` - Code generation
- `proc-macro2 ^1.0` - Token manipulation

## aspect-runtime

**Purpose**: Global aspect registry and management
**Version**: 0.1.0
**Dependencies**: `aspect-core`, `lazy_static`, `parking_lot`
**Lines of Code**: ~400

### Responsibilities

- Maintain global aspect registry
- Register aspects with pointcuts
- Match functions against pointcuts
- Order aspect execution
- Thread-safe access to aspects

### Key Components

#### AspectRegistry

```rust
pub struct AspectRegistry {
    aspects: Vec<RegisteredAspect>,
}

impl AspectRegistry {
    pub fn register(&mut self, aspect: RegisteredAspect) {
        self.aspects.push(aspect);
        self.aspects.sort_by_key(|a| a.order);
    }

    pub fn get_matching_aspects(&self, ctx: &JoinPoint)
        -> Vec<&RegisteredAspect>
    {
        self.aspects
            .iter()
            .filter(|a| a.pointcut.matches(ctx))
            .collect()
    }
}
```

#### RegisteredAspect

```rust
pub struct RegisteredAspect {
    pub aspect: Arc<dyn Aspect>,
    pub pointcut: Pointcut,
    pub order: i32,
    pub name: String,
}
```

### Global Instance

```rust
lazy_static! {
    static ref GLOBAL_REGISTRY: Mutex<AspectRegistry> =
        Mutex::new(AspectRegistry::new());
}

pub fn register_aspect(aspect: RegisteredAspect) {
    GLOBAL_REGISTRY.lock().register(aspect);
}
```

### API Surface

- **Public structs**: 2 (`AspectRegistry`, `RegisteredAspect`)
- **Public functions**: 3
- **Total tests**: 18

## aspect-std

**Purpose**: Production-ready reusable aspects
**Version**: 0.1.0
**Dependencies**: `aspect-core`, various utilities
**Lines of Code**: ~2,100

### Standard Aspects

#### LoggingAspect

Structured logging with multiple backends:

```rust
pub struct LoggingAspect {
    level: LogLevel,
    backend: LogBackend,
}

impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        self.log(
            self.level,
            format!("[ENTRY] {}", ctx.function_name)
        );
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        self.log(
            self.level,
            format!("[EXIT] {}", ctx.function_name)
        );
    }
}
```

**Features**: Level filtering, multiple backends, structured output

#### TimingAspect

Performance monitoring and metrics:

```rust
pub struct TimingAspect {
    threshold_ms: u64,
    reporter: Arc<dyn MetricsReporter>,
}

impl Aspect for TimingAspect {
    fn around(&self, pjp: ProceedingJoinPoint)
        -> Result<Box<dyn Any>, AspectError>
    {
        let start = Instant::now();
        let result = pjp.proceed();
        let elapsed = start.elapsed();

        if elapsed.as_millis() > self.threshold_ms as u128 {
            self.reporter.report_slow_function(
                pjp.context(),
                elapsed
            );
        }

        result
    }
}
```

**Features**: Threshold alerting, histogram tracking, percentiles

#### CachingAspect

Memoization with TTL support:

```rust
pub struct CachingAspect<K, V> {
    cache: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}
```

**Features**: TTL expiration, LRU eviction, cache statistics

#### Complete Aspect List

1. **LoggingAspect** - Structured logging
2. **TimingAspect** - Performance monitoring
3. **CachingAspect** - Memoization
4. **MetricsAspect** - Call statistics
5. **RateLimitAspect** - Request throttling
6. **RetryAspect** - Automatic retry with backoff
7. **TransactionAspect** - Database transactions
8. **AuthorizationAspect** - RBAC security

### API Surface

- **Public aspects**: 8
- **Total tests**: 48

## aspect-pointcut

**Purpose**: Advanced pointcut expression parsing
**Version**: 0.1.0
**Dependencies**: `aspect-core`, `regex`, `nom`
**Lines of Code**: ~900

### Pointcut Expressions

#### Execution Pointcut

```rust
execution(pub fn *(..))              // All public functions
execution(fn fetch_*(u64) -> User)   // Specific pattern
execution(async fn *(..))            // All async functions
```

#### Within Pointcut

```rust
within(crate::api)           // All functions in api module
within(crate::api::*)        // api and submodules
within(crate::*::handlers)   // Any handlers module
```

#### Boolean Combinators

```rust
execution(pub fn *(..)) && within(crate::api)    // AND
execution(pub fn *(..)) || name(fetch_*)         // OR
!within(crate::internal)                          // NOT
```

### API Surface

- **Public structs**: 4
- **Public functions**: 8
- **Total tests**: 34

## aspect-weaver

**Purpose**: Advanced code generation
**Version**: 0.1.0
**Dependencies**: `aspect-core`, `syn`, `quote`
**Lines of Code**: ~700

### Optimization Strategies

- **Inline Everything**: Mark wrappers as `#[inline(always)]`
- **Constant Propagation**: Use `const` for static data
- **Dead Code Elimination**: Remove no-op aspect calls

### API Surface

- **Public structs**: 3
- **Public functions**: 5
- **Total tests**: 22

## aspect-rustc-driver

**Purpose**: Automatic aspect weaving
**Version**: 0.1.0
**Dependencies**: `rustc_driver`, `rustc_middle`, many rustc internals
**Lines of Code**: ~3,000

### Architecture

Complete rustc integration for annotation-free AOP:

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = AspectConfig::from_args(&args);

    rustc_driver::RunCompiler::new(
        &args,
        &mut AspectCallbacks::new(config)
    ).run().unwrap();
}
```

### 6-Step Pipeline

1. **Parse Command Line**: Extract pointcut expressions
2. **Configure Compiler**: Set up custom callbacks
3. **Access TyCtxt**: Get compiler context
4. **Extract MIR**: Analyze compiled functions
5. **Match Pointcuts**: Apply pattern matching
6. **Generate Code**: Weave aspects automatically

### API Surface

- **Binaries**: 1 (`aspect-rustc-driver`)
- **Public structs**: 5
- **Total tests**: 12

## Dependency Graph

```
aspect-rustc-driver
    ├── aspect-core
    ├── aspect-pointcut
    │   └── aspect-core
    └── rustc_* (nightly)

aspect-std
    └── aspect-core

aspect-macros
    └── aspect-core (dev)

aspect-runtime
    └── aspect-core

aspect-weaver
    ├── aspect-core
    └── syn, quote

aspect-pointcut
    ├── aspect-core
    └── regex, nom

aspect-core
    (no dependencies)
```

## Size and Complexity

| Crate | Lines | Tests | Dependencies | Build Time |
|-------|-------|-------|--------------|------------|
| aspect-core | 800 | 28 | 0 | 2s |
| aspect-macros | 1,200 | 32 | 3 | 8s |
| aspect-runtime | 400 | 18 | 3 | 3s |
| aspect-std | 2,100 | 48 | 2 | 6s |
| aspect-pointcut | 900 | 34 | 3 | 5s |
| aspect-weaver | 700 | 22 | 3 | 5s |
| aspect-rustc-driver | 3,000 | 12 | 20+ | 45s |
| **Total** | **9,100** | **194** | - | **~70s** |

## API Stability

- **aspect-core**: Stable (1.0 ready)
- **aspect-macros**: Stable (1.0 ready)
- **aspect-std**: Stable (expanding)
- **aspect-runtime**: Beta (API refinement)
- **aspect-pointcut**: Beta (syntax may evolve)
- **aspect-weaver**: Alpha (internal API)
- **aspect-rustc-driver**: Alpha (experimental)

## See Also

- [Principles](principles.md) - Core design principles
- [Interactions](interactions.md) - How crates work together
- [Phases](phases.md) - Evolution across phases
- [Extensions](extensions.md) - How to extend the framework
