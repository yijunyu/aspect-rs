# Performance Benchmarks

Detailed performance analysis demonstrating aspect-rs overhead.

## Key Findings

- **Empty function**: +2ns overhead (20%)
- **Logging aspect**: +2ns overhead (13%)
- **Timing aspect**: +2ns overhead (10%)
- **Caching aspect (hit)**: +2ns overhead (40%)
- **Caching aspect (miss)**: +2ns overhead (2%)

**Conclusion**: aspect-rs has consistent ~2ns overhead regardless of function complexity.

## Benchmark Suite

All benchmarks use criterion with statistical analysis:

```bash
cargo bench --package aspect-benches
```

See [Methodology](methodology.md) for details.
