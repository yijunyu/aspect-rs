# Contributing to aspect-rs

Welcome! This chapter explains how to contribute to aspect-rs, from reporting bugs to submitting code.

## Quick Start

**New to the project?**

1. Read the [documentation](https://aspect-rs.dev)
2. Try the [examples](../ch08-case-studies/README.md)
3. Check [open issues](https://github.com/aspect-rs/issues)
4. Join our community (Discord, GitHub Discussions)

**Ready to contribute?**

1. Fork the repository
2. Clone your fork
3. Create a branch
4. Make changes
5. Submit a pull request

## Ways to Contribute

### 1. Report Bugs

**Before reporting:**
- Search existing issues
- Verify it's actually a bug
- Test on latest version

**Good bug report includes:**

```markdown
### Description
Clear explanation of the problem.

### Steps to Reproduce
1. Create a file with...
2. Run command...
3. Observe error...

### Expected Behavior
Function should return X

### Actual Behavior
Function returns Y instead

### Environment
- Rust version: 1.70.0
- aspect-rs version: 0.3.0
- OS: Ubuntu 22.04
- Cargo version: 1.70.0

### Code Sample
\`\`\`rust
#[aspect(LoggingAspect::new())]
fn buggy_function() {
    // Minimal reproducible example
}
\`\`\`

### Error Output
\`\`\`
error[E0XXX]: ...
\`\`\`
```

**Use GitHub issues:** https://github.com/aspect-rs/issues/new

### 2. Suggest Features

**Good feature requests include:**

1. **Problem statement** - What problem does this solve?
2. **Use case** - How would you use it?
3. **Examples** - Code showing desired API
4. **Alternatives** - What options did you consider?

**Example:**

```markdown
### Feature Request: Parameter Matching in Pointcuts

**Problem:**
Currently can't match functions by parameter types.

**Use Case:**
Want to apply TransactionalAspect only to functions
taking Database parameter.

**Example:**
\`\`\`rust
// Proposed syntax
--aspect-pointcut "execution(fn *(db: &Database, ..))"
\`\`\`

**Alternatives Considered:**
- Module-based matching (too broad)
- Name-based matching (too fragile)
```

### 3. Improve Documentation

**Documentation help needed:**

- API documentation (doc comments)
- Book chapters (mdBook)
- Examples (working code)
- Tutorials (step-by-step guides)
- Blog posts (use cases)

**How to help:**

```bash
# Edit documentation
cd docs/book/src
# Edit .md files
git commit -m "docs: Improve XYZ explanation"
```

**Guidelines:**

- Clear, concise writing
- Code examples that compile
- Cross-references to related sections
- Proper markdown formatting

### 4. Submit Code

**Process:**

1. **Find an issue** or create one
2. **Comment** that you'll work on it
3. **Fork** the repository
4. **Create branch**: `git checkout -b feature/my-feature`
5. **Make changes** following coding standards
6. **Add tests** for new functionality
7. **Run tests**: `cargo test --workspace`
8. **Run checks**: `cargo fmt` and `cargo clippy`
9. **Commit** with clear message
10. **Push** to your fork
11. **Create PR** with description

## Development Setup

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should be 1.70+
cargo --version
```

### Clone and Build

```bash
# Fork on GitHub first, then:
git clone https://github.com/YOUR_USERNAME/aspect-rs.git
cd aspect-rs

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run specific example
cargo run --example logging
cargo run --example api_server
```

### Useful Commands

```bash
# Format code
cargo fmt --all

# Check for issues
cargo clippy --workspace -- -D warnings

# Run benchmarks
cargo bench --workspace

# Generate documentation
cargo doc --workspace --no-deps --open

# Expand macros (debugging)
cargo install cargo-expand
cargo expand --example logging

# Check compile times
cargo build --workspace --timings

# View dependencies
cargo tree
```

## Project Structure

```
aspect-rs/
├── aspect-core/           # Core traits and types
│   ├── src/
│   │   ├── aspect.rs     # Aspect trait
│   │   ├── joinpoint.rs  # JoinPoint types
│   │   ├── error.rs      # Error types
│   │   └── lib.rs        # Public API
│   └── tests/            # Integration tests
│
├── aspect-macros/         # Procedural macros
│   ├── src/
│   │   ├── aspect_attr.rs # #[aspect] macro
│   │   ├── codegen.rs    # Code generation
│   │   └── lib.rs        # Macro entry points
│   └── tests/            # Macro expansion tests
│
├── aspect-std/            # Standard aspect library
│   ├── src/
│   │   ├── logging.rs    # LoggingAspect
│   │   ├── timing.rs     # TimingAspect
│   │   ├── caching.rs    # CachingAspect
│   │   ├── retry.rs      # RetryAspect
│   │   └── ...           # More aspects
│   └── tests/            # Aspect tests
│
├── aspect-rustc-driver/   # Phase 3: Compiler integration
│   ├── src/
│   │   ├── main.rs       # Driver entry point
│   │   ├── callbacks.rs  # Compiler callbacks
│   │   └── analysis.rs   # MIR analysis
│   └── tests/
│
├── aspect-driver/         # MIR analyzer library
│   ├── src/
│   │   ├── mir_analyzer.rs    # MIR extraction
│   │   ├── pointcut_matcher.rs # Pointcut matching
│   │   └── types.rs      # Shared types
│   └── tests/
│
├── aspect-examples/       # Example applications
│   ├── src/
│   │   ├── logging.rs    # Basic logging example
│   │   ├── api_server.rs # RESTful API
│   │   └── ...           # More examples
│   └── benches/          # Benchmarks
│
├── docs/                  # Documentation
│   └── book/             # mdBook source
│
└── Cargo.toml            # Workspace config
```

## Coding Standards

### Style

**Follow Rust conventions:**

```rust
// Good: Clear names, proper formatting
pub fn extract_function_metadata(item: &Item) -> FunctionMetadata {
    FunctionMetadata {
        name: item.ident.to_string(),
        visibility: determine_visibility(item),
    }
}

// Bad: Unclear names, poor formatting
pub fn ext(i:&Item)->FM{FM{n:i.ident.to_string(),v:det_vis(i)}}
```

**Use rustfmt:**

```bash
cargo fmt --all
```

**Use clippy:**

```bash
cargo clippy --workspace -- -D warnings
```

### Documentation

**All public items must have docs:**

```rust
/// Extracts metadata from a function item.
///
/// This function analyzes an HIR item and extracts relevant
/// information for aspect matching.
///
/// # Arguments
///
/// * `item` - The HIR item to analyze
///
/// # Returns
///
/// Function metadata including name, visibility, and location
///
/// # Examples
///
/// ```
/// use aspect_driver::extract_function_metadata;
///
/// let metadata = extract_function_metadata(&item);
/// assert_eq!(metadata.name, "my_function");
/// ```
pub fn extract_function_metadata(item: &Item) -> FunctionMetadata {
    // Implementation...
}
```

### Error Handling

**Use Result for recoverable errors:**

```rust
// Good
pub fn parse_pointcut(expr: &str) -> Result<Pointcut, ParseError> {
    if expr.is_empty() {
        return Err(ParseError::EmptyExpression);
    }
    // Parse...
}

// Bad
pub fn parse_pointcut(expr: &str) -> Pointcut {
    if expr.is_empty() {
        panic!("Empty expression!");  // Don't panic on user input!
    }
    // Parse...
}
```

**Provide helpful error messages:**

```rust
// Good
if !expr.contains("execution") && !expr.contains("within") {
    return Err(ParseError::InvalidPointcutType {
        expr: expr.to_string(),
        expected: "execution(...) or within(...)",
    });
}

// Bad
return Err(ParseError::Invalid);
```

### Testing

**Every feature needs tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_execution_pointcut() {
        let expr = "execution(pub fn *(..))";
        let pointcut = parse_pointcut(expr).unwrap();
        
        assert!(matches!(pointcut, Pointcut::Execution(_)));
    }
    
    #[test]
    fn test_parse_invalid_pointcut() {
        let expr = "invalid syntax";
        let result = parse_pointcut(expr);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_match_public_function() {
        let pointcut = Pointcut::Execution(ExecutionPattern {
            visibility: Some(VisibilityKind::Public),
            name: "*".to_string(),
        });
        
        let func = FunctionMetadata {
            name: "test_func".to_string(),
            visibility: VisibilityKind::Public,
        };
        
        let matcher = PointcutMatcher::new(vec![pointcut]);
        assert!(matcher.matches(&pointcut, &func));
    }
}
```

**Run tests before submitting:**

```bash
cargo test --workspace
```

### Commits

**Good commit messages:**

```
feat: Add parameter matching to pointcuts

Implements support for matching functions based on
parameter types in execution pointcuts.

Syntax: execution(fn *(id: u64, ..))

Closes #123
```

**Commit message format:**

```
<type>: <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

## Areas for Contribution

### High Priority

**1. Code Generation (v0.4)**
- Implement wrapper function generation
- Handle aspect instantiation
- Source code modification
- Integration tests

**2. Configuration System**
- TOML config file parsing
- Configuration validation
- Default config discovery
- Documentation

**3. Error Messages**
- Better parse errors
- Helpful suggestions
- Source location tracking
- rustc-style diagnostics

### Medium Priority

**4. Standard Aspects**
- Distributed tracing aspect
- Async retry aspect
- Connection pooling aspect
- Custom authentication aspect

**5. Performance**
- Reduce allocation overhead
- Optimize macro expansion
- Benchmark improvements
- Profile-guided optimization

**6. Documentation**
- More examples
- Tutorial blog posts
- Video walkthroughs
- API improvements

### Future Work

**7. IDE Integration**
- rust-analyzer plugin
- Aspect visualization
- Debugging support
- Code navigation

**8. Advanced Features**
- Around advice
- Parameter matching
- Return type matching
- Field access interception

## Pull Request Process

### Before Submitting

**Checklist:**

- [ ] Tests pass: `cargo test --workspace`
- [ ] Formatted: `cargo fmt --all`
- [ ] Linted: `cargo clippy --workspace -- -D warnings`
- [ ] Documentation updated
- [ ] Example added (if applicable)
- [ ] Commit messages clear

### PR Description

**Template:**

```markdown
### What does this PR do?
Brief description of changes.

### Why?
Explanation of motivation.

### How?
Technical approach taken.

### Related Issues
Closes #123

### Testing
- Added tests for X
- Verified Y manually

### Screenshots (if UI changes)
[If applicable]
```

### Review Process

1. **Automated checks** run (CI/CD)
2. **Maintainer review** (usually within 2-3 days)
3. **Feedback** addressed
4. **Approval** and merge

**Be patient and responsive:**
- Reviews may take time
- Address feedback constructively
- Ask questions if unclear

## Code of Conduct

**Be respectful and constructive:**

✅ Good:
- "I think there might be an issue with..."
- "Have you considered...?"
- "This could be improved by..."

❌ Bad:
- "This code is terrible"
- "Why did you do it this way?"
- "Obviously wrong"

**Follow [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)**

## Getting Help

**Stuck? Ask for help:**

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Discord**: Real-time chat (link in README)
- **Stack Overflow**: Tag with `aspect-rs`

**Asking good questions:**

1. What are you trying to do?
2. What did you try?
3. What happened instead?
4. Code sample reproducing the issue
5. Error messages (full output)

## Recognition

**Contributors are valued:**

- Listed in AUTHORS file
- Mentioned in release notes
- Badge on GitHub profile
- Potential core team invitation

**Contribution levels:**

1. **Contributor**: First PR merged
2. **Frequent Contributor**: 5+ PRs merged
3. **Reviewer**: Can review PRs
4. **Maintainer**: Merge rights
5. **Core Team**: Direction and decisions

## License

**By contributing, you agree:**

Your contributions will be licensed under MIT/Apache-2.0
dual license, same as the project.

## Resources

**Learn more:**

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [rustc Dev Guide](https://rustc-dev-guide.rust-lang.org/)
- [Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html)

## Key Takeaways

1. **Many ways to contribute** - Code, docs, bugs, features
2. **Clear process** - Fork, branch, code, test, PR
3. **High standards** - Tests, docs, formatting required
4. **Welcoming community** - Help available, questions encouraged
5. **Recognition** - Contributors valued and recognized

---

**Thank you for contributing to aspect-rs!**

**Related Chapters:**
- [Chapter 11.1: Achievements](./achievements.md) - What we've built
- [Chapter 11.2: Roadmap](./roadmap.md) - Where we're going
- [Chapter 11.5: Acknowledgements](./acknowledgements.md) - Thank you
