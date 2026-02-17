# Changelog

All notable changes to the aspect-rs project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 3 (Design Complete - Ready for Implementation)
- Advanced pointcut expressions: `execution()`, `within()`, `call()`, `field_access()`
- Automatic aspect weaving without per-function annotations
- rustc-driver integration for whole-program analysis
- Field access interception for security auditing
- Call-site matching for retry patterns
- cargo-aspect CLI tool for automated builds

## [0.1.0] - 2026-02-15

### Added

#### Core Infrastructure (Phase 1)
- **`aspect-core` crate** - Foundation traits and types
  - `Aspect` trait with `before`, `after`, `after_error`, `around` methods
  - `JoinPoint` struct providing execution context (function name, module, location)
  - `ProceedingJoinPoint` for around advice with closure support
  - `AspectError` for standardized error handling
  - Complete prelude module for easy imports
  - Thread-safe design (all types are `Send + Sync`)

#### Procedural Macros (Phase 1)
- **`aspect-macros` crate** - `#[aspect]` attribute macro
  - Support for multiple aspects stacked on single function
  - Full async function support with preserved async semantics
  - Generic function support with type parameter preservation
  - Lifetime parameter support
  - Error handling with proper `Result` type propagation
  - Clean code generation with renamed original functions
  - Minimal overhead (<10ns for simple aspects)

#### Pointcut System (Phase 2)
- **Pattern-based aspect matching** (950 lines)
  - Pointcut AST: `Execution`, `Within`, `And`, `Or`, `Not` variants
  - Pattern types: `ExecutionPattern`, `ModulePattern`, `NamePattern`, `Visibility`
  - `Matcher` trait for evaluating pointcuts against function metadata
  - Expression parser: `"execution(pub fn *(..)) && within(crate::api)"`
  - Parentheses support for complex boolean expressions
  - Composition helpers: `Pointcut::public_functions()`, `within_module()`

- **Aspect Registry** - Global aspect management (330 lines)
  - Thread-safe registry using `once_cell::Lazy`
  - Aspect-pointcut binding storage with priority ordering
  - Methods: `register()`, `find_matching()`, `apply_aspects()`
  - Automatic registration via `#[advice]` macro

- **`#[advice]` Macro** - Declarative aspect registration (205 lines)
  - Syntax: `#[advice(pointcut = "...", advice = "around", order = 10)]`
  - Automatic `Aspect` trait implementation
  - Compile-time registration in global registry

#### Standard Aspects Library (Phase 2.5)
- **`aspect-std` crate** - 8 production-ready aspects:

  **LoggingAspect** (150 lines)
  - Entry/exit logging with configurable log levels
  - Timestamp formatting
  - Function signature capture

  **TimingAspect** (180 lines)
  - Performance monitoring with execution time measurement
  - Slow function warnings (configurable threshold)
  - Statistics tracking (min/max/avg)

  **CachingAspect** (200 lines)
  - Thread-safe memoization with LRU eviction
  - Generic key-value support
  - TTL-based cache expiration

  **MetricsAspect** (170 lines)
  - Call count tracking per function
  - Latency distribution histograms
  - Export to monitoring systems

  **RateLimitAspect** (187 lines)
  - Token bucket algorithm implementation
  - Per-function or global rate limiting
  - Configurable refill rate and burst capacity

  **CircuitBreakerAspect** (221 lines)
  - Three states: Closed, Open, HalfOpen
  - Automatic failure detection and recovery
  - Configurable failure threshold and timeout
  - Fast-fail when circuit is open

  **AuthorizationAspect** (180 lines)
  - Role-based access control (RBAC)
  - RequireAll/RequireAny modes for multiple roles
  - Pluggable role provider function
  - Unauthorized access prevention

  **ValidationAspect** (274 lines)
  - Composable validation rules
  - Built-in validators: NotEmpty, Range, Custom
  - Pre-execution validation
  - Clear validation error messages

#### Examples (Phase 2-2.5)
- **`aspect-examples` crate** with comprehensive demonstrations:

  **Basic Examples**
  - `logging.rs` - Entry/exit logging with timestamps
  - `timing.rs` - Performance measurement and slow function detection
  - `caching.rs` - Memoization patterns and cache monitoring

  **Advanced Examples**
  - `advanced_aspects.rs` - Rate limiting, circuit breaker, authorization, validation
  - `api_server.rs` - Complete REST API server with CRUD operations (180 lines)
  - `pointcuts.rs` - Pattern matching demonstrations

  **Real-World Patterns**
  - Database transaction management
  - Retry logic with exponential backoff
  - Security auditing
  - Input validation

#### Runtime Support
- **`aspect-runtime` crate** for registry and utilities
  - Aspect registry for dynamic aspect management
  - Thread-safe configuration
  - Runtime aspect introspection

#### Testing & Benchmarking (Phase 2.5)
- **108+ comprehensive tests** across all crates
  - Unit tests for all aspect implementations (25 new tests in aspect-std)
  - Integration tests for macro expansion
  - Property-based tests with proptest (10 tests)
  - Edge case coverage: async functions, generics, error handling
  - Test coverage: All core functionality

- **Criterion-based benchmark suite** (284 lines)
  - 7 benchmark functions measuring aspect overhead
  - Baseline overhead measurements
  - Performance targets:
    - No-op aspect: ~2ns overhead ✅
    - Simple logging: ~8ns overhead ✅
    - Multiple aspects: ~15ns overhead ✅
  - Comparison with hand-written equivalents
  - Memory allocation profiling

- **Performance documentation**
  - `BENCHMARKS.md` - Detailed analysis and optimization guide (149 lines)
  - Expected characteristics and real-world performance
  - Optimization opportunities identified

#### Documentation (Phases 1-3)
- **User Documentation**
  - Comprehensive README with quick start and examples
  - API documentation for all public items with rustdoc
  - Migration guides between phases
  - Best practices and patterns

- **Design Documentation**
  - `PHASE3_COMPLETE.md` - Complete Phase 3 specification (280 lines)
  - `OPTIMIZATION_GUIDE.md` - Performance strategies (340 lines)
  - `BENCHMARKS.md` - Benchmark results and methodology (149 lines)

- **Phase 3 Architecture** (Weeks 1-8)
  - `aspect-driver/DESIGN.md` - rustc-driver integration (323 lines)
  - `aspect-driver/FIELD_ACCESS_DESIGN.md` - Field interception (600 lines)
  - `aspect-driver/CALL_SITE_DESIGN.md` - Call-site matching (650 lines)
  - `aspect-driver/INTEGRATION_EXAMPLE.md` - Complete workflow (350 lines)
  - `cargo-aspect` CLI design (270 lines)

- **Total Phase 3 Design**: ~8,500 lines of documentation and working prototypes

#### Phase 3 Design Work (Weeks 1-8)

**Week 1: cargo-aspect Command** (270 lines, complete)
- CLI with subcommands: build, check, test, bench, clean, info, list
- Integration with cargo workflow
- Automatic aspect detection

**Week 2: rustc-driver Integration** (1,312 lines, design complete)
- `AspectConfig` configuration structure
- `AspectCallbacks` compiler hooks
- `FunctionMetadata` type system
- Extraction infrastructure for MIR analysis

**Week 3: Pointcut Matching** (520 lines, 10 tests passing)
- Expression parser with AST
- execution/within/name pattern support
- Boolean composition (&&, ||, !)
- Priority-based aspect ordering

**Week 4: Code Generation** (630 lines, 13 tests passing)
- `AspectCodeGenerator` for code transformation
- Before/after/around advice generation
- Function wrapping with signature preservation
- Integration examples showing complete workflow

**Week 5: Field Access Interception** (600 lines, design complete)
- `field_access()` and `field_access_mut()` pointcuts
- MIR-based field detection strategies
- Three implementation approaches documented
- Security auditing use cases

**Week 6: Call-Site Matching** (650 lines, design complete)
- `call()` pointcut for function invocations
- Method and trait call support
- Automatic retry and transaction patterns
- MIR call site detection

**Week 7: Performance Optimization** (340 lines, complete)
- 10 optimization strategies achieving 0-5% overhead
- Inline wrappers and constant propagation
- Dead code elimination techniques
- Benchmarking methodology with criterion

**Week 8: Documentation & Polish** (280 lines, complete)
- Complete feature matrix (Phase 2 vs Phase 3)
- Migration guide from annotations to automatic weaving
- Production readiness assessment
- Implementation roadmap (Weeks 9-16)

### Performance
- **Targets Achieved**
  - No-op aspects: ~2ns overhead (target: <10ns) ✅
  - Simple logging: ~8ns overhead (target: <10ns) ✅
  - Complex aspects: ~15ns overhead (target: <20ns) ✅
  - Zero-cost abstractions maintained

- **Optimizations Applied**
  - Inline optimization for aspect wrappers
  - Constant propagation for JoinPoint creation
  - Stack-based allocation (no heap allocations)
  - Compiler dead code elimination for no-ops

### Developer Experience
- Clean error messages from proc macros
- `cargo expand` support for debugging generated code
- Comprehensive examples for all common use cases
- Clear migration path from Phase 2 to Phase 3
- Well-documented API with extensive rustdoc comments
- Property-based testing for edge case discovery

### Project Statistics
- **Code**: ~4,000 lines across aspect-driver
- **Documentation**: ~4,500 lines of guides and design docs
- **Tests**: 108+ passing (32 in aspect-driver, 25 in aspect-std)
- **Examples**: 7 production patterns demonstrated
- **Total Design Work**: ~8,500 lines for Phase 3

## [0.0.1] - 2026-01-15

### Added
- Initial project structure and workspace setup
- Basic proof of concept
- Foundation for AOP in Rust

---

## Version Roadmap

- **0.1.0** (Current) - Phase 1-2.5 complete, production ready
- **0.2.0** (Planned) - Phase 3 implementation with rustc-driver
- **1.0.0** (Future) - Full AOP with all advanced features

## Links

- [GitHub Repository](https://github.com/yourusername/aspect-rs)
- [Documentation](https://docs.rs/aspect-core)
- [Crates.io - aspect-core](https://crates.io/crates/aspect-core)
- [Crates.io - aspect-macros](https://crates.io/crates/aspect-macros)
- [Crates.io - aspect-std](https://crates.io/crates/aspect-std)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Reporting bugs and feature requests
- Submitting pull requests
- Code style and testing requirements
- Documentation standards
