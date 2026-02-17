# aspect-driver

Compiler integration for automatic aspect weaving using rustc-driver.

## Status

**Phase 3 Week 2 - Design Prototype**

This crate contains the design and structure for rustc-driver integration. Full implementation requires:

- **Nightly Rust** (unstable compiler APIs)
- **rustc internal crates** (not published on crates.io)
- **Specific version pinning** (compiler APIs change frequently)

## What's Included

### ✅ Complete Design (`DESIGN.md`)
- Detailed architecture documentation
- Implementation phases breakdown
- Technical challenges and solutions
- Example code snippets
- Development roadmap

### ✅ Type Definitions (`types.rs`)
- `FunctionMetadata` - Complete function information
- `Visibility` - Public/crate/private levels
- `GenericParam` - Generic parameter information
- `SourceLocation` - File/line/column tracking
- `MatchedFunction` - Pointcut matching results

### ✅ Extraction Structure (`extract.rs`)
- Function metadata extraction (prototype)
- Visibility detection
- Generic parameter handling
- Module path resolution
- Pointcut filtering

### ✅ Compiler Driver (`lib.rs`)
- `AspectConfig` - Configuration structure
- `AspectCallbacks` - Compiler callback skeleton
- `run_compiler()` - Entry point structure
- Full documentation of required implementation

### ✅ Tests
- 9 unit tests (all passing)
- Pattern matching tests
- Visibility tests
- Module matching tests

## Architecture

```
User Code
    ↓
cargo aspect build
    ↓
aspect-driver (this crate)
    ↓
rustc (with callbacks)
    ↓
MIR Analysis → Metadata Extraction
    ↓
Pointcut Matching
    ↓
Aspect Weaving
    ↓
Optimized Binary
```

## API Overview

### Configuration

```rust
use aspect_driver::AspectConfig;

let config = AspectConfig {
    verbose: true,
    pointcuts: vec!["execution(pub fn *(..))".to_string()],
    aspects: vec!["LoggingAspect".to_string()],
    source_files: vec!["src/main.rs".into()],
};
```

### Function Metadata

```rust
use aspect_driver::types::FunctionMetadata;

// Extracted from compiler MIR
let metadata = FunctionMetadata {
    name: "my_crate::api::fetch_user".to_string(),
    module_path: "my_crate::api".to_string(),
    visibility: Visibility::Public,
    is_async: false,
    is_const: false,
    generics: vec![],
    return_type: "User".to_string(),
    location: SourceLocation {
        file: "src/api.rs".to_string(),
        line: 42,
        column: 1,
    },
    is_trait_method: false,
    trait_name: None,
};

// Pattern matching
assert!(metadata.matches_name_pattern("fetch_*"));
assert!(metadata.is_in_module("my_crate::api"));
assert!(metadata.is_public());
```

## Full Implementation Requirements

### 1. Nightly Toolchain

```bash
rustup toolchain install nightly
rustup override set nightly
```

### 2. rustc Dependencies

```toml
[dependencies]
# These require git dependencies to rust-lang/rust
rustc_driver = { git = "https://github.com/rust-lang/rust", rev = "..." }
rustc_interface = { git = "https://github.com/rust-lang/rust", rev = "..." }
rustc_middle = { git = "https://github.com/rust-lang/rust", rev = "..." }
rustc_hir = { git = "https://github.com/rust-lang/rust", rev = "..." }
rustc_span = { git = "https://github.com/rust-lang/rust", rev = "..." }
```

### 3. Implement Callbacks

See `DESIGN.md` for complete implementation guide with examples.

## Development Phases

### Week 2: Setup & Extraction (Current)
- [x] Create crate structure
- [x] Define type system
- [x] Design extraction API
- [x] Document architecture
- [ ] Implement rustc callbacks (requires nightly)
- [ ] Extract basic metadata (requires nightly)

### Week 3: Pointcut Matching
- [ ] Load aspect registry
- [ ] Parse pointcut expressions
- [ ] Match functions to aspects
- [ ] Handle generics
- [ ] Handle trait methods

### Week 4: Code Weaving
- [ ] Design MIR injection
- [ ] Implement before/after
- [ ] Implement around advice
- [ ] Preserve semantics
- [ ] Test generated code

## Examples

See `DESIGN.md` for detailed examples of:
- Compiler callback implementation
- MIR traversal
- Metadata extraction
- Type information queries
- Source location mapping

## Testing

```bash
# Run tests (works on stable Rust)
cargo test -p aspect-driver

# All 9 tests pass
```

## Limitations

### Current (Prototype)
- Cannot actually compile code (requires nightly + rustc)
- Returns placeholder data
- No real MIR access

### With Full Implementation
- Requires nightly Rust
- Compiler API instability
- Version-specific code
- Complex setup

## Next Steps

To complete Week 2:

1. **Set up nightly Rust**
2. **Add rustc dependencies** (pin to specific commit)
3. **Implement `AspectCallbacks`** trait
4. **Access TyCtxt** in callbacks
5. **Extract function list**
6. **Print function metadata**

See `DESIGN.md` Section "Implementation Phases" for detailed steps.

## References

- [DESIGN.md](./DESIGN.md) - Complete design documentation
- [rustc dev guide](https://rustc-dev-guide.rust-lang.org/)
- [MIR documentation](https://rustc-dev-guide.rust-lang.org/mir/index.html)
- [cargo-miri](https://github.com/rust-lang/miri) - Similar approach

## Contributing

This is infrastructure for Phase 3 of aspect-rs. See main project README for contribution guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
