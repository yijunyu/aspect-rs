# Benchmark Methodology

This chapter describes the rigorous methodology used to measure the performance characteristics of the aspect-rs framework. Understanding how we benchmark ensures you can trust the results and reproduce them yourself.

## Overview

Performance benchmarking for aspect-oriented programming frameworks requires careful measurement to separate:

- **Aspect overhead** from business logic execution time
- **Compile-time costs** from runtime costs  
- **Framework overhead** from application complexity
- **Microbenchmark results** from real-world performance

We use industry-standard tools and methodologies to ensure accurate, reproducible results.

## Benchmarking Tools

### Criterion.rs

All benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs), the gold standard for Rust benchmarking:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "aspect_overhead"
harness = false
```

**Why Criterion?**
- Statistical analysis of measurements with outlier detection
- HTML reports with interactive graphs
- Warmup periods to reach stable CPU state
- Automatic comparison against saved baselines
- Confidence intervals and significance testing
- Guards against measurement bias

### Benchmark Structure

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_baseline(c: &mut Criterion) {
    c.bench_function("no_aspect", |b| {
        b.iter(|| {
            baseline_function(black_box(42))
        })
    });
}

fn benchmark_with_aspect(c: &mut Criterion) {
    c.bench_function("with_logging", |b| {
        b.iter(|| {
            aspected_function(black_box(42))
        })
    });
}

criterion_group!(benches, benchmark_baseline, benchmark_with_aspect);
criterion_main!(benches);
```

**Key elements:**
- `black_box()` prevents compiler optimization of benchmarked code
- `bench_function()` runs multiple iterations automatically  
- Statistical analysis determines confidence intervals
- Results include mean, median, standard deviation

## Measurement Categories

### 1. Aspect Overhead

Measures the performance cost of the aspect framework itself:

```rust
// Baseline: no aspects
#[inline(never)]
fn baseline_add(a: i32, b: i32) -> i32 {
    a + b
}

// With no-op aspect  
#[aspect(NoOpAspect)]
#[inline(never)]
fn aspected_add(a: i32, b: i32) -> i32 {
    a + b
}

// Benchmark both
c.bench_function("baseline", |b| b.iter(|| baseline_add(black_box(1), black_box(2))));
c.bench_function("no-op aspect", |b| b.iter(|| aspected_add(black_box(1), black_box(2))));
```

**What we measure:**
- JoinPoint structure allocation and initialization
- Aspect trait virtual method dispatch overhead
- before/after/around advice execution time
- Result boxing and unboxing costs
- Error handling propagation

Expected result: No-op aspect overhead should be <5ns on modern CPUs.

### 2. Component Costs

Isolates individual framework components to identify bottlenecks:

```rust
// Just JoinPoint creation
c.bench_function("joinpoint_creation", |b| {
    b.iter(|| {
        let ctx = JoinPoint {
            function_name: "test",
            module_path: "bench",
            location: Location { file: "bench.rs", line: 10 },
        };
        black_box(ctx);
    })
});

// Just aspect method call
c.bench_function("aspect_before_call", |b| {
    let aspect = LoggingAspect::new();
    let ctx = create_joinpoint();
    
    b.iter(|| {
        aspect.before(black_box(&ctx));
    })
});

// Just ProceedingJoinPoint proceed
c.bench_function("pjp_proceed", |b| {
    b.iter(|| {
        let pjp = create_proceeding_joinpoint(|| Ok(42));
        black_box(pjp.proceed().unwrap());
    })
});
```

This helps us understand where optimization efforts should focus.

### 3. Scaling Behavior  

Tests performance as complexity increases with multiple aspects:

```rust
// 1 aspect
#[aspect(LoggingAspect::new())]
fn one_aspect() { do_work(); }

// 3 aspects
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(MetricsAspect::new())]
fn three_aspects() { do_work(); }

// 5 aspects
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(MetricsAspect::new())]
#[aspect(CachingAspect::new())]
#[aspect(RetryAspect::new(3, 100))]
fn five_aspects() { do_work(); }
```

**Expected results:**
- Linear scaling: O(n) where n = number of aspects
- No quadratic behavior or pathological cases
- Consistent per-aspect overhead (~2-5ns each)

### 4. Real-World Scenarios

Benchmarks that simulate actual production usage patterns:

```rust
// API request simulation
c.bench_function("api_request_baseline", |b| {
    let db = setup_test_database();
    b.iter(|| {
        let request = create_request(black_box(123));
        process_request_baseline(black_box(&db), black_box(request))
    })
});

c.bench_function("api_request_with_aspects", |b| {
    let db = setup_test_database();
    b.iter(|| {
        let request = create_request(black_box(123));
        process_request_with_aspects(black_box(&db), black_box(request))
    })
});
```

These scenarios include realistic I/O, database operations, and business logic complexity.

## Benchmark Configurations

### Compiler Optimization Flags

All benchmarks run with production-level optimizations:

```toml
[profile.bench]
opt-level = 3           # Maximum optimization
lto = "fat"            # Link-time optimization across all crates
codegen-units = 1      # Better optimization (slower compile)
panic = "abort"        # Smaller code, faster unwinding
```

**Rationale:**
- `opt-level = 3`: Enables all LLVM optimizations
- `lto = "fat"`: Allows cross-crate inlining of aspect code
- `codegen-units = 1`: Gives optimizer maximum visibility
- `panic = "abort"`: Removes unwinding overhead

This configuration represents how aspect-rs would be deployed in production.

### System Configuration

For reproducible results, benchmarks should run on:

- **CPU**: Modern x86_64 processor (2+ GHz, consistent clock)
- **RAM**: 8+ GB available
- **OS**: Linux (Ubuntu 22.04 LTS) or macOS (latest)
- **System Load**: Minimal background processes

**Preparing system for benchmarking:**
```bash
# Disable CPU frequency scaling (Linux)
sudo cpupower frequency-set --governor performance

# Stop unnecessary services
sudo systemctl stop bluetooth cups avahi-daemon

# Clear system caches
sudo sync && echo 3 | sudo tee /proc/sys/vm/drop_caches

# Verify no other heavy processes
htop  # Should show <10% CPU usage at idle

# Run benchmarks
cargo bench --workspace
```

### Statistical Rigor

Criterion automatically provides robust statistics:

- **Warmup**: 3 seconds to reach stable CPU state
- **Sample size**: 100 samples minimum
- **Iterations**: 10,000+ per sample (adjusted for duration)
- **Outlier detection**: Modified Thompson Tau test
- **Confidence intervals**: 95% by default
- **Significance testing**: Student's t-test (p < 0.05)

**Example output with interpretation:**
```
no_aspect               time:   [2.1234 ns 2.1456 ns 2.1678 ns]
                        change: [-0.5123% +0.1234% +0.7890%] (p = 0.23 > 0.05)
                        No change in performance detected.

with_logging            time:   [2.2345 ns 2.2567 ns 2.2789 ns]
                        change: [-0.3456% +0.2345% +0.8901%] (p = 0.34 > 0.05)
                        No change in performance detected.

Calculated overhead: 0.1111 ns (5.18% increase)
95% confidence interval: [4.85%, 5.51%]
```

The three time values represent: [lower bound, estimate, upper bound] of the 95% confidence interval.

## Controlling Variables

### Preventing Compiler Optimization

The Rust compiler is highly intelligent and may optimize away benchmarked code:

```rust
// BAD: Compiler might optimize away unused result
fn bad_benchmark(c: &mut Criterion) {
    c.bench_function("bad", |b| {
        b.iter(|| {
            aspected_function(42)  // Result unused, may be eliminated!
        })
    });
}

// GOOD: Use black_box to prevent optimization
fn good_benchmark(c: &mut Criterion) {
    c.bench_function("good", |b| {
        b.iter(|| {
            black_box(aspected_function(black_box(42)))
        })
    });
}
```

**Why this matters:**
- Without `black_box()`, compiler may inline and optimize away entire function
- Could measure 0ns when actual code takes nanoseconds
- Results would be misleading and non-representative

### Avoiding Measurement Noise

Common sources of noise in benchmarks:

1. **CPU throttling**: Use performance governor, not powersave
2. **Background processes**: Close browsers, IDEs, chat apps
3. **Network activity**: Disable WiFi/Ethernet during benchmarks
4. **Disk I/O**: Use tmpfs (`/dev/shm`) for temporary files
5. **System updates**: Disable auto-updates temporarily
6. **Thermal throttling**: Ensure adequate cooling
7. **Turbo boost**: Can cause inconsistent results; disable if needed

### Isolation with `#[inline(never)]`

Prevents cross-function optimization for fair comparison:

```rust
#[inline(never)]
fn baseline_function(x: i32) -> i32 {
    x * 2
}

#[aspect(LoggingAspect::new())]
#[inline(never)]
fn aspected_function(x: i32) -> i32 {
    x * 2
}
```

This ensures:
- Each function is compiled as a separate unit
- No inlining across benchmark boundaries
- Fair comparison of actual runtime costs
- Results reflect real-world function call overhead

## Baseline Comparison Methodology

### Manual vs Aspect-Based Implementation

Critical comparison: aspect framework vs hand-written equivalent:

```rust
// Manual logging (baseline - what developers write without aspects)
#[inline(never)]
fn manual_logging(x: i32) -> i32 {
    println!("[ENTRY] manual_logging");
    let result = x * 2;
    println!("[EXIT] manual_logging");
    result
}

// Aspect-based logging (what aspect-rs provides)
#[aspect(LoggingAspect::new())]
#[inline(never)]
fn aspect_logging(x: i32) -> i32 {
    x * 2
}

// Benchmark both approaches
c.bench_function("manual_logging", |b| {
    b.iter(|| manual_logging(black_box(42)))
});

c.bench_function("aspect_logging", |b| {
    b.iter(|| aspect_logging(black_box(42)))
});
```

**Success criteria:** Aspect overhead should be <5% compared to manual implementation.

If overhead exceeds 10%, we investigate and optimize the framework.

## Benchmark Organization

### Microbenchmarks

Located in `aspect-core/benches/`:

```
aspect-core/benches/
├── aspect_overhead.rs      # Basic aspect overhead measurement
├── joinpoint_creation.rs   # JoinPoint allocation cost
├── advice_dispatch.rs      # Virtual method dispatch timing
├── multiple_aspects.rs     # Scaling with aspect count
├── around_advice.rs        # ProceedingJoinPoint overhead
└── error_handling.rs       # AspectError propagation cost
```

Each file focuses on one specific performance aspect.

### Integration Benchmarks

Located in `aspect-examples/benches/`:

```
aspect-examples/benches/
├── api_server_bench.rs     # Full API request/response cycle
├── database_bench.rs       # Transaction aspect overhead
├── security_bench.rs       # Authorization check performance
├── resilience_bench.rs     # Retry/circuit breaker costs
└── caching_bench.rs        # Cache lookup/store overhead
```

These measure realistic, end-to-end scenarios.

### Regression Detection

Using saved baselines to detect performance regressions:

```bash
# Save baseline from main branch
git checkout main
cargo bench --workspace -- --save-baseline main

# Switch to feature branch
git checkout feature/new-optimization
cargo bench --workspace -- --baseline main
```

**Criterion output:**
```
no_aspect               time:   [2.1456 ns 2.1678 ns 2.1890 ns]
                        change: [-1.2% +0.5% +2.1%] (p = 0.42 > 0.05)
                        No significant change detected.

with_logging            time:   [2.3456 ns 2.3678 ns 2.3890 ns]
                        change: [+8.2% +9.5% +10.8%] (p = 0.001 < 0.05)
                        Performance has regressed.
```

A regression >5% triggers investigation before merge.

## Metrics Collected

### Primary Performance Metrics

1. **Mean execution time** - Average across all samples
2. **Median execution time** - Middle value (robust against outliers)
3. **Standard deviation** - Measure of variance
4. **Min/Max** - Best and worst case timings

### Secondary Metrics

5. **Memory allocations** - Tracked via `dhat` profiler
6. **Binary size** - Measured via `cargo-bloat`
7. **Compile time** - Via `cargo build --timings`
8. **LLVM IR size** - Via `cargo-llvm-lines`

### Derived Metrics

9. **Overhead percentage**: `(aspect_time - baseline_time) / baseline_time * 100`
10. **Per-aspect cost**: `total_overhead / number_of_aspects`
11. **Throughput**: Operations per second

## Interpreting Results

### Statistical Significance

Criterion uses Student's t-test with threshold p < 0.05:

- **p < 0.05**: Change is statistically significant
- **p ≥ 0.05**: Change is within noise/variance

**Example interpretation:**
```
time:   [2.2567 ns 2.2789 ns 2.3012 ns]
change: [+5.12% +5.45% +5.78%] (p = 0.002 < 0.05)
Performance has regressed.
```

This indicates a true regression, not measurement noise.

### Acceptable Variance

Normal variance in nanosecond-level microbenchmarks:

- **0-2%**: Excellent stability
- **2-5%**: Good stability (typical for microbenchmarks)
- **5-10%**: Acceptable (environmental factors)
- **>10%**: Investigate (possible actual regression or system issue)

### Regression Investigation Thresholds

When performance degrades:

- **<3% slower**: Likely noise; monitor trend
- **3-5% slower**: Verify across multiple runs
- **5-10% slower**: Worth investigating cause
- **>10% slower**: Definite regression; requires fix before merge
- **>25% slower**: Critical regression; blocks PR immediately

## Best Practices

### DO:

1. ✅ Use `black_box()` for all inputs and outputs
2. ✅ Run on dedicated hardware when possible
3. ✅ Use `#[inline(never)]` for fair comparison
4. ✅ Benchmark realistic workloads, not just microbenchmarks
5. ✅ Save baselines for regression detection
6. ✅ Run benchmarks multiple times to verify stability
7. ✅ Document system configuration and environment
8. ✅ Compare against hand-written alternatives
9. ✅ Use appropriate sample sizes (100+ samples)
10. ✅ Warm up before measuring

### DON'T:

1. ❌ Run benchmarks on laptop battery power
2. ❌ Run with heavy background processes active
3. ❌ Compare debug vs release builds
4. ❌ Trust single-run results
5. ❌ Ignore compiler warnings about dead code elimination
6. ❌ Benchmark without `black_box()` protection
7. ❌ Compare results from different machines directly
8. ❌ Cherry-pick favorable results

## Reproducibility

### Version Control

All benchmark code is version controlled:

```
aspect-rs/
├── aspect-core/benches/       # Framework benchmarks
├── aspect-examples/benches/   # Application benchmarks
├── BENCHMARKS.md             # Results documentation
└── benches/README.md         # Running instructions
```

### Running Benchmarks

Anyone can reproduce our results:

```bash
# Clone repository
git clone https://github.com/user/aspect-rs
cd aspect-rs

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run all benchmarks
cargo bench --workspace

# View detailed HTML reports
open target/criterion/report/index.html

# Or view specific benchmark
open target/criterion/aspect_overhead/report/index.html
```

### Sharing Results

Criterion generates multiple output formats:

- **HTML**: Interactive charts and detailed statistics
- **JSON**: Raw data in `target/criterion/<bench>/base/estimates.json`
- **CSV**: Can be exported for spreadsheet analysis

```bash
# Generate comparison report
cargo bench --workspace -- --baseline previous

# Export results
cp -r target/criterion/report benchmark-results-2024-02-16/
```

## Validation

### Cross-Platform Testing

We run benchmarks on multiple platforms to ensure consistency:

- **Linux**: Ubuntu 22.04 LTS (x86_64)
- **macOS**: Latest version (ARM64 M1/M2)
- **Windows**: Windows 11 (x86_64)

Overhead percentages should be similar across platforms (within 2-3%).

### Manual Verification

Spot-check Criterion results with manual timing:

```rust
use std::time::Instant;

fn manual_timing() {
    let iterations = 10_000_000;

    // Baseline timing
    let start = Instant::now();
    for i in 0..iterations {
        black_box(baseline_function(black_box(i as i32)));
    }
    let baseline_time = start.elapsed();

    // With aspect timing
    let start = Instant::now();
    for i in 0..iterations {
        black_box(aspected_function(black_box(i as i32)));
    }
    let aspect_time = start.elapsed();

    let baseline_ns = baseline_time.as_nanos() / iterations as u128;
    let aspect_ns = aspect_time.as_nanos() / iterations as u128;
    
    println!("Baseline: {} ns", baseline_ns);
    println!("With aspect: {} ns", aspect_ns);
    println!("Overhead: {:.2}%",
        (aspect_ns as f64 - baseline_ns as f64) / baseline_ns as f64 * 100.0
    );
}
```

Results should match Criterion within ±10%.

## Key Takeaways

1. **Criterion.rs provides statistical rigor** - Use it for all benchmarks
2. **Control variables carefully** - Minimize environmental noise
3. **Prevent unwanted optimization** - Use `black_box()` and `#[inline(never)]`
4. **Compare fairly** - Benchmark against equivalent hand-written code
5. **Save baselines** - Enable regression detection over time
6. **Run multiple times** - Verify stability and reproducibility
7. **Document everything** - Record system config, compiler flags, environment
8. **Validate results** - Cross-check on multiple platforms

Understanding methodology builds confidence in results. When you see "5% overhead", you know exactly what that means and how it was measured.

## Next Steps

- See [Benchmark Results](./results.md) for actual measured performance data
- See [Real-World Performance](./realworld.md) for production scenarios
- See [Optimization Techniques](./techniques.md) for improving performance
- See [Running Benchmarks](./running.md) for step-by-step execution guide

---

**Related Chapters:**
- [Chapter 8: Case Studies](../ch08-case-studies/README.md) - Real-world examples
- [Chapter 9.2: Results](./results.md) - Measured performance data
- [Chapter 9.5: Running](./running.md) - How to execute benchmarks
