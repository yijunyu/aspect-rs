# Phase 3 Architecture

Phase 3 introduces automatic aspect weaving through deep integration with the Rust compiler infrastructure, eliminating the need for manual `#[aspect]` annotations.

## System Overview

```
User Code (No Annotations)
    ↓
aspect-rustc-driver (Custom Rust Compiler Driver)
    ↓
Compiler Pipeline Integration
    ↓
MIR Extraction & Analysis
    ↓
Pointcut Expression Matching
    ↓
Code Generation & Weaving
    ↓
Optimized Binary with Aspects
```

## Core Components

### 1. aspect-rustc-driver

Custom compiler driver wrapping `rustc_driver`:

```rust
// aspect-rustc-driver/src/main.rs
use rustc_driver::RunCompiler;
use rustc_interface::interface;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (aspect_args, rustc_args) = parse_args(&args);
    
    let mut callbacks = AspectCallbacks::new(aspect_args);
    let exit_code = RunCompiler::new(&rustc_args, &mut callbacks).run();
    
    std::process::exit(exit_code.unwrap_or(1));
}
```

**Responsibilities:**
- Parse aspect-specific arguments (`--aspect-pointcut`, etc.)
- Inject custom compiler callbacks
- Run standard Rust compilation with aspect hooks
- Generate analysis reports

### 2. Compiler Callbacks

Integration points with Rust compilation:

```rust
impl Callbacks for AspectCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        // Override query provider for analysis phase
        config.override_queries = Some(|_sess, providers| {
            providers.analysis = analyze_crate_with_aspects;
        });
    }
    
    fn after_analysis(&mut self, compiler: &Compiler, queries: &Queries) {
        queries.global_ctxt().unwrap().enter(|tcx| {
            // Access to type context for MIR inspection
            self.analyze_and_weave(tcx);
        });
    }
}
```

### 3. MIR Analyzer

Extracts function metadata from compiled MIR:

```rust
pub struct MirAnalyzer<'tcx> {
    tcx: TyCtxt<'tcx>,
    verbose: bool,
}

impl<'tcx> MirAnalyzer<'tcx> {
    pub fn extract_all_functions(&self) -> Vec<FunctionMetadata> {
        let mut functions = Vec::new();
        
        for item_id in self.tcx.hir().items() {
            let item = self.tcx.hir().item(item_id);
            if let ItemKind::Fn(sig, generics, _body_id) = &item.kind {
                let metadata = self.extract_metadata(item, sig);
                functions.push(metadata);
            }
        }
        
        functions
    }
}
```

**Extracted Information:**
- Function name (simple and qualified)
- Module path
- Visibility (pub, pub(crate), private)
- Async status
- Generic parameters
- Source location (file, line)
- Return type information

### 4. Pointcut Matcher

Matches functions against pointcut expressions:

```rust
pub struct PointcutMatcher {
    pointcuts: Vec<Pointcut>,
}

impl PointcutMatcher {
    pub fn matches(&self, func: &FunctionMetadata) -> Vec<&Pointcut> {
        self.pointcuts
            .iter()
            .filter(|pc| self.evaluate_pointcut(pc, func))
            .collect()
    }
    
    fn evaluate_pointcut(&self, pc: &Pointcut, func: &FunctionMetadata) -> bool {
        match pc {
            Pointcut::Execution(pattern) => self.matches_execution(pattern, func),
            Pointcut::Within(module) => self.matches_within(module, func),
            Pointcut::Call(name) => self.matches_call(name, func),
        }
    }
}
```

### 5. Code Generator

Generates aspect weaving code:

```rust
pub struct AspectCodeGenerator;

impl AspectCodeGenerator {
    pub fn generate_wrapper(
        &self,
        func: &FunctionMetadata,
        aspects: &[AspectInfo]
    ) -> TokenStream {
        quote! {
            #[inline(never)]
            fn __aspect_original_{name}(#params) -> #ret_type {
                #original_body
            }
            
            #[inline(always)]
            pub fn {name}(#params) -> #ret_type {
                let ctx = JoinPoint { /* ... */ };
                #(#aspect_before_calls)*
                let result = __aspect_original_{name}(#args);
                #(#aspect_after_calls)*
                result
            }
        }
    }
}
```

## Data Flow

### 1. Compilation Start

```
rustc my_crate.rs
    ↓
aspect-rustc-driver intercepts
    ↓
Parse pointcut arguments
    ↓
Initialize AspectCallbacks
```

### 2. Analysis Phase

```
Compiler runs HIR → MIR lowering
    ↓
MIR available for inspection
    ↓
analyze_crate_with_aspects() called
    ↓
MirAnalyzer extracts functions
    ↓
PointcutMatcher evaluates expressions
    ↓
Build match results
```

### 3. Code Generation

```
For each matched function:
    ↓
Generate wrapper function
    ↓
Rename original to __aspect_original_{name}
    ↓
Inject aspect calls in wrapper
    ↓
Emit modified code
```

### 4. Compilation Complete

```
Standard LLVM optimization
    ↓
Link final binary
    ↓
Generate analysis report
    ↓
Exit with status code
```

## Global State Management

Challenge: Query providers must be static functions, but need configuration data.

**Solution: Global state with synchronization**

```rust
// Global configuration storage
static CONFIG: Mutex<Option<AspectConfig>> = Mutex::new(None);
static RESULTS: Mutex<Option<AnalysisResults>> = Mutex::new(None);

// Function pointer for query provider
fn analyze_crate_with_aspects(tcx: TyCtxt<'_>, (): ()) {
    // Extract config from global state
    let config = CONFIG.lock().unwrap().clone().unwrap();
    
    // Perform analysis
    let analyzer = MirAnalyzer::new(tcx, config.verbose);
    let functions = analyzer.extract_all_functions();
    
    // Match pointcuts
    let matcher = PointcutMatcher::new(config.pointcuts);
    let matches = matcher.match_all(&functions);
    
    // Store results back to global
    *RESULTS.lock().unwrap() = Some(matches);
}

// Callbacks setup
impl AspectCallbacks {
    fn new(config: AspectConfig) -> Self {
        // Store config in global
        *CONFIG.lock().unwrap() = Some(config.clone());
        Self { config }
    }
}
```

**Why this works:**
- Function pointers have no closures (required by rustc)
- Global state accessible from static function
- Mutex ensures thread safety
- Clean separation of concerns

## Integration Points

### rustc_driver Integration

```rust
use rustc_driver::{Callbacks, Compilation, RunCompiler};
use rustc_interface::{interface, Queries};

pub struct AspectCallbacks {
    config: AspectConfig,
}

impl Callbacks for AspectCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        // Customize compiler configuration
        config.override_queries = Some(override_queries);
    }
    
    fn after_expansion(
        &mut self,
        _compiler: &interface::Compiler,
        _queries: &Queries<'_>
    ) -> Compilation {
        // Continue compilation
        Compilation::Continue
    }
    
    fn after_analysis(
        &mut self,
        compiler: &interface::Compiler,
        queries: &Queries<'_>
    ) -> Compilation {
        // Perform aspect analysis
        queries.global_ctxt().unwrap().enter(|tcx| {
            analyze_with_tcx(tcx);
        });
        
        Compilation::Continue
    }
}
```

### TyCtxt Access

```rust
fn analyze_with_tcx(tcx: TyCtxt<'_>) {
    // Access to complete type information
    for item_id in tcx.hir().items() {
        let item = tcx.hir().item(item_id);
        let def_id = item.owner_id.to_def_id();
        
        // Get function signature
        let sig = tcx.fn_sig(def_id);
        
        // Get MIR if available
        if tcx.is_mir_available(def_id) {
            let mir = tcx.optimized_mir(def_id);
            // Analyze MIR...
        }
    }
}
```

## Crate Structure

```
aspect-rs/
├── aspect-rustc-driver/        # Main driver binary
│   ├── src/
│   │   ├── main.rs            # Entry point, argument parsing
│   │   ├── callbacks.rs       # Compiler callbacks
│   │   └── analysis.rs        # Analysis orchestration
│   └── Cargo.toml
│
├── aspect-driver/             # Shared analysis logic
│   ├── src/
│   │   ├── lib.rs
│   │   ├── mir_analyzer.rs    # MIR extraction
│   │   ├── pointcut_matcher.rs # Expression evaluation
│   │   ├── code_generator.rs  # Wrapper generation
│   │   └── types.rs           # Shared data structures
│   └── Cargo.toml
│
└── aspect-core/              # Runtime aspects (unchanged)
    └── ...
```

## Configuration Schema

### Command-Line Arguments

```bash
aspect-rustc-driver [OPTIONS] <INPUT> [RUSTC_ARGS...]

OPTIONS:
  --aspect-pointcut <EXPR>    Pointcut expression to match
  --aspect-apply <ASPECT>     Aspect to apply to matches
  --aspect-output <FILE>      Write analysis report
  --aspect-verbose            Verbose output
  --aspect-config <FILE>      Load config from file
```

### Configuration File

```toml
# aspect-config.toml

[[pointcuts]]
pattern = "execution(pub fn *(..))"
aspects = ["LoggingAspect::new()"]

[[pointcuts]]
pattern = "within(crate::api)"
aspects = ["SecurityAspect::new()", "AuditAspect::new()"]

[options]
verbose = true
output = "target/aspect-analysis.txt"
```

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| MIR extraction | O(n) | n = number of functions |
| Pointcut matching | O(n × m) | n = functions, m = pointcuts |
| Code generation | O(k) | k = matched functions |
| Compile overhead | +2-5% | Varies by project size |

**Total compilation impact:** 2-5% increase for typical projects.

## Error Handling

### Compilation Errors

```rust
impl AspectCallbacks {
    fn report_error(&self, span: Span, message: &str) {
        self.sess.struct_span_err(span, message)
            .emit();
    }
}

// Usage
if !pointcut.is_valid() {
    self.report_error(span, "Invalid pointcut expression");
}
```

### Analysis Errors

```rust
fn analyze_function(&self, func: &Item) -> Result<FunctionMetadata, AnalysisError> {
    let name = func.ident.to_string();
    
    if name.is_empty() {
        return Err(AnalysisError::InvalidFunction("Empty function name"));
    }
    
    // Extract metadata...
    Ok(metadata)
}
```

## Key Architectural Decisions

1. **Function Pointers + Global State**
   - Required by rustc query system
   - Enables static function with dynamic configuration
   - Thread-safe via Mutex

2. **MIR-Level Analysis**
   - Access to compiled, type-checked code
   - More reliable than AST parsing
   - Handles macros and generated code

3. **Separate Driver Binary**
   - Wraps standard rustc
   - Users install once, use like rustc
   - No rustup override needed

4. **Zero Runtime Overhead**
   - All analysis at compile-time
   - Generated code identical to manual annotations
   - No runtime aspect resolution

## Next Steps

- See [How It Works](./how-it-works.md) for detailed implementation
- See [Pointcuts](./pointcuts.md) for matching algorithm
- See [Breakthrough](./breakthrough.md) for technical achievement
- See [Comparison](./comparison.md) for Phase 1 vs 2 vs 3

---

**Related Chapters:**
- [Chapter 8.5: Automatic Weaving](../ch08-case-studies/automatic-weaving.md)
- [Chapter 10.2: How It Works](./how-it-works.md)
- [Chapter 11: Future](../ch11-future/README.md)
