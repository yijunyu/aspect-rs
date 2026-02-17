# Production Patterns

This chapter covers battle-tested patterns for using aspects in production systems. We'll explore real-world use cases including caching, rate limiting, circuit breakers, and transaction management.

## Caching

Caching is one of the most common performance optimizations in production systems. aspect-rs makes it trivial to add caching to expensive operations without modifying business logic.

### Basic Caching

The simplest approach uses the `CachingAspect` from `aspect-std`:

```rust
use aspect_std::CachingAspect;
use aspect_macros::aspect;

#[aspect(CachingAspect::new())]
fn fetch_user(id: u64) -> User {
    // Expensive database query
    database::query_user(id)
}

#[aspect(CachingAspect::new())]
fn expensive_calculation(n: u64) -> u64 {
    // CPU-intensive computation
    (0..n).map(|i| i * i).sum()
}
```

**Key Benefits:**
- First call executes the function and caches the result
- Subsequent calls return cached value instantly
- No changes to business logic required
- Cache is transparent to callers

### Cache with TTL

For data that changes over time, use time-to-live (TTL):

```rust
use aspect_std::CachingAspect;
use std::time::Duration;

#[aspect(CachingAspect::with_ttl(Duration::from_secs(300)))]
fn fetch_exchange_rate(currency: &str) -> f64 {
    // External API call - cache for 5 minutes
    api::get_exchange_rate(currency)
}
```

### Conditional Caching

Sometimes you only want to cache successful results:

```rust
use aspect_std::CachingAspect;

#[aspect(CachingAspect::cache_on_success())]
fn fetch_data(url: &str) -> Result<String, Error> {
    // Only cache successful responses
    // Errors are not cached and will retry
    reqwest::blocking::get(url)?.text()
}
```

### Real-World Example: User Profile Service

```rust
use aspect_std::{CachingAspect, LoggingAspect, TimingAspect};
use std::time::Duration;

// Stack multiple aspects for production use
#[aspect(CachingAspect::with_ttl(Duration::from_secs(60)))]
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn get_user_profile(user_id: u64) -> Result<UserProfile, Error> {
    // 1. Check cache (CachingAspect)
    // 2. Log entry (LoggingAspect)
    // 3. Start timer (TimingAspect)
    // 4. Execute query if cache miss
    // 5. Measure time
    // 6. Log exit
    // 7. Cache result

    database::fetch_profile(user_id)
}
```

**Performance Impact:**
- Cache hit: <1µs (memory lookup)
- Cache miss: ~10ms (database query)
- 99% hit rate = 1000x faster average response

## Rate Limiting

Rate limiting prevents resource exhaustion and protects against abuse. aspect-rs provides flexible rate limiting without modifying endpoint code.

### Basic Rate Limiting

Limit calls per time window:

```rust
use aspect_std::RateLimitAspect;
use std::time::Duration;

// 100 calls per minute per client
#[aspect(RateLimitAspect::new(100, Duration::from_secs(60)))]
fn api_endpoint(request: Request) -> Response {
    handle_request(request)
}
```

### Per-User Rate Limiting

More sophisticated rate limiting based on user identity:

```rust
use aspect_std::RateLimitAspect;
use std::time::Duration;

fn get_user_id() -> String {
    // Extract from request context
    current_request::user_id()
}

#[aspect(RateLimitAspect::per_user(100, Duration::from_secs(60), get_user_id))]
fn protected_endpoint(data: RequestData) -> Result<Response, Error> {
    // Rate limit enforced per user
    process_request(data)
}
```

### Tiered Rate Limiting

Different limits for different user tiers:

```rust
use aspect_std::RateLimitAspect;

fn get_rate_limit() -> (usize, Duration) {
    match current_user::subscription_tier() {
        Tier::Free => (10, Duration::from_secs(60)),    // 10/min
        Tier::Pro => (100, Duration::from_secs(60)),     // 100/min
        Tier::Enterprise => (1000, Duration::from_secs(60)), // 1000/min
    }
}

#[aspect(RateLimitAspect::dynamic(get_rate_limit))]
fn api_call(params: ApiParams) -> Result<ApiResponse, Error> {
    execute_api_call(params)
}
```

### Real-World Example: API Server

```rust
use aspect_std::{RateLimitAspect, AuthorizationAspect, LoggingAspect};
use std::time::Duration;

// GET /api/users/:id
#[aspect(RateLimitAspect::new(1000, Duration::from_secs(60)))]
#[aspect(LoggingAspect::new())]
fn get_user(id: u64) -> Result<User, Error> {
    database::get_user(id)
}

// POST /api/users (more restrictive)
#[aspect(RateLimitAspect::new(10, Duration::from_secs(60)))]
#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
#[aspect(LoggingAspect::new())]
fn create_user(user: NewUser) -> Result<User, Error> {
    database::create_user(user)
}

// DELETE /api/users/:id (most restrictive)
#[aspect(RateLimitAspect::new(5, Duration::from_secs(60)))]
#[aspect(AuthorizationAspect::require_role("admin", get_roles))]
#[aspect(LoggingAspect::new())]
fn delete_user(id: u64) -> Result<(), Error> {
    database::delete_user(id)
}
```

**Behavior:**
- Exceeded limits return `RateLimitExceeded` error
- No execution of underlying function
- Fast rejection (<100ns overhead)
- Per-function independent limits

## Circuit Breakers

Circuit breakers protect against cascading failures when calling external services. When failures exceed a threshold, the circuit "opens" and fails fast instead of waiting for timeouts.

### Basic Circuit Breaker

```rust
use aspect_std::CircuitBreakerAspect;
use std::time::Duration;

// Opens after 5 failures, retries after 30 seconds
#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
fn call_external_service(url: &str) -> Result<Response, Error> {
    reqwest::blocking::get(url)?.json()
}
```

**States:**
1. **Closed** (normal): All calls go through
2. **Open** (failing): Immediately fail without calling service
3. **Half-Open** (testing): Allow one test call to check recovery

### Circuit Breaker with Monitoring

```rust
use aspect_std::{CircuitBreakerAspect, MetricsAspect, LoggingAspect};
use std::time::Duration;

#[aspect(CircuitBreakerAspect::new(3, Duration::from_secs(30)))]
#[aspect(MetricsAspect::new())]
#[aspect(LoggingAspect::new())]
fn payment_gateway_call(amount: f64) -> Result<TransactionId, Error> {
    // If payment gateway is down:
    // - First 3 failures recorded
    // - Circuit opens
    // - Future calls fail instantly (no timeout waits)
    // - After 30s, circuit half-opens for test
    // - Success closes circuit

    payment_api::process_payment(amount)
}
```

### Multiple External Services

Use separate circuit breakers for independent services:

```rust
use aspect_std::CircuitBreakerAspect;
use std::time::Duration;

// Payment service
#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(60)))]
fn payment_service(tx: Transaction) -> Result<Receipt, Error> {
    payment_api::process(tx)
}

// Email service (separate circuit)
#[aspect(CircuitBreakerAspect::new(10, Duration::from_secs(30)))]
fn email_service(recipient: &str, message: &str) -> Result<(), Error> {
    email_api::send(recipient, message)
}

// Inventory service (separate circuit)
#[aspect(CircuitBreakerAspect::new(3, Duration::from_secs(90)))]
fn inventory_service(product_id: u64) -> Result<Stock, Error> {
    inventory_api::check_stock(product_id)
}
```

**Benefits:**
- Failures in one service don't affect others
- Independent recovery times
- Different thresholds based on reliability

### Real-World Example: Microservices

```rust
use aspect_std::{CircuitBreakerAspect, RetryAspect, TimingAspect};
use std::time::Duration;

// Critical service: aggressive circuit breaker
#[aspect(CircuitBreakerAspect::new(2, Duration::from_secs(120)))]
#[aspect(RetryAspect::new(3, Duration::from_millis(100)))]
#[aspect(TimingAspect::new())]
fn user_auth_service(credentials: Credentials) -> Result<Session, Error> {
    auth_api::authenticate(credentials)
}

// Non-critical service: lenient circuit breaker
#[aspect(CircuitBreakerAspect::new(10, Duration::from_secs(30)))]
#[aspect(TimingAspect::new())]
fn recommendation_service(user_id: u64) -> Result<Vec<Product>, Error> {
    // Can tolerate more failures
    // Shorter recovery time
    recommendations_api::get(user_id)
}
```

## Transactions

Database transactions ensure ACID properties. aspect-rs can automatically wrap operations in transactions without polluting business logic.

### Basic Transaction Management

```rust
use aspect_std::TransactionalAspect;

#[aspect(TransactionalAspect::new())]
fn transfer_money(from: u64, to: u64, amount: f64) -> Result<(), Error> {
    // Automatically wrapped in transaction:
    // BEGIN TRANSACTION
    database::debit_account(from, amount)?;
    database::credit_account(to, amount)?;
    // COMMIT (on success) or ROLLBACK (on error)

    Ok(())
}
```

**Behavior:**
- `TransactionalAspect` starts transaction before function
- Success: automatic COMMIT
- Error: automatic ROLLBACK
- Exception: automatic ROLLBACK

### Nested Transactions

Handle complex workflows with nested operations:

```rust
use aspect_std::TransactionalAspect;

#[aspect(TransactionalAspect::new())]
fn create_order(order: Order) -> Result<OrderId, Error> {
    // Outer transaction
    let order_id = database::insert_order(order)?;

    // These also have TransactionalAspect
    // In supporting databases, uses nested transactions or savepoints
    allocate_inventory(order.items)?;
    process_payment(order.total)?;
    send_confirmation(order.customer_email)?;

    Ok(order_id)
}

#[aspect(TransactionalAspect::new())]
fn allocate_inventory(items: Vec<OrderItem>) -> Result<(), Error> {
    for item in items {
        database::decrement_stock(item.product_id, item.quantity)?;
    }
    Ok(())
}

#[aspect(TransactionalAspect::new())]
fn process_payment(amount: f64) -> Result<(), Error> {
    database::record_payment(amount)?;
    Ok(())
}
```

### Read-Only Transactions

Optimize for read-heavy operations:

```rust
use aspect_std::TransactionalAspect;

#[aspect(TransactionalAspect::read_only())]
fn generate_report(start_date: Date, end_date: Date) -> Result<Report, Error> {
    // Read-only transaction:
    // - Consistent snapshot of data
    // - No write locks
    // - Better performance
    // - Still ACID compliant

    let users = database::get_users_in_range(start_date, end_date)?;
    let transactions = database::get_transactions_in_range(start_date, end_date)?;

    Ok(Report::generate(users, transactions))
}
```

### Transaction Isolation Levels

Control isolation for specific use cases:

```rust
use aspect_std::TransactionalAspect;
use aspect_std::IsolationLevel;

// Serializable: Highest isolation, prevents phantom reads
#[aspect(TransactionalAspect::with_isolation(IsolationLevel::Serializable))]
fn critical_financial_operation(data: FinancialData) -> Result<(), Error> {
    // Strictest consistency guarantees
    database::process_critical_transaction(data)
}

// Read Committed: Lower isolation, better performance
#[aspect(TransactionalAspect::with_isolation(IsolationLevel::ReadCommitted))]
fn generate_dashboard(user_id: u64) -> Result<Dashboard, Error> {
    // Acceptable for non-critical reads
    database::fetch_dashboard_data(user_id)
}
```

### Real-World Example: E-Commerce

```rust
use aspect_std::{TransactionalAspect, LoggingAspect, TimingAspect};

#[aspect(TransactionalAspect::new())]
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn checkout(cart: ShoppingCart, payment: PaymentInfo) -> Result<Receipt, Error> {
    // All-or-nothing transaction

    // 1. Reserve inventory
    for item in &cart.items {
        database::reserve_item(item.product_id, item.quantity)?;
    }

    // 2. Process payment
    let charge_id = payment_gateway::charge(payment, cart.total())?;
    database::record_charge(charge_id)?;

    // 3. Create order
    let order_id = database::create_order(&cart, charge_id)?;

    // 4. Commit inventory changes
    for item in &cart.items {
        database::commit_reservation(item.product_id, item.quantity)?;
    }

    // 5. Generate receipt
    Ok(Receipt {
        order_id,
        charge_id,
        items: cart.items.clone(),
        total: cart.total(),
    })

    // If any step fails, entire transaction rolls back:
    // - Inventory released
    // - Payment refunded
    // - Order not created
}
```

## Production Best Practices

### Aspect Composition

Order matters when stacking aspects:

```rust
// Correct order (outside to inside):
#[aspect(AuthorizationAspect::require_role("admin", get_roles))]  // 1. Check auth first
#[aspect(RateLimitAspect::new(10, Duration::from_secs(60)))]      // 2. Then rate limit
#[aspect(TransactionalAspect::new())]                              // 3. Start transaction
#[aspect(LoggingAspect::new())]                                    // 4. Log execution
#[aspect(TimingAspect::new())]                                     // 5. Measure time
fn sensitive_operation(data: Data) -> Result<(), Error> {
    database::process(data)
}
```

**Rationale:**
1. Authorization: Reject unauthorized users immediately
2. Rate Limiting: Prevent abuse before expensive operations
3. Transaction: Only start transaction for valid requests
4. Logging: Log all execution attempts
5. Timing: Measure actual business logic

### Error Handling Strategy

```rust
use aspect_std::{LoggingAspect, MetricsAspect};

#[aspect(LoggingAspect::new())]
#[aspect(MetricsAspect::new())]
fn robust_api_call(params: Params) -> Result<Response, ApiError> {
    // Aspects automatically handle:
    // - Logging entry/exit/errors
    // - Recording success/failure metrics

    validate_params(&params)?;

    let response = external_api::call(params)?;

    validate_response(&response)?;

    Ok(response)
}

// After_error advice in aspects captures all errors automatically
```

### Performance Monitoring

Monitor aspect overhead in production:

```rust
use aspect_std::{TimingAspect, MetricsAspect};

#[aspect(TimingAspect::with_threshold(Duration::from_millis(100)))]
#[aspect(MetricsAspect::with_percentiles(vec![50, 95, 99]))]
fn monitored_endpoint(request: Request) -> Result<Response, Error> {
    // TimingAspect: Warns if execution > 100ms
    // MetricsAspect: Records p50, p95, p99 latencies

    process_request(request)
}
```

### Graceful Degradation

Use circuit breakers with fallbacks:

```rust
use aspect_std::CircuitBreakerAspect;

#[aspect(CircuitBreakerAspect::new(5, Duration::from_secs(30)))]
fn fetch_recommendations(user_id: u64) -> Result<Vec<Product>, Error> {
    recommendation_service::get(user_id)
}

fn get_recommendations_with_fallback(user_id: u64) -> Vec<Product> {
    match fetch_recommendations(user_id) {
        Ok(products) => products,
        Err(_) => {
            // Circuit open or service down - use fallback
            get_popular_products() // Default recommendations
        }
    }
}
```

## Summary

Production patterns covered:

1. **Caching**: Improve performance with transparent caching
2. **Rate Limiting**: Protect resources from exhaustion
3. **Circuit Breakers**: Prevent cascading failures
4. **Transactions**: Ensure data consistency

**Key Takeaways:**
- Aspects separate infrastructure concerns from business logic
- Multiple aspects compose cleanly
- Order matters when stacking aspects
- Production systems benefit from declarative cross-cutting concerns
- aspect-rs overhead is negligible (<5%) for most patterns

**Next Steps:**
- See [Advanced Patterns](advanced.md) for composition techniques
- Review [Configuration](configuration.md) for environment-specific settings
- Check [Benchmarks](../ch09-benchmarks/README.md) for performance data
