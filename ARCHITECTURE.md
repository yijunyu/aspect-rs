# Architecture Guide

## Overview

aspect-rs is built as a modular workspace with clear separation of concerns:

```
aspect-rs/
├── aspect-core/      # Core traits and abstractions
├── aspect-macros/    # Procedural macros (#[aspect], #[advice])
├── aspect-runtime/   # Runtime support (registry)
├── aspect-std/       # Standard aspects library
└── aspect-examples/  # Examples and demonstrations
```

## Core Design Principles

1. **Zero Runtime Overhead**: Compile-time weaving via proc macros
2. **Type Safety**: Full Rust type system integration
3. **Thread Safety**: All aspects must be `Send + Sync`
4. **Composability**: Aspects can be combined and ordered
5. **Extensibility**: Easy to create custom aspects

## Component Responsibilities

### aspect-core
**Purpose**: Foundation - traits and types

**Key Types**:
- `Aspect` trait - Base trait all aspects implement
- `JoinPoint` - Function execution context
- `ProceedingJoinPoint` - Wrapper for around advice
- `Pointcut` - Pattern matching for functions

**Dependencies**: None (zero-dependency core)

### aspect-macros
**Purpose**: Compile-time aspect weaving

**Macros**:
- `#[aspect(Expr)]` - Apply aspect to individual functions
- `#[advice(...)]` - Register aspect globally with pointcut

**Process**:
1. Parse annotated function
2. Generate wrapper with aspect calls
3. Preserve original function signature

### aspect-runtime
**Purpose**: Global aspect management

**Components**:
- `AspectRegistry` - Thread-safe global registry
- `RegisteredAspect` - Aspect + pointcut + metadata
- Registration and matching logic

### aspect-std
**Purpose**: Production-ready reusable aspects

**Aspects**:
- `LoggingAspect` - Structured logging
- `TimingAspect` - Performance monitoring
- `CachingAspect` - Memoization
- `MetricsAspect` - Call statistics

## Data Flow

### Function Execution with Aspects

```
User Code
    ↓
#[aspect(Logger)]
fn my_function() { }
    ↓
Macro Expansion
    ↓
fn my_function() {
    let aspect = Logger;
    let pjp = ProceedingJoinPoint::new(
        || __original_function(),
        context
    );
    aspect.around(pjp)  // Calls before/proceed/after
}
```

### Pointcut Matching

```
Pointcut Expression
    ↓
Parser → AST
    ↓
Matcher → FunctionInfo
    ↓
Boolean Result
```

## Extension Points

### Creating Custom Aspects

```rust
struct MyAspect;

impl Aspect for MyAspect {
    fn before(&self, ctx: &JoinPoint) {
        // Custom logic
    }
}
```

### Custom Pointcut Patterns

Currently: Extend `pattern.rs` with new pattern types
Future (Phase 3): MIR-based patterns

## Performance Characteristics

- **No-op aspect**: 0 overhead (optimized away)
- **Simple aspect**: <2ns overhead
- **Complex aspect**: Comparable to hand-written code

## Future Architecture (Phase 3)

```
cargo-aspect (CLI)
    ↓
aspect-driver (rustc plugin)
    ↓
MIR Analysis
    ↓
Code Generation
    ↓
Optimized Binary
```

This enables:
- True automatic weaving
- Field access interception
- Call-site matching
- Zero per-function annotations
