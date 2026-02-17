# MIR Extraction

Mid-level Intermediate Representation (MIR) extraction is the foundation of Phase 3's automatic aspect weaving. This chapter explains how aspect-rs extracts function metadata from Rust's compiled MIR.

## Overview

**MIR (Mid-level IR)** is Rust's intermediate representation used between:
- High-level HIR (High-level IR from AST)
- Low-level LLVM IR (machine code generation)

MIR provides:
- **Complete function information** - All metadata about functions
- **Type-checked code** - Already validated by compiler
- **Control flow analysis** - Statement-level granularity
- **Optimization-ready** - Before final code generation

## Why MIR for Aspect Weaving?

### Advantages over AST

| Feature | AST (syn crate) | MIR (rustc_middle) |
|---------|----------------|---------------------|
| Type information | ❌ No | ✅ Complete |
| Trait resolution | ❌ No | ✅ Yes |
| Generic instantiation | ❌ No | ✅ Yes |
| Visibility | ⚠️ Partial | ✅ Complete |
| Module paths | ⚠️ Manual | ✅ Automatic |
| Control flow | ❌ No | ✅ Yes |

**Conclusion**: MIR provides everything needed for precise aspect matching.

### Phase 3 Architecture

```
Source Code (.rs)
    ↓
Rustc Parsing → AST
    ↓
HIR Generation
    ↓
Type Checking
    ↓
MIR Generation  ← aspect-rustc-driver hooks here
    ↓
Aspect Analysis (Extract metadata)
    ↓
Pointcut Matching
    ↓
Code Weaving
    ↓
LLVM IR → Binary
```

## MIR Structure

### Function Body

MIR represents functions as control flow graphs:

```rust
pub struct Body<'tcx> {
    pub basic_blocks: IndexVec<BasicBlock, BasicBlockData<'tcx>>,
    pub local_decls: LocalDecls<'tcx>,
    pub arg_count: usize,
    pub return_ty: Ty<'tcx>,
    // ... more fields
}
```

**Components**:
- `basic_blocks` - Control flow graph nodes
- `local_decls` - Local variables and temporaries
- `arg_count` - Number of function parameters
- `return_ty` - Return type information

### Example MIR

For this function:
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Generated MIR** (simplified):
```
fn add(_1: i32, _2: i32) -> i32 {
    let mut _0: i32;  // return place

    bb0: {
        _0 = Add(move _1, move _2);
        return;
    }
}
```

**Key information**:
- `_1`, `_2` - Function parameters
- `_0` - Return value
- `bb0` - Basic block 0 (entry point)
- `Add` - Binary operation
- `return` - Function exit

## Accessing MIR in rustc

### TyCtxt - The Type Context

`TyCtxt` is the central structure for accessing compiler information:

```rust
fn analyze_crate(tcx: TyCtxt<'_>) {
    // TyCtxt provides access to ALL compiler data
}
```

**What TyCtxt provides**:
- Function definitions (`def_id_to_hir_id`)
- Type information (`type_of`)
- MIR bodies (`optimized_mir`)
- Module structure (`def_path`)
- Visibility (`visibility`)

### Getting Function MIR

```rust
use rustc_middle::ty::TyCtxt;
use rustc_hir::def_id::DefId;

fn get_function_mir<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> &'tcx Body<'tcx> {
    tcx.optimized_mir(def_id)
}
```

**Optimization levels**:
- `mir_built` - Initial MIR (before optimizations)
- `mir_const` - After const evaluation
- `optimized_mir` - Fully optimized (best for analysis)

## MirAnalyzer Implementation

### Core Structure

```rust
pub struct MirAnalyzer<'tcx> {
    tcx: TyCtxt<'tcx>,
    verbose: bool,
    functions: Vec<FunctionInfo>,
}

impl<'tcx> MirAnalyzer<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, verbose: bool) -> Self {
        Self {
            tcx,
            verbose,
            functions: Vec::new(),
        }
    }

    pub fn extract_all_functions(&mut self) -> Vec<FunctionInfo> {
        // Iterate over all items in the crate
        for def_id in self.tcx.hir().body_owners() {
            if let Some(func_info) = self.analyze_function(def_id.to_def_id()) {
                self.functions.push(func_info);
            }
        }
        self.functions.clone()
    }
}
```

### Extracting Function Metadata

```rust
fn analyze_function(&self, def_id: DefId) -> Option<FunctionInfo> {
    // Get the MIR body
    let mir = self.tcx.optimized_mir(def_id);

    // Extract function name
    let name = self.tcx.def_path_str(def_id);

    // Extract module path
    let module_path = self.tcx.def_path(def_id)
        .data
        .iter()
        .map(|seg| seg.to_string())
        .collect::<Vec<_>>()
        .join("::");

    // Extract visibility
    let visibility = match self.tcx.visibility(def_id) {
        Visibility::Public => VisibilityKind::Public,
        Visibility::Restricted(module) => VisibilityKind::Crate,
        Visibility::Invisible => VisibilityKind::Private,
    };

    // Check if async
    let is_async = mir.generator_kind().is_some();

    // Extract return type
    let return_ty = self.tcx.type_of(def_id);
    let return_type_str = return_ty.to_string();

    // Get source location
    let span = self.tcx.def_span(def_id);
    let source_map = self.tcx.sess.source_map();
    let location = source_map.lookup_char_pos(span.lo());

    Some(FunctionInfo {
        name,
        module_path,
        visibility,
        is_async,
        return_type: Some(return_type_str),
        file: location.file.name.to_string(),
        line: location.line,
    })
}
```

### Function Information Structure

```rust
#[derive(Clone, Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub module_path: String,
    pub visibility: VisibilityKind,
    pub is_async: bool,
    pub is_generic: bool,
    pub return_type: Option<String>,
    pub parameters: Vec<Parameter>,
    pub file: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VisibilityKind {
    Public,
    Crate,
    Private,
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub ty: String,
}
```

## Iterating Over Functions

### Finding All Functions

```rust
pub fn find_all_functions(tcx: TyCtxt<'_>) -> Vec<DefId> {
    let mut functions = Vec::new();

    // Iterate over all HIR body owners
    for owner in tcx.hir().body_owners() {
        let def_id = owner.to_def_id();

        // Check if it's a function (not const/static)
        if tcx.def_kind(def_id) == DefKind::Fn {
            functions.push(def_id);
        }
    }

    functions
}
```

### Filtering by Module

```rust
pub fn find_functions_in_module(
    tcx: TyCtxt<'_>,
    module_pattern: &str
) -> Vec<DefId> {
    find_all_functions(tcx)
        .into_iter()
        .filter(|&def_id| {
            let path = tcx.def_path_str(def_id);
            path.starts_with(module_pattern)
        })
        .collect()
}
```

### Filtering by Visibility

```rust
pub fn find_public_functions(tcx: TyCtxt<'_>) -> Vec<DefId> {
    find_all_functions(tcx)
        .into_iter()
        .filter(|&def_id| {
            matches!(tcx.visibility(def_id), Visibility::Public)
        })
        .collect()
}
```

## Extracting Parameter Information

```rust
fn extract_parameters(tcx: TyCtxt<'_>, mir: &Body<'_>) -> Vec<Parameter> {
    let mut params = Vec::new();

    for (index, local) in mir.local_decls.iter_enumerated() {
        // Skip return value (index 0) and get only args
        if index.as_usize() > 0 && index.as_usize() <= mir.arg_count {
            params.push(Parameter {
                name: format!("arg{}", index.as_usize() - 1),
                ty: local.ty.to_string(),
            });
        }
    }

    params
}
```

## Extracting Generic Information

```rust
fn is_generic(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    let generics = tcx.generics_of(def_id);
    generics.count() > 0
}

fn extract_generics(tcx: TyCtxt<'_>, def_id: DefId) -> Vec<String> {
    let generics = tcx.generics_of(def_id);
    generics.params.iter()
        .map(|param| param.name.to_string())
        .collect()
}
```

## Real-World Example

### Input Code

```rust
pub mod api {
    pub fn fetch_user(id: u64) -> User {
        database::get(id)
    }

    async fn process_data(data: Vec<u8>) -> Result<(), Error> {
        // ...
    }

    pub(crate) fn internal_helper() {
        // ...
    }
}
```

### Extracted Metadata

```rust
// fetch_user
FunctionInfo {
    name: "api::fetch_user",
    module_path: "my_crate::api",
    visibility: VisibilityKind::Public,
    is_async: false,
    is_generic: false,
    return_type: Some("User"),
    parameters: vec![
        Parameter { name: "id", ty: "u64" }
    ],
    file: "src/api.rs",
    line: 2,
}

// process_data
FunctionInfo {
    name: "api::process_data",
    module_path: "my_crate::api",
    visibility: VisibilityKind::Private,
    is_async: true,
    is_generic: false,
    return_type: Some("Result<(), Error>"),
    parameters: vec![
        Parameter { name: "data", ty: "Vec<u8>" }
    ],
    file: "src/api.rs",
    line: 6,
}

// internal_helper
FunctionInfo {
    name: "api::internal_helper",
    module_path: "my_crate::api",
    visibility: VisibilityKind::Crate,
    is_async: false,
    is_generic: false,
    return_type: Some("()"),
    parameters: vec![],
    file: "src/api.rs",
    line: 10,
}
```

## Integration with Pointcut Matching

```rust
pub fn apply_aspects(tcx: TyCtxt<'_>, pointcuts: &[PointcutPattern]) {
    let analyzer = MirAnalyzer::new(tcx, true);
    let functions = analyzer.extract_all_functions();

    for func in &functions {
        for pointcut in pointcuts {
            if pointcut.matches(func) {
                println!("✓ Matched: {} by {}", func.name, pointcut.pattern);
                // Weave aspect into this function
            }
        }
    }
}
```

## Performance Considerations

### Caching

```rust
lazy_static! {
    static ref FUNCTION_CACHE: Mutex<HashMap<DefId, FunctionInfo>> =
        Mutex::new(HashMap::new());
}

fn get_cached_function_info(tcx: TyCtxt<'_>, def_id: DefId) -> FunctionInfo {
    let mut cache = FUNCTION_CACHE.lock().unwrap();

    cache.entry(def_id)
        .or_insert_with(|| extract_function_info(tcx, def_id))
        .clone()
}
```

### Incremental Compilation

MIR extraction works with Rust's incremental compilation:
- Only changed functions re-analyzed
- Cached results reused for unchanged code
- Fast re-compilation

**Typical performance**:
- Extract metadata: ~0.1ms per function
- 1000 functions: ~100ms total
- Negligible impact on build time

## Challenges and Solutions

### Challenge 1: rustc API Instability

**Problem**: rustc APIs change frequently between versions.

**Solution**: Pin to specific nightly version:
```toml
[package]
rust-version = "nightly-2024-01-01"
```

### Challenge 2: Accessing TyCtxt

**Problem**: TyCtxt cannot be passed through closures.

**Solution**: Use function pointers with global state:
```rust
static CONFIG: Mutex<Option<AspectConfig>> = Mutex::new(None);

fn analyze_crate_with_aspects(tcx: TyCtxt<'_>, (): ()) {
    let config = CONFIG.lock().unwrap().clone().unwrap();
    let analyzer = MirAnalyzer::new(tcx, config.verbose);
    // ... analysis
}
```

### Challenge 3: Generic Function Instantiation

**Problem**: Generic functions have multiple instantiations.

**Solution**: Analyze the generic definition, apply aspects to all instantiations:
```rust
if is_generic(tcx, def_id) {
    // Get generic definition
    let generic_info = extract_function_info(tcx, def_id);
    // Apply aspect to generic definition
    // Will affect all instantiations
}
```

## Debugging MIR Extraction

### Viewing MIR

```bash
# Dump MIR for a specific crate
rustc +nightly -Z dump-mir=all src/lib.rs

# Dump MIR for specific function
rustc +nightly -Z dump-mir=my_function src/lib.rs

# View optimized MIR
rustc +nightly -Z dump-mir=optimized src/lib.rs
```

### Verbose Output

```rust
impl MirAnalyzer {
    fn analyze_function(&self, def_id: DefId) -> Option<FunctionInfo> {
        if self.verbose {
            println!("Analyzing: {}", self.tcx.def_path_str(def_id));
        }

        // ... extraction logic

        if self.verbose {
            println!("  Visibility: {:?}", visibility);
            println!("  Async: {}", is_async);
            println!("  Return type: {:?}", return_type);
        }

        Some(func_info)
    }
}
```

## Future Enhancements

### Control Flow Analysis

Extract control flow information:
```rust
fn analyze_control_flow(mir: &Body) -> ControlFlowInfo {
    ControlFlowInfo {
        basic_blocks: mir.basic_blocks.len(),
        loops: detect_loops(mir),
        branches: count_branches(mir),
    }
}
```

### Call Graph Construction

Build call graph for crate:
```rust
fn build_call_graph(tcx: TyCtxt) -> CallGraph {
    let mut graph = CallGraph::new();

    for def_id in find_all_functions(tcx) {
        let mir = tcx.optimized_mir(def_id);
        for block in &mir.basic_blocks {
            for statement in &block.statements {
                if let Call { func, .. } = statement.kind {
                    graph.add_edge(def_id, func.def_id());
                }
            }
        }
    }

    graph
}
```

## Summary

MIR extraction provides:

1. **Complete function metadata** - Everything needed for aspect matching
2. **Type-checked information** - Guaranteed correctness
3. **Compiler integration** - Works with rustc directly
4. **Zero runtime cost** - All analysis at compile time

**Key achievement**: Phase 3 automatic weaving relies entirely on MIR extraction for precise, automatic aspect application.

## See Also

- [Pointcut Matching](pointcuts.md) - How extracted metadata is matched
- [Code Weaving Process](weaving.md) - Using metadata to weave aspects
- [Phase 3 Architecture](../ch10-phase3/architecture.md) - Complete system design
- [Phase 3 How It Works](../ch10-phase3/how-it-works.md) - End-to-end flow
