# Call-Site Matching Design

**Phase 3 Week 6**

## Overview

Call-site matching intercepts function calls (not function definitions), enabling aspects to wrap any call to specific functions regardless of where they're defined.

## Key Difference

### execution() vs call()

**execution()** - Matches function definitions:
```rust
#[advice(pointcut = "execution(pub fn *(..))", advice = "before")]
// Applies to: The function definition itself
pub fn my_function() { } // ← Matched here
```

**call()** - Matches function calls:
```rust
#[advice(pointcut = "call(* database::*(..))", advice = "around")]
// Applies to: Any code calling database functions

fn process() {
    database::query();  // ← Matched here
    database::insert(); // ← Matched here
}
```

## Use Cases

### Automatic Retry for External Calls

```rust
#[advice(pointcut = "call(* api::*(..))", advice = "around")]
fn retry_api_calls(pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    for attempt in 1..=3 {
        match pjp.proceed() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < 3 => {
                log::warn!("API call failed (attempt {}), retrying...", attempt);
                std::thread::sleep(Duration::from_millis(100 * attempt));
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

// Automatically retries:
let user = api::fetch_user(42)?;  // Retries on failure
let data = api::get_data()?;       // Retries on failure
```

### Database Call Instrumentation

```rust
#[advice(pointcut = "call(* database::*(..))", advice = "around")]
fn instrument_db_calls(pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    let start = Instant::now();

    let result = pjp.proceed();

    let duration = start.elapsed();
    metrics::record_db_call(duration);

    if duration > Duration::from_millis(100) {
        log::warn!("Slow database call: {:?}", duration);
    }

    result
}

// All database calls automatically instrumented:
database::query_user(id);
database::insert_record(data);
database::update_field(key, value);
```

### Permission Checks for Sensitive Operations

```rust
#[advice(pointcut = "call(* admin::*(..))", advice = "before")]
fn check_admin_permission(ctx: &JoinPoint) {
    if !current_user().has_role("admin") {
        panic!("Unauthorized call to admin function: {}", ctx.target_function);
    }
}

// All admin function calls protected:
admin::delete_user(user_id);  // Checks permission
admin::modify_settings(cfg);   // Checks permission
```

## Pointcut Syntax

### Basic Patterns

```rust
// Call any function in module
"call(* database::*(..))"

// Call specific function anywhere
"call(* query_user(..))"

// Call with specific return type
"call(Result<*, Error> *(..))"

// Call methods on specific type
"call(* User::*(..))"

// Call trait methods
"call(* <T as Clone>::clone(..))"
```

### Composed Patterns

```rust
// Database calls in API module
"call(* database::*(..)  && within(crate::api)"

// External calls excluding internal
"call(* external::*(..) && !within(crate::internal)"

// Specific function in any module
"call(* query(..)) || call(* fetch(..))"
```

## MIR Representation

### Function Call

```rust
// Source code:
let result = database::query_user(42);

// MIR (simplified):
_2 = const database::query_user;
_3 = const 42_u64;
_1 = move _2(move _3) -> [return: bb1, unwind: bb2];
```

### Method Call

```rust
// Source code:
let cloned = user.clone();

// MIR (simplified):
_2 = &_1;  // borrow user
_3 = <User as Clone>::clone(move _2) -> [return: bb1, unwind: bb2];
```

## Detection Strategy

### Step 1: Find Call Sites in MIR

```ignore
use rustc_middle::mir::{Body, Terminator, TerminatorKind};
use rustc_middle::ty::TyCtxt;

fn find_call_sites<'tcx>(tcx: TyCtxt<'tcx>, body: &Body<'tcx>) -> Vec<CallSite> {
    let mut call_sites = Vec::new();

    for (block_idx, block) in body.basic_blocks.iter_enumerated() {
        if let Some(terminator) = &block.terminator {
            match &terminator.kind {
                TerminatorKind::Call {
                    func,
                    args,
                    destination,
                    ..
                } => {
                    // Extract function being called
                    if let Some(def_id) = func.const_fn_def() {
                        call_sites.push(CallSite {
                            def_id,
                            args: args.clone(),
                            location: terminator.source_info.span,
                            caller_function: body.source.def_id(),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    call_sites
}
```

### Step 2: Extract Call Metadata

```ignore
struct CallSiteMetadata {
    /// Function being called
    target_function: String,

    /// Module of target function
    target_module: String,

    /// Is this a method call?
    is_method: bool,

    /// Is this a trait method?
    is_trait_method: bool,

    /// Trait name (if trait method)
    trait_name: Option<String>,

    /// Function where call occurs
    caller_function: String,

    /// Source location
    location: SourceLocation,
}

fn extract_call_metadata<'tcx>(
    tcx: TyCtxt<'tcx>,
    call_site: &CallSite,
) -> CallSiteMetadata {
    let target_def_id = call_site.def_id;

    CallSiteMetadata {
        target_function: tcx.def_path_str(target_def_id),
        target_module: extract_module_path(tcx, target_def_id),
        is_method: tcx.is_method(target_def_id),
        is_trait_method: tcx.trait_of_item(target_def_id).is_some(),
        trait_name: tcx.trait_of_item(target_def_id)
            .map(|t| tcx.def_path_str(t)),
        caller_function: tcx.def_path_str(call_site.caller_function),
        location: extract_source_location(tcx, call_site.location),
    }
}
```

### Step 3: Match Against Pointcuts

```rust
fn matches_call_pointcut(call: &CallSiteMetadata, pointcut: &str) -> bool {
    // Parse: "call(* database::*(..))"
    let pattern = parse_call_pattern(pointcut)?;

    // Match module
    if !matches_module(&call.target_module, &pattern.module) {
        return false;
    }

    // Match function name
    if !matches_name(&call.target_function, &pattern.name) {
        return false;
    }

    // Match return type (if specified)
    if let Some(ret_pattern) = &pattern.return_type {
        if !matches_return_type(&call.target_function, ret_pattern) {
            return false;
        }
    }

    true
}
```

## Code Generation

### Strategy: Wrap Call Site

Transform each matched call site to go through an aspect wrapper.

#### Before Transformation

```rust
fn process_user(id: u64) {
    let user = database::query_user(id);
    println!("Got user: {:?}", user);
}
```

#### After Transformation

```rust
fn process_user(id: u64) {
    // Create JoinPoint for this call
    let ctx = CallSiteJoinPoint {
        target_function: "database::query_user",
        caller_function: "process_user",
        location: Location { ... },
    };

    // Wrap call in ProceedingJoinPoint
    let pjp = ProceedingJoinPoint::new(
        || {
            // Original call
            let result = database::query_user(id);
            Ok(Box::new(result) as Box<dyn Any>)
        },
        ctx
    );

    // Apply aspect
    let user = DatabaseRetryAspect::new().around(pjp)?.downcast().unwrap();

    println!("Got user: {:?}", user);
}
```

### MIR Transformation

Directly modify MIR to insert aspect calls:

```ignore
// Original MIR:
bb0: {
    _2 = const database::query_user;
    _1 = move _2(42_u64) -> [return: bb1, unwind: bb2];
}

// Transformed MIR:
bb0: {
    // Create JoinPoint
    _jp = const CallSiteJoinPoint { ... };

    // Create aspect
    _aspect = DatabaseRetryAspect::new();

    // Create ProceedingJoinPoint with original call
    _pjp = ProceedingJoinPoint::new(|| {
        _2 = const database::query_user;
        move _2(42_u64)
    }, _jp);

    // Call aspect's around method
    _1 = _aspect.around(_pjp) -> [return: bb1, unwind: bb2];
}
```

## JoinPoint Extension

### CallSiteJoinPoint

```rust
/// Information about a function call site.
pub struct CallSiteJoinPoint {
    /// Function being called
    pub target_function: &'static str,

    /// Module of target function
    pub target_module: &'static str,

    /// Function where call occurs
    pub caller_function: &'static str,

    /// Is this a method call?
    pub is_method: bool,

    /// Trait name (if trait method)
    pub trait_name: Option<&'static str>,

    /// Source location of call
    pub location: Location,

    /// Arguments (as Any)
    pub args: Vec<Box<dyn Any>>,
}
```

## Examples

### Example 1: Automatic Retry

```rust
struct RetryAspect {
    max_attempts: usize,
    backoff_ms: u64,
}

impl Aspect for RetryAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            match pjp.proceed() {
                Ok(result) => {
                    if attempt > 1 {
                        log::info!("Succeeded after {} attempts", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_attempts {
                        let delay = Duration::from_millis(self.backoff_ms * 2_u64.pow(attempt as u32 - 1));
                        log::warn!("Attempt {} failed, retrying in {:?}...", attempt, delay);
                        std::thread::sleep(delay);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }
}

#[advice(
    pointcut = "call(* external::api::*(..))",
    advice = "around"
)]
static RETRY: RetryAspect = RetryAspect {
    max_attempts: 3,
    backoff_ms: 100,
};

// All external API calls automatically retry:
fn fetch_data() {
    let data = external::api::get_user(42)?;  // Retries on failure
    let posts = external::api::get_posts()?;   // Retries on failure
}
```

### Example 2: Database Transaction Wrapping

```rust
#[advice(
    pointcut = "call(* database::*(..))",
    advice = "around"
)]
fn wrap_in_transaction(pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    let tx = database::begin_transaction()?;

    match pjp.proceed() {
        Ok(result) => {
            tx.commit()?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback()?;
            Err(e)
        }
    }
}

// All database calls automatically transactional:
fn update_user(id: u64, name: String) {
    database::update_field(id, "name", name)?;  // In transaction
    database::update_timestamp(id)?;            // In transaction
}
```

### Example 3: Call Logging

```rust
#[advice(
    pointcut = "call(* business::*(..))",
    advice = "before"
)]
fn log_business_calls(ctx: &CallSiteJoinPoint) {
    log::info!(
        "[CALL] {} called {} at {}:{}",
        ctx.caller_function,
        ctx.target_function,
        ctx.location.file,
        ctx.location.line
    );
}

// All business logic calls logged:
fn process() {
    business::validate_input(data);  // Logged
    business::process_payment(amt);  // Logged
    business::send_notification();   // Logged
}
```

## Performance Considerations

### Overhead Per Call

- Create CallSiteJoinPoint: ~5ns
- Create ProceedingJoinPoint: ~10ns
- Call aspect: ~5-10ns per aspect
- Total: ~20-30ns per intercepted call

### Optimization Strategies

1. **Selective Matching**
   ```rust
   // ✗ Too broad
   "call(*::*(..))"

   // ✓ Specific
   "call(* database::*(..))"
   "call(* api::fetch_*(..))"
   ```

2. **Inline Aspects**
   ```rust
   #[inline(always)]
   fn before(&self, ctx: &CallSiteJoinPoint) {
       // Fast path
   }
   ```

3. **Compile-Time Evaluation**
   - Evaluate pointcuts at compile time
   - Skip no-op aspects
   - Optimize closure creation

## Challenges

### 1. Generic Function Calls

**Problem:** Generic functions monomorphize late

```rust
fn process<T: Clone>(x: T) {
    let cloned = x.clone();  // What to match?
}
```

**Solution:** Match after monomorphization

```rust
// Matches all clone() calls
"call(* <* as Clone>::clone(..))"

// Matches specific type
"call(* <User as Clone>::clone(..))"
```

### 2. Trait Object Calls

**Problem:** Dynamic dispatch obscures target

```rust
fn process(x: &dyn Display) {
    x.fmt();  // Target unknown at compile time
}
```

**Solution:** Match trait method pattern

```rust
"call(* <* as Display>::fmt(..))"
```

### 3. Closure Calls

**Problem:** Closures are anonymous

```rust
let f = || println!("hello");
f();  // How to match?
```

**Solution:** Special closure call pattern

```rust
"call(closure *(..))"
```

## Integration

### With Existing Components

**extract.rs:** Add call site detection
**match.rs:** Add `call()` pattern support
**generate.rs:** Add call wrapping code generation

### With execution() Pointcuts

Can combine:

```rust
// Apply to function definition AND all calls to it
"execution(pub fn dangerous(..)) || call(* dangerous(..))"

// Method definition and all method calls
"execution(* User::modify(..)) || call(* User::modify(..))"
```

## Implementation Roadmap

### Phase 1: Detection (Days 1-2)
- [ ] Parse call() pointcuts
- [ ] Detect call sites in MIR
- [ ] Extract call metadata
- [ ] Match against pointcuts

### Phase 2: Code Generation (Days 3-4)
- [ ] Generate CallSiteJoinPoint
- [ ] Wrap calls in aspects
- [ ] Handle return values
- [ ] Preserve error handling

### Phase 3: Testing (Day 5)
- [ ] Unit tests for detection
- [ ] Integration tests
- [ ] Method call tests
- [ ] Trait method tests

## Status

**Week 6 Deliverable:** Design Complete ✅

**Next Steps:**
1. Implement MIR call detection
2. Generate call site wrappers
3. Test with real examples
4. Optimize performance

**Requires:** Nightly Rust + rustc integration

## Comparison Table

| Feature | execution() | call() |
|---------|------------|--------|
| Matches | Function definitions | Function calls |
| Application | Once per function | Each call site |
| Use case | Modify function behavior | Wrap specific calls |
| Granularity | Coarse | Fine |
| Overhead | Lower | Higher (per call) |
| Example | Add logging to a function | Retry all API calls |

## Conclusion

Call-site matching provides fine-grained control over function invocations, enabling powerful patterns like automatic retry, transaction wrapping, and per-call instrumentation. Combined with execution() pointcuts, it offers complete coverage of both function definitions and their usage sites.
