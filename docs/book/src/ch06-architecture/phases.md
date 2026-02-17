# Evolution Across Phases

aspect-rs was developed in three phases, each building on the previous with increasing power and automation. This chapter compares the phases and explains the evolution.

## Phase Overview

| Phase | Status | Approach | Annotation | Automation |
|-------|--------|----------|------------|------------|
| Phase 1 | ✅ Complete | Proc macros | `#[aspect]` required | Manual |
| Phase 2 | ✅ Complete | Pointcuts + Registry | `#[advice]` optional | Semi-automatic |
| Phase 3 | ✅ Complete | MIR weaving | None required | Fully automatic |

## Phase 1: Basic Infrastructure

### Goal

Establish foundational AOP capabilities in Rust with minimal complexity.

### Features

- **Core trait**: `Aspect` trait with before/after/around advice
- **JoinPoint**: Execution context with metadata
- **Proc macro**: `#[aspect(Expr)]` attribute
- **Manual application**: Explicit annotation on each function

### Example

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;

struct Logger;

impl Aspect for Logger {
    fn before(&self, ctx: &JoinPoint) {
        println!("[ENTRY] {}", ctx.function_name);
    }
}

// Manual annotation required
#[aspect(Logger)]
fn fetch_user(id: u64) -> User {
    database::get(id)
}

#[aspect(Logger)]  // Must repeat for each function
fn save_user(user: User) -> Result<()> {
    database::save(user)
}
```

### Strengths

- **Simple**: Easy to understand and implement
- **Explicit**: Clear what functions have aspects
- **Zero dependencies**: aspect-core has no dependencies
- **Fast compilation**: Minimal code generation

### Limitations

- **Repetitive**: Must annotate every function
- **Error-prone**: Easy to forget annotations
- **Not scalable**: Tedious for large codebases
- **Limited patterns**: Can't apply based on patterns

### Implementation

**Crates**: 3
- aspect-core (traits)
- aspect-macros (#[aspect])
- aspect-std (standard aspects)

**Lines of Code**: ~4,000
**Tests**: 108
**Build Time**: ~15 seconds

## Phase 2: Production-Ready

### Goal

Add declarative aspect application with pointcut patterns and global registry.

### Features

- **Pointcut expressions**: Pattern matching for functions
- **Global registry**: Centralized aspect management
- **#[advice] macro**: Register aspects with pointcuts
- **Boolean combinators**: AND, OR, NOT for pointcuts
- **Aspect ordering**: Control execution order

### Example

```rust
use aspect_macros::advice;

// Register aspect with pointcut pattern
#[advice(
    pointcut = "execution(pub fn *(..)) && within(crate::api)",
    advice = "around",
    order = 10
)]
fn api_logger(pjp: ProceedingJoinPoint)
    -> Result<Box<dyn Any>, AspectError>
{
    println!("[API] {}", pjp.context().function_name);
    pjp.proceed()
}

// No annotations needed on functions!
pub fn fetch_user(id: u64) -> User {
    database::get(id)  // Automatically gets logging
}

pub fn save_user(user: User) -> Result<()> {
    database::save(user)  // Automatically gets logging
}
```

### Pointcut Patterns

```rust
// Match all public functions
execution(pub fn *(..))

// Match functions in api module
within(crate::api)

// Match functions with specific names
name(fetch_* | save_*)

// Combine with boolean logic
execution(pub fn *(..)) && within(crate::api) && !name(test_*)
```

### Strengths

- **Declarative**: Define once, apply everywhere
- **Pattern-based**: Flexible matching rules
- **Composable**: Multiple aspects with ordering
- **Maintainable**: Easy to add/remove aspects

### Limitations

- **Still needs registration**: Must use #[advice] somewhere
- **Compile-time only**: Can't change aspects at runtime
- **Limited to function-level**: Can't intercept field access

### Implementation

**Crates**: 5
- Previous 3 crates
- aspect-pointcut (pattern matching)
- aspect-runtime (global registry)

**Lines of Code**: ~6,000
**Tests**: 142
**Build Time**: ~25 seconds

## Phase 3: Automatic Weaving

### Goal

Achieve AspectJ-style automatic weaving with zero annotations via rustc integration.

### Features

- **MIR analysis**: Extract functions from compiled code
- **Automatic matching**: Apply pointcuts without annotations
- **rustc integration**: Custom compiler driver
- **Zero annotations**: Completely annotation-free
- **Command-line config**: Aspects configured via CLI

### Example

```rust
// User code - NO ANNOTATIONS AT ALL!
pub fn fetch_user(id: u64) -> User {
    database::get(id)
}

pub fn save_user(user: User) -> Result<()> {
    database::save(user)
}

fn internal_helper() -> i32 {
    42
}
```

**Compilation**:

```bash
# Apply logging to all public functions
aspect-rustc-driver \
    --aspect-pointcut "execution(pub fn *(..))" \
    --aspect-type "LoggingAspect" \
    src/main.rs --crate-type lib
```

**Result**:

```
✅ Extracted 3 functions from MIR
✅ Matched 2 public functions:
   - fetch_user
   - save_user
✅ Applied LoggingAspect automatically
```

### 6-Step Pipeline

1. **Parse CLI**: Extract pointcut expressions from command line
2. **Configure compiler**: Set up custom rustc callbacks
3. **Access TyCtxt**: Get compiler's type context
4. **Extract MIR**: Analyze mid-level IR for function metadata
5. **Match pointcuts**: Apply pattern matching automatically
6. **Generate code**: Weave aspects without annotations

### Strengths

- **Zero boilerplate**: No annotations in code
- **Centralized config**: All aspects in one place
- **Impossible to forget**: Can't miss applying aspects
- **True AOP**: Matches AspectJ capabilities
- **Still zero-cost**: Compile-time weaving preserved

### Limitations

- **Requires nightly**: Uses unstable rustc APIs
- **Complex build**: Custom compiler driver
- **Longer compilation**: MIR analysis adds time

### Implementation

**Crates**: 7
- Previous 5 crates
- aspect-weaver (advanced code generation)
- aspect-rustc-driver (rustc integration)

**Lines of Code**: ~9,100
**Tests**: 194
**Build Time**: ~70 seconds (including rustc)

## Comparison Matrix

### Feature Comparison

| Feature | Phase 1 | Phase 2 | Phase 3 |
|---------|---------|---------|---------|
| Annotation required | ✅ Always | ⚠️ Optional | ❌ Never |
| Pointcut patterns | ❌ | ✅ | ✅ |
| Global registry | ❌ | ✅ | ✅ |
| Aspect ordering | ⚠️ Via nesting | ✅ Explicit | ✅ Explicit |
| MIR analysis | ❌ | ❌ | ✅ |
| Automatic matching | ❌ | ⚠️ Semi | ✅ Full |
| Compile-time only | ✅ | ✅ | ✅ |
| Zero overhead | ✅ | ✅ | ✅ |
| Stable Rust | ✅ | ✅ | ❌ Nightly |
| Build time | Fast | Medium | Slower |
| Learning curve | Low | Medium | Medium |

### Use Case Recommendations

**Choose Phase 1 when:**
- Learning AOP in Rust
- Small codebase (<1000 functions)
- Explicit control desired
- Stable Rust required
- Fast iteration needed

**Choose Phase 2 when:**
- Medium/large codebase
- Pattern-based application desired
- Multiple aspects needed
- Aspect ordering important
- Stable Rust required

**Choose Phase 3 when:**
- Annotation-free code desired
- Maximum automation needed
- Large existing codebase
- Nightly Rust acceptable
- Production deployment (after testing)

## Migration Path

### Phase 1 → Phase 2

**Before (Phase 1)**:

```rust
#[aspect(Logger)]
fn fetch_user(id: u64) -> User { ... }

#[aspect(Logger)]
fn save_user(user: User) -> Result<()> { ... }

#[aspect(Logger)]
fn delete_user(id: u64) -> Result<()> { ... }
```

**After (Phase 2)**:

```rust
// Register once
#[advice(
    pointcut = "execution(pub fn *_user(..))",
    advice = "around"
)]
fn user_logger(pjp: ProceedingJoinPoint) { ... }

// Functions are automatically matched
fn fetch_user(id: u64) -> User { ... }
fn save_user(user: User) -> Result<()> { ... }
fn delete_user(id: u64) -> Result<()> { ... }
```

**Benefits**:
- 67% less boilerplate (1 annotation vs 3)
- Centralized aspect management
- Easier to modify aspect rules

### Phase 2 → Phase 3

**Before (Phase 2)**:

```rust
#[advice(pointcut = "execution(pub fn *(..))", ...)]
fn logger(pjp: ProceedingJoinPoint) { ... }

pub fn fetch_user(id: u64) -> User { ... }
```

**After (Phase 3)**:

```rust
// Code remains unchanged - no annotations!
pub fn fetch_user(id: u64) -> User { ... }
```

**Build command**:

```bash
# Instead of: cargo build
aspect-rustc-driver \
    --aspect-pointcut "execution(pub fn *(..))" \
    --aspect-type "LoggingAspect" \
    main.rs
```

**Benefits**:
- 100% annotation-free
- Build configuration instead of code annotations
- Can change aspects without touching code

## Timeline

### Development

| Phase | Duration | Effort | Milestone |
|-------|----------|--------|-----------|
| Phase 1 | Weeks 1-4 | 4,000 LOC | Basic AOP working |
| Phase 2 | Weeks 5-8 | +2,000 LOC | Pointcuts working |
| Phase 3 | Weeks 9-14 | +3,100 LOC | MIR weaving complete |

**Total**: 14 weeks, 9,100 lines of code, 194 tests

### Testing

| Phase | Tests | Coverage | Status |
|-------|-------|----------|--------|
| Phase 1 | 108 | 85% | ✅ All passing |
| Phase 2 | 142 | 82% | ✅ All passing |
| Phase 3 | 194 | 78% | ✅ All passing |

## Performance Across Phases

| Metric | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|---------|
| No-op aspect overhead | 0ns | 0ns | 0ns |
| Simple aspect overhead | ~2% | ~2% | ~2% |
| Code size increase | ~5% | ~8% | ~8% |
| Compile time increase | +10% | +25% | +50% |
| Runtime overhead | 0% | 0% | 0% |

**All phases achieve zero runtime overhead!**

## Architectural Evolution

### Phase 1 Architecture

```
User Code
    ↓
#[aspect] macro
    ↓
Code generation
    ↓
Compiled binary
```

**Simple linear flow.**

### Phase 2 Architecture

```
User Code (#[advice] registrations)
    ↓
aspect-runtime registry
    ↓
#[aspect] macro OR automatic weaving
    ↓
Pointcut matching
    ↓
Code generation
    ↓
Compiled binary
```

**Added registry and pattern matching.**

### Phase 3 Architecture

```
User Code (no annotations)
    ↓
aspect-rustc-driver
    ↓
rustc compilation
    ↓
MIR extraction
    ↓
Pointcut matching
    ↓
Automatic code weaving
    ↓
Optimized binary
```

**Fully integrated with compiler.**

## Future Phases

### Potential Phase 4: Runtime AOP

**Concept**: Dynamic aspect application

**Features**:
- Load aspects at runtime
- Modify aspects without recompilation
- JIT aspect weaving
- Hot-reload aspects

**Challenges**:
- Runtime overhead inevitable
- Type safety harder to guarantee
- Performance impact

**Status**: Research stage

## See Also

- [Crate Organization](crates.md) - How crates evolved
- [Principles](principles.md) - Preserved across all phases
- [Phase 3 Details](../ch10-phase3/) - Complete Phase 3 guide
- [Migration Guide](../ch05-usage-guide/patterns.md) - Practical migration
