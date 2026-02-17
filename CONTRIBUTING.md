# Contributing to aspect-rs

Thank you for your interest in contributing to aspect-rs! We welcome contributions from everyone.

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Clear title** describing the issue
- **Detailed description** of the problem
- **Steps to reproduce** the behavior
- **Expected vs actual behavior**
- **Environment details** (Rust version, OS, aspect-rs version)
- **Code sample** demonstrating the issue
- **Error messages** if applicable

### Suggesting Features

Feature requests are welcome! Please:

- **Check existing feature requests** first
- **Describe the use case** - why is this needed?
- **Provide examples** of how it would be used
- **Consider the scope** - does it fit the project goals?

### Pull Requests

1. **Fork the repository** and create a branch from `main`
2. **Make your changes** following our coding standards
3. **Add tests** for any new functionality
4. **Update documentation** including doc comments and README if needed
5. **Ensure all tests pass**: `cargo test --workspace`
6. **Run formatting**: `cargo fmt --all`
7. **Run clippy**: `cargo clippy --workspace -- -D warnings`
8. **Commit your changes** with clear, descriptive commit messages
9. **Push to your fork** and submit a pull request

## Development Setup

```bash
# Clone the repository
git clone https://github.com/yijunyu/aspect-rs.git
cd aspect-rs

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run examples
cargo run --example logging
cargo run --example api_server
cargo run --example advanced_aspects

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --workspace -- -D warnings

# Run benchmarks
cargo bench

# Expand macros (useful for debugging)
cargo install cargo-expand
cargo expand --example logging

# Generate documentation
cargo doc --workspace --no-deps --open
```

## Project Structure

```
aspect-rs/
├── aspect-core/           # Core traits (Aspect, JoinPoint)
│   ├── src/
│   │   ├── aspect.rs     # Aspect trait definition
│   │   ├── joinpoint.rs  # JoinPoint structs
│   │   ├── error.rs      # Error types
│   │   └── lib.rs        # Public API
│   └── tests/            # Integration tests
├── aspect-macros/         # Procedural macros
│   ├── src/
│   │   ├── aspect_attr.rs # #[aspect] macro
│   │   ├── advice_attr.rs # #[advice] macro
│   │   └── lib.rs        # Macro entry points
│   └── tests/            # Macro tests
├── aspect-std/            # Standard aspects (8 production aspects)
│   ├── src/
│   │   ├── logging.rs    # LoggingAspect
│   │   ├── timing.rs     # TimingAspect
│   │   ├── caching.rs    # CachingAspect
│   │   ├── ratelimit.rs  # RateLimitAspect
│   │   └── ...           # More aspects
│   └── tests/            # Aspect tests
├── aspect-runtime/        # Runtime support
├── aspect-examples/       # Comprehensive examples
├── aspect-driver/         # rustc integration (design)
└── cargo-aspect/          # cargo plugin (design)
```

## Coding Standards

### Style

- Follow standard Rust formatting (`cargo fmt`)
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use meaningful variable and function names
- Keep functions focused and reasonably sized
- Prefer explicit types in public APIs

### Documentation

All public items must have documentation:

```rust
/// Brief one-line summary.
///
/// More detailed explanation if needed.
///
/// # Examples
///
/// ```
/// use aspect_core::Aspect;
///
/// // Example usage
/// ```
///
/// # Errors
///
/// Returns an error if...
pub fn my_function() { }
```

### Error Handling

- Use `Result<T, E>` for recoverable errors
- Use `panic!` only for bugs or unrecoverable errors
- Provide helpful error messages
- Use custom error types from `aspect-core::error`

### Testing

All code changes should include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange, Act, Assert
        let aspect = MyAspect::new();
        let ctx = create_test_joinpoint();

        aspect.before(&ctx);

        assert!(condition);
    }
}
```

### Commits

- Write clear, concise commit messages
- Use present tense ("Add feature" not "Added feature")
- Reference issues and PRs where relevant (#123)
- Keep commits focused and atomic

## Questions?

- Open a [GitHub Issue](https://github.com/yijunyu/aspect-rs/issues)
- Start a [Discussion](https://github.com/yijunyu/aspect-rs/discussions)
- Read the [documentation](https://docs.rs/aspect-core)

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT/Apache-2.0 dual license).

---

**Thank you for contributing to aspect-rs!** 🎉
