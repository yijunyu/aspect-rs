# Crosscutting Concerns Explained

## Definition

A **crosscutting concern** is functionality that affects multiple parts of an application but doesn't naturally fit into a single module or component.

### Examples of Crosscutting Concerns

| Concern | Where it applies | Why it's crosscutting |
|---------|------------------|----------------------|
| **Logging** | Every function | Needed across all modules |
| **Performance monitoring** | Critical paths | Scattered across components |
| **Caching** | Expensive operations | Applied inconsistently |
| **Authorization** | Public APIs | Duplicated in every endpoint |
| **Transactions** | Database operations | Repeated in every DAO |
| **Retry logic** | Network calls | Spread across HTTP, database, etc. |
| **Metrics collection** | Key operations | Manually added everywhere |
| **Validation** | Input handling | Copy-pasted validation code |

## The Core Problem

### Horizontal vs Vertical Concerns

Traditional modularity handles **vertical concerns** well:

```rust
mod user {
    pub fn create_user(...) { }
    pub fn delete_user(...) { }
}

mod order {
    pub fn create_order(...) { }
    pub fn cancel_order(...) { }
}

mod payment {
    pub fn process_payment(...) { }
    pub fn refund_payment(...) { }
}
```

Each module focuses on **one domain** (users, orders, payments).

But **horizontal concerns** cut across all modules:

```
                 Logging
                    ↓
┌─────────────────────────────────┐
│  user::create_user()            │ ← Needs logging
│  user::delete_user()            │ ← Needs logging
├─────────────────────────────────┤
│  order::create_order()          │ ← Needs logging
│  order::cancel_order()          │ ← Needs logging
├─────────────────────────────────┤
│  payment::process_payment()     │ ← Needs logging
│  payment::refund_payment()      │ ← Needs logging
└─────────────────────────────────┘
```

**Logging, metrics, and caching** are needed in **all modules**, breaking encapsulation.

## Code Scattering

Without AOP, crosscutting code is **scattered** across your codebase:

```rust
// user.rs
fn create_user(name: String) -> Result<User, Error> {
    log::info!("Creating user: {}", name);
    let start = Instant::now();

    let user = database::insert_user(name)?;

    log::info!("User created in {:?}", start.elapsed());
    metrics::record("user_created", start.elapsed());
    Ok(user)
}

// order.rs
fn create_order(items: Vec<Item>) -> Result<Order, Error> {
    log::info!("Creating order with {} items", items.len());
    let start = Instant::now();

    let order = database::insert_order(items)?;

    log::info!("Order created in {:?}", start.elapsed());
    metrics::record("order_created", start.elapsed());
    Ok(order)
}

// payment.rs
fn process_payment(amount: u64) -> Result<Receipt, Error> {
    log::info!("Processing payment: ${}", amount);
    let start = Instant::now();

    let receipt = payment_gateway::charge(amount)?;

    log::info!("Payment processed in {:?}", start.elapsed());
    metrics::record("payment_processed", start.elapsed());
    Ok(receipt)
}
```

**Problem**: The same logging/timing/metrics pattern is **copy-pasted** three times!

## Code Tangling

Crosscutting code **tangles** with business logic:

```rust
fn transfer_funds(from: Account, to: Account, amount: u64) -> Result<(), Error> {
    // Logging (line 1-2)
    log::info!("Transferring ${} from {} to {}", amount, from.id, to.id);
    let start = Instant::now();

    // Authorization (line 4-7)
    if !has_permission("transfer", from.user_id) {
        log::error!("Unauthorized transfer attempt");
        return Err(Error::Unauthorized);
    }

    // Validation (line 9-12)
    if amount == 0 || amount > from.balance {
        log::error!("Invalid transfer amount");
        return Err(Error::InvalidAmount);
    }

    // Business logic (finally! line 14-17)
    from.balance -= amount;
    to.balance += amount;
    database::save_account(from)?;
    database::save_account(to)?;

    // Metrics (line 19-21)
    log::info!("Transfer completed in {:?}", start.elapsed());
    metrics::record("transfer", start.elapsed());

    Ok(())
}
```

**Business logic** (lines 14-17) is **buried** in 20+ lines of crosscutting code!

## Maintenance Nightmare

### Scenario: Update Log Format

Boss: "Add correlation IDs to all logs for distributed tracing."

**Without AOP**:
- Find all `log::info!` calls (100+ locations)
- Manually update each one
- Hope you didn't miss any
- Test everything again

```rust
// Before
log::info!("Creating user: {}", name);

// After
log::info!("[correlation_id={}] Creating user: {}", get_correlation_id(), name);
```

**With aspect-rs**:
- Update `LoggingAspect::before()` (1 location)
- Recompile
- Done!

```rust
impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        log::info!("[{}] → {}", get_correlation_id(), ctx.function_name);
    }
}
```

## Testing Difficulty

Crosscutting code makes unit testing harder:

```rust
#[test]
fn test_transfer_funds() {
    // Must mock logging
    let _log_guard = setup_test_logging();

    // Must mock metrics
    let _metrics_guard = setup_test_metrics();

    // Must mock authorization
    let _auth_guard = setup_test_auth();

    // Finally test business logic
    let result = transfer_funds(...);
    assert!(result.is_ok());
}
```

**With aspect-rs**:

```rust
#[test]
fn test_transfer_funds() {
    // Test pure business logic, no mocking needed
    let result = transfer_funds_impl(...);
    assert!(result.is_ok());
}
```

## The AOP Solution

AOP lets you **modularize** crosscutting concerns:

```rust
// Define once
struct LoggingAspect;
impl Aspect for LoggingAspect {
    fn before(&self, ctx: &JoinPoint) {
        log::info!("[{}] → {}", get_correlation_id(), ctx.function_name);
    }
}

// Apply everywhere
#[aspect(LoggingAspect::new())]
fn create_user(name: String) -> Result<User, Error> { ... }

#[aspect(LoggingAspect::new())]
fn create_order(items: Vec<Item>) -> Result<Order, Error> { ... }

#[aspect(LoggingAspect::new())]
fn process_payment(amount: u64) -> Result<Receipt, Error> { ... }
```

**Benefits**:
- ✅ **No scattering**: Logging logic in one place
- ✅ **No tangling**: Business logic stands alone
- ✅ **Easy maintenance**: Change logging in `LoggingAspect`
- ✅ **Better testing**: Test business logic without mocks

Next, let's learn the [AOP Terminology](terminology.md) used throughout aspect-rs.
