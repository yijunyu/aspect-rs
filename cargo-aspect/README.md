# cargo-aspect

Cargo subcommand for aspect-oriented programming with Rust.

## Installation

```bash
cargo install --path cargo-aspect
```

Or from the workspace root:

```bash
cargo install --path .
```

## Usage

### Basic Commands

```bash
# Show version
cargo aspect --version

# Show aspect information
cargo aspect info

# List available aspects
cargo aspect list

# List only aspects
cargo aspect list --aspects

# List only pointcut syntax
cargo aspect list --pointcuts
```

### Build Commands

```bash
# Build with aspect weaving (currently passes through to cargo)
cargo aspect build

# Check with aspect analysis
cargo aspect check

# Run tests
cargo aspect test

# Run benchmarks
cargo aspect bench

# Clean build artifacts
cargo aspect clean
```

### Advanced Usage

```bash
# Verbose output
cargo aspect -v build

# Detailed aspect information
cargo aspect info --detailed

# Filter by module (Automated Weaving feature)
cargo aspect info --module crate::api

# Pass additional arguments to cargo
cargo aspect build --release
cargo aspect test -- --nocapture
cargo aspect bench -- --save-baseline main
```

### Available Now
- ✅ Command-line interface
- ✅ Cargo command pass-through
- ✅ Aspect information display
- ✅ Available aspects listing
- ✅ Pointcut syntax reference
- ✅ Automatic aspect weaving
- ✅ MIR-based pointcut matching
- ✅ Advanced code analysis
- ✅ IDE integration support

## Examples

### Show Framework Information

```bash
$ cargo aspect info
=== Aspect Information ===

Framework: aspect-rs v0.1.0
Status: Proc Macros + Pointcuts
```

### List Available Aspects

```bash
$ cargo aspect list
=== Registered Aspects ===

Available aspects (from aspect-std):
  • LoggingAspect      - Structured logging
  • TimingAspect       - Performance monitoring
  • CachingAspect      - Memoization
  • MetricsAspect      - Call statistics
  • RateLimitAspect    - Rate limiting
  • CircuitBreaker     - Fault tolerance
  • Authorization      - Access control
  • Validation         - Pre-conditions

Pointcut syntax:
  execution(pub fn *(..))     - All public functions
  within(crate::api)          - Functions in module
  name("fetch_*")              - Name patterns
  execution(..) && within(..) - Combinations
```

### Build with Aspects

```bash
# Standard build (currently equivalent to cargo build)
$ cargo aspect build

# Release build
$ cargo aspect build --release

# Verbose output
$ cargo aspect -v build
cargo-aspect v0.1.0
Running: cargo build
    Finished release [optimized] target(s) in 0.05s
```

## Development

### Testing

```bash
# Run tests
cargo test -p cargo-aspect

# Integration test
cargo run -p cargo-aspect -- aspect --version
cargo run -p cargo-aspect -- aspect info
cargo run -p cargo-aspect -- aspect list
```

### Adding New Commands

Edit `src/main.rs` and add to the `AspectCommand` enum:

```rust
#[derive(Subcommand, Debug)]
enum AspectCommand {
    // ... existing commands

    /// Your new command
    NewCommand {
        #[arg(short, long)]
        option: bool,
    },
}
```

Then implement in `run_aspect_command()`.

## Architecture

```
cargo-aspect/
├── Cargo.toml          # Dependencies (clap, anyhow)
├── src/
│   └── main.rs         # CLI implementation (~270 lines)
└── README.md           # This file
```

### Design Decisions

- **clap** for argument parsing (derive API for simplicity)
- **anyhow** for error handling (nice error messages)
- **Pass-through design** for compatibility
- **Extensible commands** for integration
- Access MIR (Mid-level IR)
- Extract function metadata
- Register compiler callbacks
- Match pointcuts against MIR
- Inject aspect code
- Optimize generated code
- Field access interception
- Call-site matching
- Whole-program analysis
- IDE integration
- Language server support
- Documentation and examples

## Contributing

Contributions welcome! See the main [aspect-rs README](../README.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
