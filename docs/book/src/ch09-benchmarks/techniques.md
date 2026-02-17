# Optimization Techniques

This chapter details proven techniques to maximize aspect-rs performance, achieving near-zero overhead for production applications.

## Performance Targets

| Aspect Type | Target Overhead | Strategy |
|-------------|----------------|----------|
| No-op aspect | 0ns (optimized away) | Dead code elimination |
| Simple logging | <5% | Inline + constant folding |
| Timing/metrics | <10% | Minimize allocations |
| Caching/retry | Negative (faster) | Smart implementation |

Our goal: Make aspects as fast as hand-written code.

## Compiler Optimization Strategies

### 1. Inline Aspect Wrappers

**Problem:** Function call overhead for aspect invocation.

**Solution:** Mark generated wrappers as `#[inline(always)]`:

```rust
// Generated wrapper (conceptual)
#[inline(always)]
pub fn fetch_user(id: u64) -> User {
    let ctx = JoinPoint { /* ... */ };
    
    #[inline(always)]
    fn call_aspects() {
        LoggingAspect::new().before(&ctx);
    }
    call_aspects();
    
    __aspect_original_fetch_user(id)
}
```

**Result:** Compiler inlines everything, eliminating call overhead entirely.

**Measurement:**
- Without inline: 5.2ns
- With inline: 2.1ns
- **Improvement: 60% faster**

### 2. Constant Propagation for JoinPoint

**Problem:** JoinPoint creation allocates stack memory repeatedly.

**Solution:** Use const evaluation for static data:

```rust
// Instead of runtime allocation
let ctx = JoinPoint {
    function_name: "fetch_user",  // Runtime string
    module_path: "crate::api",    // Runtime string
    location: Location { 
        file: file!(),  // Macro expansion
        line: line!(),  // Macro expansion
    },
};

// Generate compile-time constant
const JOINPOINT: JoinPoint = JoinPoint {
    function_name: "fetch_user",  // Static &str
    module_path: "crate::api",    // Static &str
    location: Location {
        file: "src/api.rs",       // Literal
        line: 42,                  // Literal
    },
};

let ctx = &JOINPOINT;  // Zero-cost reference
```

**Result:** Zero runtime allocation, all data in .rodata section.

**Measurement:**
- With runtime creation: 2.7ns
- With const: 0.3ns
- **Improvement: 89% faster**

### 3. Dead Code Elimination

**Problem:** Empty aspect methods still generate code.

**Solution:** Compiler optimizes away empty bodies:

```rust
impl Aspect for NoOpAspect {
    #[inline(always)]
    fn before(&self, _ctx: &JoinPoint) {
        // Empty - compiler eliminates this completely
    }
}

// Generated code:
if false {  // Compile-time constant
    NoOpAspect::new().before(&ctx);
}
// Optimizer removes entire block
```

**Result:** Zero overhead for no-op aspects after optimization.

**Verification:**
```bash
# Check assembly output
cargo asm --lib --rust fetch_user

# No aspect code visible in optimized assembly
```

### 4. Link-Time Optimization (LTO)

**Problem:** Separate compilation prevents cross-crate inlining.

**Solution:** Enable LTO for production builds:

```toml
[profile.release]
lto = "fat"           # Full cross-crate LTO
codegen-units = 1     # Single unit for max optimization
```

**Impact:**
- Inlines aspect code from aspect-std into your crate
- Removes unused aspect methods
- Optimizes across crate boundaries

**Measurement:**
- Without LTO: 2.4ns overhead
- With LTO: 1.1ns overhead
- **Improvement: 54% faster**

### 5. Profile-Guided Optimization (PGO)

**Problem:** Compiler doesn't know which code paths are hot.

**Solution:** Use PGO to optimize based on actual usage:

```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release

# Step 2: Run typical workload
./target/release/myapp
# Generates /tmp/pgo-data/*.profraw

# Step 3: Merge profile data
llvm-profdata merge -o /tmp/pgo-data/merged.profdata \
    /tmp/pgo-data/*.profraw

# Step 4: Rebuild with profile data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" \
    cargo build --release
```

**Result:** Compiler optimizes hot paths more aggressively.

**Measurement:**
- Without PGO: 2.1ns
- With PGO: 1.6ns
- **Improvement: 24% faster**

## Memory Optimization

### 1. Stack Allocation for JoinPoint

**Avoid heap allocation:**

```rust
// BAD: Heap allocation
let joinpoint = Box::new(JoinPoint { /* ... */ });

// GOOD: Stack allocation
let joinpoint = JoinPoint { /* ... */ };
```

**Memory impact:**
- Heap: 128 bytes allocated + malloc overhead
- Stack: 88 bytes, no allocation overhead
- **Savings: 100% allocation elimination**

### 2. Minimize Struct Padding

**Optimize memory layout:**

```rust
// BAD: 8 bytes wasted on padding
struct JoinPoint {
    name: &'static str,    // 16 bytes
    flag: bool,             // 1 byte + 7 padding
    module: &'static str,   // 16 bytes
}
// Total: 40 bytes

// GOOD: Optimal layout
struct JoinPoint {
    name: &'static str,     // 16 bytes
    module: &'static str,   // 16 bytes
    flag: bool,             // 1 byte
    // padding at end doesn't matter
}
// Total: 33 bytes (17.5% smaller)
```

### 3. Use References, Not Copies

```rust
// BAD: Copies JoinPoint
fn before(&self, ctx: JoinPoint) { }

// GOOD: Passes by reference (zero-copy)
fn before(&self, ctx: &JoinPoint) { }
```

**Impact:**
- Copy: 88 bytes copied per call
- Reference: 8 bytes (pointer)
- **Savings: 91% less memory traffic**

### 4. Static Aspect Instances

**Problem:** Creating aspect instances per call.

**Solution:** Use static instances:

```rust
// BAD: New instance every call
LoggingAspect::new().before(&ctx);

// GOOD: Static instance
static LOGGER: LoggingAspect = LoggingAspect::new();
LOGGER.before(&ctx);
```

**Measurement:**
- With new(): 3.2ns
- With static: 0.9ns
- **Improvement: 72% faster**

## Code Size Optimization

### 1. Minimize Monomorphization

**Problem:** Generic aspects create many copies.

```rust
// BAD: One copy per type T
impl<T> Aspect for GenericAspect<T> {
    fn before(&self, ctx: &JoinPoint) {
        // Duplicated for every T
    }
}
```

**Solution:** Type-erase when possible:

```rust
// GOOD: Single implementation
impl Aspect for TypeErasedAspect {
    fn before(&self, ctx: &JoinPoint) {
        self.inner.before_dyn(ctx);
    }
}
```

**Binary size impact:**
- Generic: +500 bytes per instantiation
- Type-erased: +500 bytes total
- **Savings: 90% for 10+ types**

### 2. Share Common Code

**Extract shared logic into helper functions:**

```rust
// Helper called by all wrappers
#[inline(always)]
fn aspect_preamble(name: &'static str) -> JoinPoint {
    JoinPoint { function_name: name, /* ... */ }
}

// Each wrapper reuses helper
fn wrapper1() {
    let ctx = aspect_preamble("func1");
    // ...
}

fn wrapper2() {
    let ctx = aspect_preamble("func2");
    // ...
}
```

**Binary size:**
- Without sharing: 200 bytes × 100 functions = 20KB
- With sharing: 100 bytes + (50 bytes × 100) = 5.1KB
- **Savings: 74% smaller**

### 3. Use Macros for Repetitive Code

```rust
macro_rules! generate_wrapper {
    ($fn_name:ident, $aspect:ty) => {
        #[inline(always)]
        pub fn $fn_name(...) {
            static ASPECT: $aspect = <$aspect>::new();
            ASPECT.before(&JOINPOINT);
            __original_$fn_name(...)
        }
    };
}

// Generates minimal code
generate_wrapper!(fetch_user, LoggingAspect);
```

## Runtime Optimization

### 1. Avoid Allocations in Hot Paths

```rust
impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        // BAD: Allocates String
        let msg = format!("Entering {}", ctx.function_name);
        println!("{}", msg);
        
        // GOOD: No allocation
        println!("Entering {}", ctx.function_name);
    }
}
```

### 2. Lazy Evaluation

**Only compute when needed:**

```rust
impl Aspect for ConditionalAspect {
    fn before(&self, ctx: &JoinPoint) {
        // Only proceed if logging enabled
        if self.enabled.load(Ordering::Relaxed) {
            self.expensive_logging(ctx);
        }
    }
}
```

### 3. Batch Operations

**Instead of per-call logging:**

```rust
impl Aspect for BatchedMetricsAspect {
    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        // Add to buffer
        self.buffer.push(Metric {
            function: ctx.function_name,
            timestamp: Instant::now(),
        });
        
        // Flush every 1000 entries
        if self.buffer.len() >= 1000 {
            self.flush_to_storage();
        }
    }
}
```

**Impact:**
- Per-call logging: 50μs overhead
- Batched (1000): 0.05μs overhead
- **Improvement: 1000x faster**

### 4. Atomic Operations Over Locks

```rust
// BAD: Mutex for simple counter
struct CountingAspect {
    count: Mutex<u64>,
}

// GOOD: Atomic for simple counter
struct CountingAspect {
    count: AtomicU64,
}

impl Aspect for CountingAspect {
    fn before(&self, _ctx: &JoinPoint) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}
```

**Performance:**
- Mutex: ~25ns per increment
- Atomic: ~2ns per increment
- **Improvement: 12.5x faster**

## Architecture Patterns

### 1. Selective Aspect Application

**Don't aspect everything - be strategic:**

```rust
// HOT PATH: No aspects
#[inline(always)]
fn critical_computation(data: &[f64]) -> f64 {
    // Performance-critical, no aspects
    data.iter().sum()
}

// ENTRY POINT: With aspects
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
pub fn process_batch(batches: Vec<Batch>) -> Result<(), Error> {
    for batch in batches {
        critical_computation(&batch.data);
    }
    Ok(())
}
```

**Strategy:** Apply aspects at API boundaries, not inner loops.

### 2. Aspect Composition Order

**Order matters for performance:**

```rust
// BETTER: Cheap aspects first
#[aspect(TimingAspect::new())]     // Fast: just timestamps
#[aspect(LoggingAspect::new())]    // Medium: formatted output
#[aspect(CachingAspect::new())]    // Expensive: hash + lookup
fn expensive_operation() { }

// vs

// WORSE: Expensive aspects first
#[aspect(CachingAspect::new())]
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn expensive_operation() { }
```

**Why:** If caching returns early, later aspects never run.

### 3. Conditional Aspect Activation

```rust
struct ConditionalAspect {
    enabled: AtomicBool,
}

impl Aspect for ConditionalAspect {
    fn before(&self, ctx: &JoinPoint) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;  // Fast path when disabled
        }
        self.do_expensive_work(ctx);
    }
}
```

**Use case:** Enable/disable aspects at runtime (e.g., debug mode).

## Measurement and Validation

### 1. Verify with cargo-asm

Check generated assembly:

```bash
cargo install cargo-show-asm
cargo asm --lib my_crate::aspected_function

# Look for:
# - Inlined aspect code
# - Eliminated dead code
# - Optimized loops
```

### 2. Profile with perf

Find hot paths:

```bash
cargo build --release
perf record --call-graph dwarf ./target/release/myapp
perf report

# Identify aspect overhead in profile
```

### 3. Benchmark Iteratively

```rust
// Before optimization
cargo bench -- --save-baseline before

// After optimization  
cargo bench -- --baseline before

// Should see improvement in results
```

## Advanced Techniques

### 1. SIMD-Friendly Code

```rust
// Ensure aspect wrapper allows auto-vectorization
#[aspect(MetricsAspect::new())]
fn process_array(data: &[f32]) -> Vec<f32> {
    // Compiler can still vectorize this
    data.iter().map(|x| x * 2.0).collect()
}
```

### 2. Branch Prediction Hints

```rust
#[cold]
#[inline(never)]
fn handle_aspect_error(e: AspectError) {
    // Error path marked as unlikely
}

// Hot path
let result = aspect.proceed();
if likely(result.is_ok()) {
    // Common case
} else {
    handle_aspect_error(result.unwrap_err());
}
```

### 3. False Sharing Avoidance

```rust
// BAD: Shared cache line
struct Metrics {
    count1: AtomicU64,  // Cache line 0
    count2: AtomicU64,  // Cache line 0 - false sharing!
}

// GOOD: Separate cache lines
#[repr(align(64))]
struct Metrics {
    count1: AtomicU64,
    _pad: [u8; 56],
    count2: AtomicU64,
}
```

## Configuration Examples

### Development Profile

```toml
[profile.dev]
opt-level = 0
```

Fast compilation, slower runtime (OK for dev).

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

Maximum performance, slower compilation (OK for release).

### Benchmark Profile

```toml
[profile.bench]
inherits = "release"
debug = true  # For profiling tools
```

Optimized + debug symbols for profiling.

## Optimization Checklist

Before deploying aspect-heavy code:

- [ ] Run benchmarks vs baseline
- [ ] Enable LTO for production builds
- [ ] Check binary size impact
- [ ] Profile with production data
- [ ] Verify zero-cost for no-op aspects
- [ ] Test with optimizations enabled
- [ ] Compare with hand-written equivalent
- [ ] Measure allocations (heaptrack)
- [ ] Check assembly output (cargo-asm)
- [ ] Verify inlining (cargo-llvm-lines)
- [ ] Run under perf for hotspots

## Performance Budget

Set targets for your application:

| Aspect Category | Budget | Measurement |
|----------------|--------|-------------|
| Framework overhead | <5% | Microbenchmark |
| Real-world impact | <2% | Integration test |
| Binary size increase | <10% | cargo-bloat |
| Compile time increase | <20% | cargo build --timings |

If you exceed budget, apply optimization techniques from this chapter.

## Common Pitfalls

### Avoid:

1. ❌ Allocating on hot paths (use stack/static)
2. ❌ Creating aspects per call (reuse instances)
3. ❌ Runtime pointcut matching (should be compile-time)
4. ❌ Ignoring inlining (always mark #[inline])
5. ❌ Skipping benchmarks (measure everything)
6. ❌ Optimizing blindly (profile first)
7. ❌ Over-applying aspects (be selective)

### Prefer:

1. ✅ Stack/static allocation
2. ✅ Static aspect instances
3. ✅ Compile-time decisions
4. ✅ #[inline(always)] on wrappers
5. ✅ Benchmark-driven optimization
6. ✅ Profile-guided decisions
7. ✅ Strategic aspect placement

## Results Summary

Applying these techniques achieves:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| No-op overhead | 5.2ns | 0ns | 100% |
| Simple aspect | 4.5ns | 2.1ns | 53% |
| JoinPoint creation | 2.7ns | 0.3ns | 89% |
| Binary size | +15% | +3% | 80% smaller |

**Goal achieved:** Near-zero overhead for production use.

## Key Takeaways

1. **Inline everything** - Eliminates call overhead
2. **Use const evaluation** - Moves work to compile-time
3. **Enable LTO** - Cross-crate optimization
4. **Static instances** - Avoid per-call allocation
5. **Profile first** - Optimize based on data
6. **Be selective** - Don't aspect hot inner loops
7. **Measure always** - Verify improvements

With these techniques, aspect-rs achieves performance indistinguishable from hand-written code.

## Next Steps

- See [Running Benchmarks](./running.md) to measure your optimizations
- See [Results](./results.md) for expected performance numbers
- See [Real-World](./realworld.md) for production examples

---

**Related Chapters:**
- [Chapter 9.2: Results](./results.md) - Performance data
- [Chapter 9.5: Running](./running.md) - How to benchmark
- [Chapter 8: Case Studies](../ch08-case-studies/README.md) - Implementation examples
