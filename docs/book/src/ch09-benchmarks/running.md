# Running Benchmarks

This chapter provides step-by-step instructions for running aspect-rs benchmarks and interpreting results.

## Quick Start

```bash
# Clone repository
git clone https://github.com/user/aspect-rs
cd aspect-rs

# Run all benchmarks
cargo bench --workspace

# View HTML reports
open target/criterion/report/index.html
```

That's it! Criterion will run all benchmarks and generate detailed reports.

## Prerequisites

### System Requirements

- **OS**: Linux, macOS, or Windows
- **Rust**: 1.70+ (stable)
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 2GB for build artifacts

### Installing Rust

```bash
# If Rust not installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Installing Criterion

Criterion is included as a dev-dependency, so no separate installation needed.

## Running Benchmarks

### All Benchmarks

```bash
# Run everything (takes 5-10 minutes)
cargo bench --workspace
```

**Output:**
```
Benchmarking no_aspect: Collecting 100 samples in estimated 5.0000 s
no_aspect               time:   [2.1234 ns 2.1456 ns 2.1678 ns]

Benchmarking with_logging: Collecting 100 samples in estimated 5.0000 s  
with_logging            time:   [2.2345 ns 2.2567 ns 2.2789 ns]
                        change: [+4.89% +5.18% +5.47%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

### Specific Benchmarks

```bash
# Run aspect overhead benchmarks only
cargo bench -p aspect-core --bench aspect_overhead

# Run API server benchmarks
cargo bench -p aspect-examples --bench api_server_bench

# Run specific benchmark by name
cargo bench --workspace -- aspect_overhead
```

### With Verbose Output

```bash
# See what's being measured
cargo bench --workspace -- --verbose

# Show measurement iterations
cargo bench --workspace -- --verbose --sample-size 10
```

## Benchmark Organization

### Core Framework Benchmarks

Located in `aspect-core/benches/`:

```bash
cargo bench -p aspect-core

# Individual benches:
cargo bench -p aspect-core --bench aspect_overhead
cargo bench -p aspect-core --bench joinpoint_creation  
cargo bench -p aspect-core --bench advice_dispatch
cargo bench -p aspect-core --bench multiple_aspects
```

### Integration Benchmarks

Located in `aspect-examples/benches/`:

```bash
cargo bench -p aspect-examples

# Individual benches:
cargo bench -p aspect-examples --bench api_server_bench
cargo bench -p aspect-examples --bench database_bench
cargo bench -p aspect-examples --bench security_bench
```

## Interpreting Results

### Understanding Output

```
no_aspect               time:   [2.1234 ns 2.1456 ns 2.1678 ns]
                        change: [-0.5123% +0.1234% +0.7890%] (p = 0.23 > 0.05)
                        No change in performance detected.
```

**Breaking it down:**
- `time:   [2.1234 ns 2.1456 ns 2.1678 ns]`
  - First number: Lower bound of 95% confidence interval
  - Middle: Estimated median time
  - Last: Upper bound of 95% confidence interval
  
- `change: [-0.5123% +0.1234% +0.7890%]`
  - Change compared to previous run or baseline
  - Format: [lower, estimate, upper] of confidence interval
  
- `(p = 0.23 > 0.05)`
  - p-value from significance test
  - p < 0.05: Statistically significant change
  - p ≥ 0.05: Change within noise

- `No change in performance detected`
  - Interpretation based on statistical analysis

### Reading HTML Reports

```bash
# Open main report
open target/criterion/report/index.html
```

**Report sections:**
1. **Violin plots**: Distribution of measurements
2. **Iteration times**: Time per iteration over samples
3. **Statistics**: Mean, median, std deviation
4. **Comparison**: vs baseline (if available)

### Comparison Indicators

| Symbol | Meaning |
|--------|---------|
| ✅ Green | Performance improved |
| ❌ Red | Performance regressed |
| ⚪ Gray | No significant change |

## Baseline Comparison

### Saving a Baseline

```bash
# Save current performance as baseline
cargo bench --workspace -- --save-baseline main

# Results saved to: target/criterion/<bench>/main/
```

### Comparing Against Baseline

```bash
# Compare current performance to saved baseline
cargo bench --workspace -- --baseline main
```

**Example output:**
```
with_logging            time:   [2.3456 ns 2.3678 ns 2.3900 ns]
                        change: [+9.23% +10.12% +11.01%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

This shows current performance is ~10% slower than the `main` baseline.

### Multiple Baselines

```bash
# Save different baselines
cargo bench --workspace -- --save-baseline feature-branch
cargo bench --workspace -- --save-baseline optimization-attempt

# Compare against any baseline
cargo bench --workspace -- --baseline feature-branch
cargo bench --workspace -- --baseline optimization-attempt
```

## System Preparation

### Linux

```bash
# Set CPU governor to performance mode
sudo cpupower frequency-set --governor performance

# Disable turbo boost (for consistency)
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Stop unnecessary services
sudo systemctl stop bluetooth cups avahi-daemon

# Clear caches
sync && echo 3 | sudo tee /proc/sys/vm/drop_caches

# Verify CPU speed
cat /proc/cpuinfo | grep MHz
```

### macOS

```bash
# Close unnecessary applications
# Disable Spotlight indexing temporarily
sudo mdutil -a -i off

# Run benchmarks
cargo bench --workspace

# Re-enable Spotlight  
sudo mdutil -a -i on
```

### Windows

```powershell
# Set power plan to High Performance
powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c

# Close background apps
# Disable Windows Defender real-time scanning temporarily

# Run benchmarks
cargo bench --workspace
```

## Configuration Options

### Sample Size

```bash
# Default: 100 samples
cargo bench --workspace

# Fewer samples (faster, less accurate)
cargo bench --workspace -- --sample-size 20

# More samples (slower, more accurate)
cargo bench --workspace -- --sample-size 500
```

### Measurement Time

```bash
# Default: 5 seconds per benchmark
cargo bench --workspace

# Longer measurement (more accurate)
cargo bench --workspace -- --measurement-time 10

# Shorter measurement (faster)
cargo bench --workspace -- --measurement-time 2
```

### Warm-Up Time

```bash
# Default: 3 seconds warm-up
cargo bench --workspace

# Longer warm-up (for JIT, caches)
cargo bench --workspace -- --warm-up-time 5

# No warm-up (not recommended)
cargo bench --workspace -- --warm-up-time 0
```

## Custom Benchmarks

### Writing Your Own

Create `benches/my_bench.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_crate::{baseline_function, aspected_function};

fn benchmark_comparison(c: &mut Criterion) {
    c.bench_function("baseline", |b| {
        b.iter(|| baseline_function(black_box(42)))
    });
    
    c.bench_function("with_aspect", |b| {
        b.iter(|| aspected_function(black_box(42)))
    });
}

criterion_group!(benches, benchmark_comparison);
criterion_main!(benches);
```

Add to `Cargo.toml`:

```toml
[[bench]]
name = "my_bench"
harness = false
```

Run:

```bash
cargo bench --bench my_bench
```

### Parameterized Benchmarks

```rust
fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("aspect_count");
    
    for count in [1, 2, 5, 10] {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, &count| {
                b.iter(|| function_with_n_aspects(black_box(count)))
            }
        );
    }
    
    group.finish();
}
```

## Common Issues

### Issue: Inconsistent Results

**Symptoms:** Large variance between runs

**Causes:**
- Background processes consuming CPU
- Thermal throttling
- CPU frequency scaling
- Insufficient warm-up

**Solutions:**
```bash
# Increase sample size
cargo bench -- --sample-size 200

# Increase warm-up
cargo bench -- --warm-up-time 10

# Check for background processes
top  # or htop

# Verify CPU governor
cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### Issue: "Optimization disabled" Warning

**Message:**
```
warning: the benchmark has been compiled without optimizations
```

**Solution:**
```bash
# Always run benchmarks in release mode
cargo bench  # Uses release profile automatically

# Don't run:
cargo test --bench my_bench  # This uses dev profile!
```

### Issue: Out of Memory

**Symptoms:** Benchmarks crash or system freezes

**Causes:**
- Large data structures
- Memory leaks
- Insufficient RAM

**Solutions:**
```bash
# Reduce sample size
cargo bench -- --sample-size 20

# Run benches one at a time
cargo bench -p aspect-core --bench aspect_overhead
cargo bench -p aspect-core --bench joinpoint_creation
# etc.

# Monitor memory usage
watch -n 1 free -h  # Linux
```

### Issue: Benchmarks Take Too Long

**Solutions:**
```bash
# Reduce measurement time
cargo bench -- --measurement-time 2

# Reduce sample size
cargo bench -- --sample-size 50

# Run specific benchmarks
cargo bench -- aspect_overhead
```

## Continuous Integration

### GitHub Actions

`.github/workflows/benchmark.yml`:

```yaml
name: Benchmarks

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run benchmarks
        run: cargo bench --workspace -- --save-baseline pr
        
      - name: Compare to main
        run: |
          git checkout main
          cargo bench --workspace -- --save-baseline main
          git checkout -
          cargo bench --workspace -- --baseline main
```

## Best Practices

### DO:

1. ✅ Close unnecessary applications
2. ✅ Use performance CPU governor
3. ✅ Run multiple times to verify stability
4. ✅ Save baselines for comparison
5. ✅ Use release profile (cargo bench does this)
6. ✅ Let warm-up complete
7. ✅ Check system temperature
8. ✅ Document system configuration

### DON'T:

1. ❌ Run on laptop battery power
2. ❌ Run with heavy background processes
3. ❌ Compare results from different machines
4. ❌ Trust single-run results
5. ❌ Skip warm-up period
6. ❌ Run while system is under load
7. ❌ Ignore warning messages

## Analyzing Results

### Exporting Data

```bash
# Criterion saves JSON data automatically
# Location: target/criterion/<bench>/base/estimates.json

# View raw data
cat target/criterion/aspect_overhead/base/estimates.json | jq

# Export to CSV
cargo install criterion-table
criterion-table --output results.csv
```

### Graphing Results

```python
# Python script to graph results
import json
import matplotlib.pyplot as plt

with open('target/criterion/aspect_overhead/base/estimates.json') as f:
    data = json.load(f)

times = [data['mean']['point_estimate']]
errors = [data['mean']['standard_error']]

plt.bar(['No Aspect', 'With Aspect'], times, yerr=errors)
plt.ylabel('Time (ns)')
plt.title('Aspect Overhead')
plt.savefig('overhead.png')
```

### Statistical Analysis

```r
# R script for analysis
library(jsonlite)

data <- fromJSON('target/criterion/aspect_overhead/base/estimates.json')

cat(sprintf("Mean: %.2f ns\n", data$mean$point_estimate))
cat(sprintf("Std Dev: %.2f ns\n", data$std_dev$point_estimate))
cat(sprintf("95%% CI: [%.2f, %.2f] ns\n", 
    data$mean$confidence_interval$lower_bound,
    data$mean$confidence_interval$upper_bound))
```

## Troubleshooting

### Getting Help

```bash
# Criterion help
cargo bench --workspace -- --help

# Verbose output for debugging
cargo bench --workspace -- --verbose

# List all benchmarks without running
cargo bench --workspace -- --list
```

### Checking Configuration

```bash
# View Criterion configuration
cat Cargo.toml | grep criterion -A 5

# Check benchmark files
ls -la benches/

# Verify release profile
cat Cargo.toml | grep -A 10 "\[profile.release\]"
```

## Key Takeaways

1. **Use cargo bench** - Automatically uses release profile
2. **Save baselines** - Enable regression detection
3. **Prepare system** - Minimize background noise
4. **Run multiple times** - Verify stability
5. **Interpret statistically** - p-value < 0.05 for significance
6. **View HTML reports** - Detailed visualizations
7. **Document config** - Record system setup

## Next Steps

- See [Methodology](./methodology.md) for measurement principles
- See [Results](./results.md) for expected performance
- See [Techniques](./techniques.md) for optimization strategies

---

**Related Chapters:**
- [Chapter 9.1: Methodology](./methodology.md) - Benchmarking approach
- [Chapter 9.2: Results](./results.md) - Performance data
- [Chapter 9.4: Techniques](./techniques.md) - Optimization
