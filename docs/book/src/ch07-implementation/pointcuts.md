# Pointcut Matching

Pointcuts are pattern expressions that select which functions should have aspects applied. This chapter explains how aspect-rs matches functions against pointcut patterns.

## Overview

A **pointcut** is a predicate that matches join points (function calls). In aspect-rs, pointcuts enable:

- **Declarative aspect application** - Specify patterns instead of annotating individual functions
- **Centralized policy** - Define cross-cutting concerns in one place
- **Automatic weaving** - New functions automatically get aspects applied

## Pointcut Expression Language

### Basic Syntax

Pointcut expressions follow AspectJ-inspired syntax:

```
execution(<visibility> fn <name>(<parameters>)) <operator> <additional-patterns>
```

### Execution Pointcuts

Match function execution:

```rust
// Match all public functions
execution(pub fn *(..))

// Match specific function name
execution(pub fn fetch_user(..))

// Match pattern in name
execution(pub fn *_user(..))

// Match functions with specific signature
execution(pub fn process(u64) -> Result<*, *>)
```

**Components**:
- `pub` - Visibility (pub, pub(crate), or omit for private)
- `fn` - Function keyword
- `*` - Wildcard for names
- `(..)` - Any parameters
- `->` - Return type (optional)

### Within Pointcuts

Match functions within a module:

```rust
// All functions in api module
within(crate::api)

// All functions in api and submodules
within(crate::api::*)

// Specific module path
within(my_crate::handlers::user)
```

### Combined Pointcuts

Use boolean operators to combine patterns:

```rust
// AND - both conditions must match
execution(pub fn *(..)) && within(crate::api)

// OR - either condition matches
execution(pub fn fetch_*(..)) || execution(pub fn get_*(..))

// NOT - inverse of condition
execution(pub fn *(..)) && !within(crate::internal)
```

## Implementation Architecture

### Pointcut Parser

The `PointcutMatcher` parses and evaluates pointcut expressions:

```rust
pub struct PointcutMatcher {
    pattern: String,
    ast: PointcutAst,
}

impl PointcutMatcher {
    pub fn new(pattern: &str) -> Result<Self, ParseError> {
        let ast = parse_pointcut(pattern)?;
        Ok(Self {
            pattern: pattern.to_string(),
            ast,
        })
    }

    pub fn matches(&self, func_info: &FunctionInfo) -> bool {
        evaluate_pointcut(&self.ast, func_info)
    }
}
```

### Function Information

Functions are represented as metadata structures:

```rust
pub struct FunctionInfo {
    pub name: String,
    pub qualified_name: String,
    pub module_path: String,
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_generic: bool,
    pub return_type: Option<String>,
    pub parameters: Vec<Parameter>,
}

pub enum Visibility {
    Public,
    Crate,
    Private,
}

pub struct Parameter {
    pub name: String,
    pub ty: String,
}
```

### Matching Algorithm

```
1. Parse pointcut expression into AST
2. Extract function metadata
3. Evaluate AST against function info
4. Return boolean match result
```

## Matching Strategies

### Execution Matching

**Pattern**: `execution(pub fn fetch_user(..))`

**Algorithm**:
```rust
fn match_execution(pattern: &ExecutionPattern, func: &FunctionInfo) -> bool {
    // Check visibility
    if let Some(vis) = &pattern.visibility {
        if !matches_visibility(vis, &func.visibility) {
            return false;
        }
    }

    // Check function name
    if !matches_name(&pattern.name, &func.name) {
        return false;
    }

    // Check parameters
    if let Some(params) = &pattern.parameters {
        if !matches_parameters(params, &func.parameters) {
            return false;
        }
    }

    // Check return type
    if let Some(ret) = &pattern.return_type {
        if !matches_return_type(ret, &func.return_type) {
            return false;
        }
    }

    true
}
```

### Name Pattern Matching

Wildcards and patterns:

```rust
fn matches_name(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;  // Match any name
    }

    if pattern.contains('*') {
        // Wildcard pattern matching
        let regex = pattern.replace('*', ".*");
        Regex::new(&regex).unwrap().is_match(name)
    } else {
        // Exact match
        pattern == name
    }
}
```

**Examples**:
- `*` matches: `fetch_user`, `save_user`, `anything`
- `fetch_*` matches: `fetch_user`, `fetch_data`, but not `get_user`
- `*_user` matches: `fetch_user`, `save_user`, but not `user_info`

### Module Path Matching

```rust
fn matches_within(pattern: &str, module_path: &str) -> bool {
    if pattern.ends_with("::*") {
        // Match module and submodules
        let prefix = pattern.trim_end_matches("::*");
        module_path.starts_with(prefix)
    } else {
        // Exact module match
        module_path == pattern
    }
}
```

**Examples**:
- `crate::api` matches: `crate::api` only
- `crate::api::*` matches: `crate::api`, `crate::api::users`, `crate::api::orders`

### Boolean Operator Evaluation

```rust
enum PointcutAst {
    Execution(ExecutionPattern),
    Within(String),
    And(Box<PointcutAst>, Box<PointcutAst>),
    Or(Box<PointcutAst>, Box<PointcutAst>),
    Not(Box<PointcutAst>),
}

fn evaluate_pointcut(ast: &PointcutAst, func: &FunctionInfo) -> bool {
    match ast {
        PointcutAst::Execution(pattern) => {
            match_execution(pattern, func)
        }
        PointcutAst::Within(module) => {
            matches_within(module, &func.module_path)
        }
        PointcutAst::And(left, right) => {
            evaluate_pointcut(left, func) && evaluate_pointcut(right, func)
        }
        PointcutAst::Or(left, right) => {
            evaluate_pointcut(left, func) || evaluate_pointcut(right, func)
        }
        PointcutAst::Not(inner) => {
            !evaluate_pointcut(inner, func)
        }
    }
}
```

## Pattern Examples

### Common Patterns

**All public functions**:
```rust
execution(pub fn *(..))
```

**All API endpoints**:
```rust
execution(pub fn *(..)) && within(crate::api::handlers)
```

**All functions returning Result**:
```rust
execution(fn *(..) -> Result<*, *>)
```

**All async functions**:
```rust
execution(async fn *(..))
```

**Database operations**:
```rust
execution(fn *(..) -> *) && within(crate::db)
```

**User-related functions**:
```rust
execution(pub fn *_user(..)) ||
execution(pub fn user_*(..))
```

### Complex Patterns

**Public API except internal**:
```rust
execution(pub fn *(..)) &&
within(crate::api) &&
!within(crate::api::internal)
```

**Critical functions needing audit**:
```rust
(execution(pub fn delete_*(..)) ||
 execution(pub fn remove_*(..))) &&
within(crate::api)
```

**All Result-returning functions except tests**:
```rust
execution(fn *(..) -> Result<*, *>) &&
!within(crate::tests)
```

## Performance Considerations

### Compile-Time Matching

Pointcut matching happens at compile time with negligible overhead.

### Optimization Strategies

**Cache pattern compilation**:
```rust
lazy_static! {
    static ref COMPILED_PATTERNS: Mutex<HashMap<String, CompiledPattern>> =
        Mutex::new(HashMap::new());
}
```

**Pre-compute matches**:
```rust
// During macro expansion
let matches = registry.find_matching(&func_info);
// Generate code only for matches
```

## Testing Pointcuts

### Unit Tests

```rust
#[test]
fn test_execution_matching() {
    let pattern = PointcutMatcher::new("execution(pub fn fetch_user(..))").unwrap();

    let func = FunctionInfo {
        name: "fetch_user".to_string(),
        visibility: Visibility::Public,
        ..Default::default()
    };

    assert!(pattern.matches(&func));
}
```

## Best Practices

### Writing Effective Pointcuts

**DO**:
- ✅ Be specific to avoid over-matching
- ✅ Use `within` to scope to modules
- ✅ Test pointcuts with sample functions
- ✅ Document complex pointcut expressions

**DON'T**:
- ❌ Use `execution(*)` (too broad)
- ❌ Create overly complex boolean expressions
- ❌ Match too many functions
- ❌ Forget to exclude test code

## Summary

Pointcut matching in aspect-rs:

1. **AspectJ-inspired syntax** - Familiar for AOP developers
2. **Compile-time evaluation** - Zero runtime overhead
3. **Boolean combinators** - Flexible pattern composition
4. **Module scoping** - Precise control over application

**Key advantage**: Declare once, apply everywhere - true separation of concerns.

## See Also

- [Macro Code Generation](macros.md) - How macros use pointcuts
- [MIR Extraction](mir.md) - Function metadata extraction
- [Code Weaving](weaving.md) - Applying matched aspects
- [Phase 3 Automatic Weaving](../ch10-phase3/pointcuts.md) - Advanced pointcut features
