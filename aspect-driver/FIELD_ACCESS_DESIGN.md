# Field Access Interception Design

**Phase 3 Week 5**

## Overview

Field access interception enables aspects to run when struct fields are read or written, allowing fine-grained control over data access patterns.

## Use Cases

### Security Auditing
```rust
#[advice(pointcut = "field_access_mut(User::password)", advice = "before")]
fn audit_password_change(ctx: &JoinPoint) {
    log::warn!("Password modified by {}", ctx.function_name);
}

// Automatically triggers aspect:
user.password = new_password;
```

### Access Control
```rust
#[advice(pointcut = "field_access(Account::balance)", advice = "before")]
fn check_permission(ctx: &JoinPoint) {
    if !has_permission() {
        panic!("Unauthorized access to balance");
    }
}

// Protected access:
let balance = account.balance; // Checks permission
```

### Change Tracking
```rust
#[advice(pointcut = "field_access_mut(*::*)", advice = "after")]
fn track_changes(ctx: &JoinPoint, old_value: &dyn Any, new_value: &dyn Any) {
    changelog::record(ctx.struct_name, ctx.field_name, old_value, new_value);
}

// All field writes tracked:
user.email = "new@example.com"; // Tracked
```

## Pointcut Syntax

### Basic Patterns

```rust
// Match specific field
"field_access(User::password)"
"field_access_mut(User::password)"

// Match all fields in type
"field_access(User::*)"

// Match field across all types
"field_access(*::email)"

// Match all field accesses
"field_access(*::*)"
```

### Composed Patterns

```rust
// Mutable access to sensitive fields
"field_access_mut(User::password) || field_access_mut(User::secret_key)"

// Read access in specific module
"field_access(*::*) && within(crate::api)"

// Exclude internal access
"field_access_mut(*::*) && !within(crate::internal)"
```

## MIR Representation

### Field Read

```rust
// Source code:
let value = user.name;

// MIR (simplified):
_2 = (_1.0: String);  // Read field 0 (name) from _1 (user)
```

### Field Write

```rust
// Source code:
user.name = new_name;

// MIR (simplified):
(_1.0: String) = move _2;  // Write _2 to field 0 (name) of _1 (user)
```

## Detection Strategy

### Step 1: Identify Field Access in MIR

```ignore
use rustc_middle::mir::{Body, Place, PlaceElem, Statement, StatementKind};

fn find_field_accesses<'tcx>(body: &Body<'tcx>) -> Vec<FieldAccess> {
    let mut accesses = Vec::new();

    for (block_idx, block) in body.basic_blocks.iter_enumerated() {
        for statement in &block.statements {
            if let StatementKind::Assign(box (place, rvalue)) = &statement.kind {
                // Check if place contains field projection
                for (idx, elem) in place.projection.iter().enumerate() {
                    if let PlaceElem::Field(field, ty) = elem {
                        accesses.push(FieldAccess {
                            struct_type: place.local_decl().ty,
                            field_index: field.index(),
                            is_write: true,
                            location: statement.source_info.span,
                        });
                    }
                }

                // Check rvalue for field reads
                if let Rvalue::Use(Operand::Copy(place) | Operand::Move(place)) = rvalue {
                    for elem in place.projection {
                        if let PlaceElem::Field(field, ty) = elem {
                            accesses.push(FieldAccess {
                                struct_type: place.local_decl().ty,
                                field_index: field.index(),
                                is_write: false,
                                location: statement.source_info.span,
                            });
                        }
                    }
                }
            }
        }
    }

    accesses
}
```

### Step 2: Extract Field Metadata

```ignore
use rustc_middle::ty::{TyCtxt, AdtDef};

struct FieldMetadata {
    struct_name: String,
    field_name: String,
    field_type: String,
    is_public: bool,
    is_mutable: bool,
}

fn extract_field_metadata<'tcx>(
    tcx: TyCtxt<'tcx>,
    adt_def: &AdtDef,
    field_idx: usize,
) -> FieldMetadata {
    let field = &adt_def.all_fields().nth(field_idx).unwrap();

    FieldMetadata {
        struct_name: tcx.def_path_str(adt_def.did()),
        field_name: field.name.to_string(),
        field_type: field.ty(tcx, substs).to_string(),
        is_public: field.vis.is_public(),
        is_mutable: field.mutability.is_mut(),
    }
}
```

### Step 3: Match Against Pointcuts

```rust
fn matches_field_pointcut(
    field: &FieldMetadata,
    pointcut: &str,
    is_write: bool,
) -> bool {
    // Parse pointcut: "field_access_mut(User::password)"
    let (access_type, pattern) = parse_field_pointcut(pointcut)?;

    // Check access type
    match access_type {
        FieldAccessType::Read if is_write => return false,
        FieldAccessType::Write if !is_write => return false,
        FieldAccessType::Any => {},
    }

    // Check pattern
    let (struct_pattern, field_pattern) = parse_pattern(pattern)?;

    matches_struct(&field.struct_name, struct_pattern)
        && matches_field(&field.field_name, field_pattern)
}
```

## Code Generation

### Strategy 1: Getter/Setter Injection

Transform direct field access into method calls:

```rust
// Before:
user.password = new_password;

// After:
user.__set_password(new_password);

// Generated setter:
impl User {
    #[inline]
    fn __set_password(&mut self, value: String) {
        let ctx = FieldAccessJoinPoint {
            struct_name: "User",
            field_name: "password",
            access_type: FieldAccessType::Write,
        };

        PasswordAuditAspect::new().before(&ctx);
        self.password = value;
    }
}
```

### Strategy 2: Proxy Wrapper

Wrap the struct in a proxy that intercepts field access:

```rust
// Original:
pub struct User {
    pub name: String,
    pub password: String,
}

// Generated proxy:
pub struct User__Proxy {
    inner: User,
}

impl User__Proxy {
    pub fn name(&self) -> &String {
        let ctx = FieldAccessJoinPoint { ... };
        AccessAspect::new().before(&ctx);
        &self.inner.name
    }

    pub fn set_name(&mut self, value: String) {
        let ctx = FieldAccessJoinPoint { ... };
        AccessAspect::new().before(&ctx);
        self.inner.name = value;
    }

    pub fn password(&self) -> &String {
        let ctx = FieldAccessJoinPoint { ... };
        PasswordAspect::new().before(&ctx);
        &self.inner.password
    }

    pub fn set_password(&mut self, value: String) {
        let ctx = FieldAccessJoinPoint { ... };
        PasswordAspect::new().before(&ctx);
        self.inner.password = value;
    }
}
```

### Strategy 3: MIR Rewriting (Preferred)

Directly modify MIR to insert aspect calls:

```ignore
// Original MIR:
_3 = (_1.0: String);  // Read user.password

// Transformed MIR:
_tmp1 = const FieldAccessJoinPoint { ... };
_tmp2 = PasswordAspect::new();
call _tmp2.before(_tmp1);
_3 = (_1.0: String);  // Original access
```

## JoinPoint Extension

### FieldAccessJoinPoint

```rust
/// Information about a field access.
pub struct FieldAccessJoinPoint {
    /// Struct type name
    pub struct_name: &'static str,

    /// Field name
    pub field_name: &'static str,

    /// Access type
    pub access_type: FieldAccessType,

    /// Function where access occurs
    pub function_name: &'static str,

    /// Source location
    pub location: Location,

    /// Old value (for writes)
    pub old_value: Option<Box<dyn Any>>,
}

pub enum FieldAccessType {
    Read,
    Write,
}
```

### Aspect Trait Extension

```rust
pub trait Aspect {
    // Existing methods...

    /// Called before field access.
    fn before_field_access(&self, _ctx: &FieldAccessJoinPoint) {}

    /// Called after field access.
    fn after_field_access(&self, _ctx: &FieldAccessJoinPoint, _value: &dyn Any) {}

    /// Wrap field access.
    fn around_field_access(
        &self,
        pjp: FieldAccessProceedingJoinPoint,
    ) -> Result<Box<dyn Any>, AspectError> {
        pjp.proceed()
    }
}
```

## Performance Considerations

### Overhead

**Per-field-access overhead:**
- Create FieldAccessJoinPoint: ~5ns
- Call aspect methods: ~2-5ns per aspect
- Total: ~10-20ns per access

**Mitigation:**
- Inline aspect calls
- Compile-time evaluation when possible
- Cache JoinPoint data
- Skip no-op aspects

### Selective Application

Only apply to matched fields:

```rust
// Don't intercept ALL field access
"field_access(*::*)"  // ✗ Heavy overhead

// Be specific
"field_access_mut(User::password)"  // ✓ Minimal overhead
"field_access(Account::balance)"     // ✓ Only when needed
```

## Examples

### Example 1: Password Security Audit

```rust
#[derive(Debug)]
struct User {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[advice(
    pointcut = "field_access_mut(User::password)",
    advice = "before"
)]
fn audit_password_change(ctx: &FieldAccessJoinPoint) {
    log::warn!(
        "PASSWORD CHANGE: User password modified in {} at {}:{}",
        ctx.function_name,
        ctx.location.file,
        ctx.location.line
    );

    // Additional security checks
    if !is_secure_context() {
        panic!("Insecure password modification attempt!");
    }
}

fn main() {
    let mut user = User {
        username: "alice".to_string(),
        password: "secret123".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Triggers audit
    user.password = "new_secret456".to_string();
    // Log: "PASSWORD CHANGE: User password modified in main at ..."

    // No audit (different field)
    user.email = "alice@newdomain.com".to_string();
}
```

### Example 2: Balance Validation

```rust
struct Account {
    pub id: u64,
    pub balance: f64,
}

#[advice(
    pointcut = "field_access_mut(Account::balance)",
    advice = "before"
)]
fn validate_balance(ctx: &FieldAccessJoinPoint) {
    if let Some(old_value) = &ctx.old_value {
        if let Some(old_balance) = old_value.downcast_ref::<f64>() {
            log::info!("Balance changing from ${}", old_balance);
        }
    }
}

#[advice(
    pointcut = "field_access_mut(Account::balance)",
    advice = "after"
)]
fn check_balance_integrity(ctx: &FieldAccessJoinPoint, new_value: &dyn Any) {
    if let Some(balance) = new_value.downcast_ref::<f64>() {
        if *balance < 0.0 {
            panic!("Negative balance not allowed!");
        }
        log::info!("Balance updated to ${}", balance);
    }
}
```

### Example 3: Change Tracking

```rust
struct Document {
    pub title: String,
    pub content: String,
    pub last_modified: DateTime,
}

#[advice(
    pointcut = "field_access_mut(Document::*)",
    advice = "after"
)]
fn track_document_changes(ctx: &FieldAccessJoinPoint, new_value: &dyn Any) {
    changelog::record(ChangeEvent {
        struct_name: ctx.struct_name,
        field_name: ctx.field_name,
        timestamp: Utc::now(),
        modified_by: current_user(),
    });

    log::info!("Document field '{}' modified", ctx.field_name);
}
```

## Implementation Roadmap

### Phase 1: Detection (Days 1-2)
- [ ] Parse field_access() pointcuts
- [ ] Detect field access in MIR
- [ ] Extract field metadata
- [ ] Match against pointcuts

### Phase 2: Code Generation (Days 3-4)
- [ ] Generate FieldAccessJoinPoint
- [ ] Insert aspect calls before/after access
- [ ] Handle read vs write
- [ ] Preserve semantics

### Phase 3: Testing (Day 5)
- [ ] Unit tests for detection
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Edge cases (generics, traits)

## Challenges

### 1. Borrow Checker

**Problem:** Aspects need immutable reference while field is being mutably accessed

**Solution:**
```rust
// Store old value before modification
let old_value = user.password.clone();

// Allow aspect to inspect it
aspect.before_field_access(&ctx_with_old_value);

// Then perform modification
user.password = new_value;
```

### 2. Generic Fields

**Problem:** Field type depends on generic parameters

**Solution:** Monomorphization - apply aspects after generic resolution

### 3. Private Fields

**Problem:** Can't generate public accessors for private fields

**Solution:** Keep accessors private, maintain encapsulation

## Integration

### With Existing Components

- **extract.rs**: Add field access detection
- **match.rs**: Add field_access() pattern support
- **generate.rs**: Add field access code generation

### With Phase 2

Compatible with proc macros:

```rust
// Manual annotation (Phase 2)
#[aspect(FieldAudit)]
struct User { ... }

// Automatic (Phase 3)
// Applied via pointcut matching
struct User { ... }
```

## Status

**Week 5 Deliverable:** Design Complete ✅

**Next Steps:**
1. Implement MIR field detection
2. Generate FieldAccessJoinPoint code
3. Test with real examples
4. Optimize performance

**Requires:** Nightly Rust + rustc integration
