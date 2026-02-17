# Development Roadmap

This chapter outlines the future development plans for aspect-rs, organized by timeline and priority.

## Current Status

**Version:** 0.3.0 (Phase 3 Analysis Complete)

**What Works:**
- ✅ Core AOP framework (Phase 1)
- ✅ Production features (Phase 2)
- ✅ Standard aspect library
- ✅ MIR extraction and analysis
- ✅ Pointcut expression matching
- ✅ Automatic function detection
- ✅ Comprehensive documentation

**What's Next:**
- Code generation for automatic weaving
- IDE integration
- Community ecosystem
- Advanced features

## Short Term (v0.4 - Next 3 Months)

### Priority 1: Code Generation

**Goal:** Complete automatic aspect weaving with code generation.

**Features:**

1. **Wrapper Function Generation**
   ```rust
   // Input (clean code)
   pub fn fetch_user(id: u64) -> User {
       database::get(id)
   }
   
   // Generated (automatic)
   #[inline(never)]
   fn __aspect_original_fetch_user(id: u64) -> User {
       database::get(id)
   }
   
   #[inline(always)]
   pub fn fetch_user(id: u64) -> User {
       let ctx = JoinPoint { /* ... */ };
       LoggingAspect::new().before(&ctx);
       let result = __aspect_original_fetch_user(id);
       LoggingAspect::new().after(&ctx, &result);
       result
   }
   ```

2. **Aspect Application**
   - Parse `--aspect-apply` arguments
   - Instantiate aspects correctly
   - Handle aspect errors gracefully
   - Support multiple aspects per function

3. **Source Code Modification**
   - Generate modified source files
   - Preserve formatting and comments
   - Handle module structure correctly
   - Support incremental compilation

**Deliverables:**
- Working code generation engine
- End-to-end automatic weaving
- Integration tests
- Performance benchmarks

**Timeline:** 6-8 weeks

### Priority 2: Configuration System

**Goal:** Flexible aspect configuration without command-line arguments.

**Configuration File Format:**

```toml
# aspect-config.toml

[[pointcuts]]
pattern = "execution(pub fn *(..))"
aspects = ["LoggingAspect::new()"]
description = "Log all public functions"

[[pointcuts]]
pattern = "within(api::handlers)"
aspects = [
    "TimingAspect::new()",
    "AuthorizationAspect::require_role(\"user\")",
]
description = "Time and authorize API handlers"

[[pointcuts]]
pattern = "within(database::ops)"
aspects = ["TransactionalAspect"]
description = "Wrap database operations in transactions"

[options]
verbose = true
output = "target/aspect-analysis.txt"
verify_only = false
```

**Features:**
- TOML configuration parsing
- Default config file discovery (`aspect-config.toml`)
- Override with command-line args
- Validation and error reporting
- Multiple config file support

**Deliverables:**
- Config parser implementation
- Documentation
- Examples
- Migration guide from CLI args

**Timeline:** 2-3 weeks

### Priority 3: Error Messages

**Goal:** Production-quality error reporting.

**Improvements:**

1. **Pointcut Parse Errors**
   ```
   error: Invalid pointcut expression
     --> aspect-config.toml:5:11
      |
   5  | pattern = "execution(pub fn)"
      |           ^^^^^^^^^^^^^^^^^^
      |
      = note: Missing parameter list '(..)' in execution pointcut
      = help: Expected: execution(pub fn PATTERN(..))
   ```

2. **Match Failures**
   ```
   warning: No functions matched pointcut
     --> aspect-config.toml:10:11
      |
   10 | pattern = "within(nonexistent::module)"
      |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
      |
      = note: 0 functions found in module 'nonexistent'
      = help: Check module path and visibility
   ```

3. **Aspect Instantiation Errors**
   ```
   error: Failed to instantiate aspect
     --> aspect-config.toml:3:13
      |
   3  | aspects = ["InvalidAspect::new()"]
      |           ^^^^^^^^^^^^^^^^^^^^^^^^
      |
      = note: Aspect 'InvalidAspect' not found
      = help: Add dependency: aspect-std = "0.3"
   ```

**Deliverables:**
- Structured error types
- Source location tracking
- Helpful error messages
- Integration with rustc diagnostics

**Timeline:** 2-3 weeks

## Medium Term (v0.5 - Next 6 Months)

### Priority 1: IDE Integration

**Goal:** First-class developer experience in IDEs.

**rust-analyzer Extension:**

1. **Aspect Visualization**
   - Inline hints showing applied aspects
   - Hover tooltips with aspect details
   - Go-to-definition for aspect source

2. **Pointcut Assistance**
   - Auto-completion for pointcut expressions
   - Real-time validation
   - Function count preview

3. **Debugging Support**
   - Step through aspect code
   - Breakpoints in aspects
   - Aspect call stack

**Example IDE Features:**

```rust
pub fn fetch_user(id: u64) -> User {
    // ← Aspects: LoggingAspect, TimingAspect (click to view)
    database::get(id)
}
```

**Deliverables:**
- rust-analyzer plugin
- VS Code extension
- IntelliJ IDEA plugin (community)
- Documentation

**Timeline:** 8-10 weeks

### Priority 2: Advanced Pointcuts

**Goal:** Richer pointcut expression language.

**New Pointcut Types:**

1. **Parameter Matching**
   ```bash
   # Functions with specific parameter types
   execution(fn *(id: u64, ..))
   execution(fn *(user: &User))
   ```

2. **Return Type Matching**
   ```bash
   # Functions returning Result
   execution(fn *(..) -> Result<*, *>)
   
   # Functions returning specific type
   execution(fn *(..) -> User)
   ```

3. **Annotation Matching**
   ```bash
   # Functions with specific attributes
   execution(@deprecated fn *(..))
   execution(@test fn *(..))
   ```

4. **Call-Site Matching**
   ```bash
   # Functions that call specific functions
   call(database::query) && within(api)
   ```

5. **Field Access**
   ```bash
   # Get/set field access
   get(User.email)
   set(User.*)
   ```

**Deliverables:**
- Extended pointcut parser
- Matcher implementations
- Tests and documentation
- Examples

**Timeline:** 6-8 weeks

### Priority 3: Aspect Composition

**Goal:** Control aspect ordering and composition.

**Features:**

1. **Explicit Ordering**
   ```toml
   [[pointcuts]]
   pattern = "execution(pub fn *(..))"
   aspects = [
       { aspect = "AuthorizationAspect", order = 1 },
       { aspect = "LoggingAspect", order = 2 },
       { aspect = "TimingAspect", order = 3 },
   ]
   ```

2. **Dependency Declarations**
   ```rust
   impl Aspect for MyAspect {
       fn dependencies(&self) -> Vec<&str> {
           vec!["LoggingAspect"]
       }
   }
   ```

3. **Conditional Aspects**
   ```toml
   [[pointcuts]]
   pattern = "execution(pub fn *(..))"
   aspects = ["LoggingAspect"]
   condition = "cfg(debug_assertions)"
   ```

**Deliverables:**
- Ordering system
- Dependency resolution
- Conditional compilation
- Documentation

**Timeline:** 4-6 weeks

## Long Term (v1.0 - Next 12 Months)

### Priority 1: Stable Release

**Goal:** Production-ready v1.0 release.

**Requirements:**

1. **API Stability**
   - Finalize public APIs
   - Semantic versioning commitment
   - Deprecation policy

2. **Performance**
   - <1% overhead for simple aspects
   - <5% overhead for complex aspects
   - Compilation time <2% increase

3. **Testing**
   - 95%+ code coverage
   - Comprehensive integration tests
   - Real-world stress testing
   - Fuzz testing

4. **Documentation**
   - Complete API documentation
   - Tutorial series
   - Migration guides
   - Best practices guide

5. **Tooling**
   - IDE integration stable
   - Cargo plugin released
   - Build tool integration

**Deliverables:**
- aspect-rs v1.0.0
- Published to crates.io
- Announcement blog post
- Conference talk submissions

**Timeline:** 12 months

### Priority 2: Ecosystem Growth

**Goal:** Build community and ecosystem.

**Community:**

1. **Contribution Infrastructure**
   - Contributor guide
   - Code of conduct
   - Issue templates
   - PR review process

2. **Communication Channels**
   - Discord server
   - GitHub Discussions
   - Blog/newsletter
   - Twitter/social media

3. **Events**
   - Conference talks
   - Workshops
   - Webinars
   - Meetups

**Ecosystem:**

1. **Third-Party Aspects**
   - Tracing integration (OpenTelemetry)
   - Metrics (Prometheus)
   - Logging (tracing, log)
   - Async runtime integration

2. **Framework Integration**
   - Axum support
   - Actix-web support
   - Rocket support
   - Warp support

3. **Tool Ecosystem**
   - cargo-aspect plugin
   - Benchmarking tools
   - Aspect profiler
   - Visualization tools

**Deliverables:**
- Active community
- Third-party aspects library
- Framework integrations
- Tool ecosystem

**Timeline:** Ongoing

### Priority 3: Research Features

**Goal:** Explore advanced AOP capabilities.

**Research Areas:**

1. **Around Advice**
   ```rust
   impl Aspect for AroundAspect {
       fn around(&self, ctx: &JoinPoint, proceed: Proceed) -> Result<Any> {
           // Full control over execution
           if should_skip(ctx) {
               return Ok(cached_value);
           }
           proceed.call()
       }
   }
   ```

2. **Inter-Type Declarations**
   ```rust
   // Add methods to existing types
   aspect! {
       impl User {
           fn validate(&self) -> bool { }
       }
   }
   ```

3. **Compile-Time Aspect Selection**
   ```rust
   // Different aspects per build profile
   #[cfg_attr(debug_assertions, aspect(VerboseLoggingAspect))]
   #[cfg_attr(release, aspect(MinimalLoggingAspect))]
   fn my_function() { }
   ```

4. **Aspect State Management**
   ```rust
   // Stateful aspects with safe access
   impl Aspect for StatefulAspect {
       type State = AtomicU64;
       
       fn before(&self, state: &Self::State, ctx: &JoinPoint) {
           state.fetch_add(1, Ordering::Relaxed);
       }
   }
   ```

**Deliverables:**
- Prototype implementations
- Research papers
- Experimental features
- Community feedback

**Timeline:** Ongoing research

## Feature Requests from Community

### High Demand

1. **Async Aspects**
   - Full async/await support
   - Async before/after hooks
   - Concurrent aspect execution

2. **Better Error Propagation**
   - Custom error types in aspects
   - Error transformation
   - Automatic retry on errors

3. **Performance Profiling**
   - Built-in profiling aspects
   - Flamegraph generation
   - Bottleneck detection

### Under Consideration

1. **Dynamic Aspects**
   - Runtime aspect enable/disable
   - Hot-reload aspect configuration
   - A/B testing support

2. **Aspect Templates**
   - Reusable aspect patterns
   - Parameterized aspects
   - Aspect libraries

3. **Cross-Language Support**
   - FFI aspect support
   - Interop with C/C++
   - WebAssembly integration

## Deprecation Timeline

### v0.4
- None (fully backward compatible)

### v0.5
- Deprecate CLI-only configuration (favor config files)
- Deprecate legacy pointcut syntax

### v1.0
- Remove deprecated features
- Finalize API surface

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | Week 4 | Phase 1: Basic AOP |
| 0.2.0 | Week 8 | Phase 2: Production features |
| 0.3.0 | Week 14 | Phase 3: MIR extraction |
| 0.4.0 | +3 months | Code generation |
| 0.5.0 | +6 months | IDE integration |
| 1.0.0 | +12 months | Stable release |

## Success Metrics

### v0.4 Goals
- 100+ GitHub stars
- 10+ external contributors
- 5+ production deployments
- 1,000+ downloads from crates.io

### v0.5 Goals
- 500+ GitHub stars
- 50+ external contributors
- 25+ production deployments
- 10,000+ downloads

### v1.0 Goals
- 2,000+ GitHub stars
- 100+ external contributors
- 100+ production deployments
- 100,000+ downloads
- Conference presentations
- Rust blog features

## How to Contribute

See [Contributing Guide](./contributing.md) for detailed information.

**Priority Areas:**
1. Code generation implementation
2. IDE integration
3. Documentation improvements
4. Standard aspect additions
5. Example applications

## Key Takeaways

1. **Short term:** Complete automatic weaving with code generation
2. **Medium term:** IDE integration and advanced pointcuts
3. **Long term:** Stable v1.0 and ecosystem growth
4. **Community:** Active contribution and ecosystem development
5. **Innovation:** Research features and advanced capabilities

---

**Related Chapters:**
- [Chapter 11.1: Achievements](./achievements.md) - What we've built
- [Chapter 11.3: Vision](./vision.md) - Long-term direction
- [Chapter 11.4: Contributing](./contributing.md) - How to help
