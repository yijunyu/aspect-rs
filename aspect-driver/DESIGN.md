# aspect-driver Design Document

## Overview

The `aspect-driver` crate provides rustc compiler integration for automatic aspect weaving based on pointcuts.

## Architecture

### High-Level Flow

```
User Code
    ↓
cargo aspect build
    ↓
aspect-driver (custom compiler driver)
    ↓
rustc (with callbacks)
    ↓
MIR Analysis → Function Metadata
    ↓
Pointcut Matching
    ↓
Aspect Weaving (code injection)
    ↓
Optimized Binary
```

## Components

### 1. Compiler Driver (`lib.rs`)

**Purpose**: Replace standard rustc with custom driver

**Key Functions**:
- `run_compiler()` - Entry point from cargo-aspect
- `register_callbacks()` - Hook into compilation phases
- `configure_compiler()` - Set up compiler options

**Dependencies**:
```toml
rustc_driver = { git = "https://github.com/rust-lang/rust" }
rustc_interface = { git = "https://github.com/rust-lang/rust" }
rustc_middle = { git = "https://github.com/rust-lang/rust" }
```

### 2. Metadata Extraction (`extract.rs`)

**Purpose**: Extract function information from MIR

**Extracted Data**:
```rust
pub struct FunctionMetadata {
    pub name: String,
    pub module_path: String,
    pub visibility: Visibility,
    pub is_async: bool,
    pub generics: Vec<GenericParam>,
    pub return_type: Type,
    pub location: SourceLocation,
}
```

**Process**:
1. Access MIR for each function
2. Extract DefId (definition ID)
3. Query type information
4. Build FunctionMetadata

### 3. Pointcut Matching (`match.rs` - Week 3)

**Purpose**: Match functions against pointcut expressions

**Process**:
1. Load registered aspects from registry
2. Parse pointcut expressions
3. Match against FunctionMetadata
4. Return list of (Function, Aspect) pairs

### 4. Code Generation (`generate.rs` - Week 4)

**Purpose**: Inject aspect code into MIR

**Approach**:
- Insert aspect calls at function entry/exit
- Wrap function body in ProceedingJoinPoint
- Preserve function signature and semantics

## Implementation Phases

### Phase 1: Basic Setup (Week 2)

**Goal**: Access MIR and extract basic information

**Steps**:
1. Create compiler driver skeleton
2. Register compilation callbacks
3. Access HIR (High-level IR)
4. Access MIR (Mid-level IR)
5. Extract function names and modules

**Output**:
```bash
$ cargo aspect build
Analyzing functions...
  Found: my_crate::api::get_user (public)
  Found: my_crate::api::create_user (public)
  Found: my_crate::internal::helper (private)
Total: 3 functions
```

### Phase 2: Metadata Extraction (Week 2-3)

**Goal**: Extract complete function information

**Data Points**:
- Function name (qualified path)
- Module path
- Visibility (pub, pub(crate), private)
- Async/sync
- Generic parameters
- Return type
- Source location

### Phase 3: Pointcut Matching (Week 3)

**Goal**: Match functions to aspects

**Example**:
```rust
// Pointcut: execution(pub fn *(..))
// Matches: my_crate::api::get_user, my_crate::api::create_user
// Skips: my_crate::internal::helper (private)
```

### Phase 4: Code Weaving (Week 4)

**Goal**: Inject aspect code

**Transformation**:
```rust
// Original
pub fn get_user(id: u64) -> User {
    database::fetch(id)
}

// After weaving (conceptual)
pub fn get_user(id: u64) -> User {
    let aspect = LoggingAspect::new();
    let ctx = JoinPoint { ... };
    aspect.before(&ctx);
    let result = {
        database::fetch(id)
    };
    aspect.after(&ctx, &result);
    result
}
```

## Technical Challenges

### 1. Unstable APIs

**Problem**: rustc APIs are unstable and change frequently

**Solution**:
- Pin to specific rustc version
- Document required nightly version
- Provide version compatibility matrix

### 2. MIR Modification

**Problem**: MIR is designed for analysis, not modification

**Alternatives**:
1. **HIR Modification** (easier but less precise)
2. **MIR Injection** (harder but more powerful)
3. **LLVM IR** (very low-level, harder still)

**Chosen**: Start with HIR, move to MIR later

### 3. Type Erasure

**Problem**: Generic types are monomorphized late

**Solution**:
- Match on generic type patterns
- Apply aspects to all instantiations
- Use trait bounds for filtering

### 4. Macro Expansion

**Problem**: Macros expand before our analysis

**Solution**:
- Analyze post-expansion HIR
- Track source locations through expansion
- Allow pointcuts to filter by macro origin

## rustc Compiler Phases

```
Source Code
    ↓
Lexing & Parsing
    ↓
AST (Abstract Syntax Tree)
    ↓
Macro Expansion
    ↓
HIR (High-level IR)  ← We can hook here
    ↓
Type Checking
    ↓
MIR (Mid-level IR)   ← Primary hook point
    ↓
Monomorphization
    ↓
LLVM IR
    ↓
Machine Code
```

**Our Hook Points**:
1. **After MIR construction** - Extract metadata
2. **Before MIR optimization** - Inject aspect code
3. **After type checking** - Safe to analyze types

## Example: Compiler Callback

```rust
use rustc_driver::{Compilation, RunCompiler};
use rustc_interface::{interface, Queries};
use rustc_middle::ty::TyCtxt;

struct AspectCallbacks;

impl rustc_driver::Callbacks for AspectCallbacks {
    fn after_parsing<'tcx>(
        &mut self,
        compiler: &interface::Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        queries.global_ctxt().unwrap().enter(|tcx| {
            // Access type context
            analyze_functions(tcx);
        });
        Compilation::Continue
    }
}

fn analyze_functions<'tcx>(tcx: TyCtxt<'tcx>) {
    // Iterate over all items in the crate
    for (def_id, _) in tcx.hir().items() {
        if let Some(fn_sig) = tcx.fn_sig(def_id) {
            println!("Found function: {:?}", tcx.def_path_str(def_id));
        }
    }
}
```

## Development Roadmap

### Week 2: Setup & Extraction
- [ ] Create aspect-driver crate
- [ ] Set up rustc dependencies
- [ ] Implement basic callback
- [ ] Extract function names
- [ ] Extract module paths
- [ ] Extract visibility

### Week 3: Matching
- [ ] Load aspect registry
- [ ] Parse pointcut expressions
- [ ] Match functions to aspects
- [ ] Handle generics
- [ ] Handle trait methods

### Week 4: Weaving
- [ ] Design MIR injection strategy
- [ ] Implement before/after injection
- [ ] Implement around advice wrapping
- [ ] Preserve function semantics
- [ ] Test generated code

### Week 5-6: Advanced Features
- [ ] Field access interception
- [ ] Call-site matching
- [ ] Cross-crate weaving
- [ ] Optimization passes

### Week 7-8: Polish
- [ ] Error messages
- [ ] Documentation
- [ ] Examples
- [ ] Integration tests

## Dependencies

### Required Nightly Version

```toml
[package]
rust-version = "nightly-2024-01-15"  # Example, pin to specific version
```

### Compiler Crates

```toml
[dependencies]
# These are git dependencies, not on crates.io
rustc_driver = { git = "https://github.com/rust-lang/rust", rev = "abc123" }
rustc_interface = { git = "https://github.com/rust-lang/rust", rev = "abc123" }
rustc_middle = { git = "https://github.com/rust-lang/rust", rev = "abc123" }
rustc_hir = { git = "https://github.com/rust-lang/rust", rev = "abc123" }
rustc_span = { git = "https://github.com/rust-lang/rust", rev = "abc123" }
```

### Supporting Crates

```toml
aspect-core = { workspace = true }
aspect-runtime = { workspace = true }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Testing Strategy

### Unit Tests
- Test metadata extraction on simple functions
- Test pointcut matching logic
- Test code generation utilities

### Integration Tests
```rust
#[test]
fn test_extract_public_functions() {
    let src = r#"
        pub fn public_fn() {}
        fn private_fn() {}
    "#;

    let metadata = extract_from_source(src);
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata[0].visibility, Visibility::Public);
    assert_eq!(metadata[1].visibility, Visibility::Private);
}
```

### End-to-End Tests
- Compile real crates with aspects
- Verify aspect code is executed
- Check performance overhead

## References

- [rustc dev guide](https://rustc-dev-guide.rust-lang.org/)
- [MIR documentation](https://rustc-dev-guide.rust-lang.org/mir/index.html)
- [Callbacks example](https://github.com/rust-lang/rust/tree/master/tests/run-make)
- [cargo-miri](https://github.com/rust-lang/miri) - Similar compiler driver
- [cargo-expand](https://github.com/dtolnay/cargo-expand) - Macro expansion tool
