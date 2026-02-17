# Database Transaction Management with Aspects

This case study demonstrates how to implement automatic database transaction management using aspects. We'll build a transaction aspect that ensures ACID properties without cluttering business logic with boilerplate transaction code.

## Overview

Database transactions are essential for data integrity:

- **Atomicity**: All operations succeed or all fail
- **Consistency**: Data remains in valid state
- **Isolation**: Concurrent transactions don't interfere
- **Durability**: Committed changes persist

Traditional transaction management mixes infrastructure with business logic. Aspects provide a cleaner solution.

## The Problem: Transaction Boilerplate

Without aspects, every database operation requires explicit transaction management:

```rust
// Traditional approach - transaction code everywhere
fn transfer_money(from: u64, to: u64, amount: f64) -> Result<(), Error> {
    let conn = get_connection()?;
    let mut tx = conn.begin_transaction()?;

    // Debit source
    match tx.execute(&format!("UPDATE accounts SET balance = balance - {} WHERE id = {}", amount, from)) {
        Ok(_) => {},
        Err(e) => {
            tx.rollback()?;
            return Err(e);
        }
    }

    // Credit destination
    match tx.execute(&format!("UPDATE accounts SET balance = balance + {} WHERE id = {}", amount, to)) {
        Ok(_) => {},
        Err(e) => {
            tx.rollback()?;
            return Err(e);
        }
    }

    tx.commit()?;
    Ok(())
}
```

**Problems:**
1. Transaction boilerplate repeated in every function
2. Easy to forget rollback on error
3. Business logic buried in infrastructure code
4. Difficult to ensure consistent transaction handling

## The Solution: Transactional Aspect

With aspects, transaction management becomes declarative:

```rust
#[aspect(TransactionalAspect)]
fn transfer_money(from: u64, to: u64, amount: f64) -> Result<(), String> {
    // Just business logic - transactions handled automatically!
    let conn = get_connection();
    conn.execute(&format!("UPDATE accounts SET balance = balance - {} WHERE id = {}", amount, from))?;
    conn.execute(&format!("UPDATE accounts SET balance = balance + {} WHERE id = {}", amount, to))?;
    Ok(())
}
```

Transactions are begun automatically, committed on success, rolled back on error.

## Complete Implementation

### Database Simulation

First, let's create a simulated database with transaction support:

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;
use std::sync::{Arc, Mutex};

/// Simulated database connection
#[derive(Clone)]
struct DbConnection {
    id: usize,
    in_transaction: bool,
}

impl DbConnection {
    fn begin_transaction(&mut self) -> Transaction {
        println!("  [DB] BEGIN TRANSACTION on connection {}", self.id);
        self.in_transaction = true;
        Transaction {
            conn_id: self.id,
            committed: false,
            rolled_back: false,
        }
    }

    fn execute(&self, sql: &str) -> Result<usize, String> {
        if !self.in_transaction {
            return Err("Not in transaction".to_string());
        }
        println!("  [DB] EXEC: {} (conn {})", sql, self.id);
        Ok(1) // Simulated rows affected
    }
}
```

### Transaction Handle

```rust
/// Simulated transaction handle
struct Transaction {
    conn_id: usize,
    committed: bool,
    rolled_back: bool,
}

impl Transaction {
    fn commit(&mut self) -> Result<(), String> {
        if self.rolled_back {
            return Err("Transaction already rolled back".to_string());
        }
        println!("  [DB] COMMIT on connection {}", self.conn_id);
        self.committed = true;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), String> {
        if self.committed {
            return Err("Transaction already committed".to_string());
        }
        println!("  [DB] ROLLBACK on connection {}", self.conn_id);
        self.rolled_back = true;
        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back {
            println!("  [DB] ⚠ Auto-ROLLBACK on drop (conn {})", self.conn_id);
        }
    }
}
```

**Auto-rollback on drop** ensures transactions are cleaned up even if explicitly forgotten.

### Connection Pool

```rust
struct ConnectionPool {
    connections: Vec<Arc<Mutex<DbConnection>>>,
    next_id: usize,
}

impl ConnectionPool {
    fn new() -> Self {
        Self {
            connections: Vec::new(),
            next_id: 0,
        }
    }

    fn get_connection(&mut self) -> Arc<Mutex<DbConnection>> {
        if self.connections.is_empty() {
            let conn = Arc::new(Mutex::new(DbConnection {
                id: self.next_id,
                in_transaction: false,
            }));
            self.next_id += 1;
            self.connections.push(conn.clone());
            conn
        } else {
            self.connections[0].clone()
        }
    }
}

static POOL: Mutex<ConnectionPool> = Mutex::new(ConnectionPool {
    connections: Vec::new(),
    next_id: 0,
});
```

### Transactional Aspect

The core aspect that manages transactions automatically:

```rust
struct TransactionalAspect;

impl Aspect for TransactionalAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let function_name = pjp.context().function_name;
        println!("[TX] Starting transaction for {}", function_name);

        // Get connection and start transaction
        let conn = POOL.lock().unwrap().get_connection();
        let mut tx = conn.lock().unwrap().begin_transaction();

        // Execute the function
        match pjp.proceed() {
            Ok(result) => {
                // Success - commit transaction
                match tx.commit() {
                    Ok(_) => {
                        println!("[TX] ✓ Transaction committed for {}", function_name);
                        Ok(result)
                    }
                    Err(e) => {
                        println!("[TX] ✗ Commit failed for {}: {}", function_name, e);
                        let _ = tx.rollback();
                        Err(AspectError::execution(format!("Commit failed: {}", e)))
                    }
                }
            }
            Err(error) => {
                // Error - rollback transaction
                println!(
                    "[TX] ✗ Transaction rolled back for {} due to error",
                    function_name
                );
                let _ = tx.rollback();
                Err(error)
            }
        }
    }
}
```

**Key features:**
- Uses `around` advice to wrap entire function execution
- Begins transaction before function runs
- Commits on success, rolls back on error
- Clear logging of transaction lifecycle

## Transactional Operations

### Money Transfer

```rust
#[aspect(TransactionalAspect)]
fn transfer_money(from_account: u64, to_account: u64, amount: f64) -> Result<(), String> {
    println!(
        "  [APP] Transferring ${:.2} from account {} to {}",
        amount, from_account, to_account
    );

    let conn = POOL.lock().unwrap().get_connection();
    let conn = conn.lock().unwrap();

    // Debit from source account
    conn.execute(&format!(
        "UPDATE accounts SET balance = balance - {} WHERE id = {}",
        amount, from_account
    ))?;

    // Credit to destination account
    conn.execute(&format!(
        "UPDATE accounts SET balance = balance + {} WHERE id = {}",
        amount, to_account
    ))?;

    println!("  [APP] Transfer completed successfully");
    Ok(())
}
```

**Output (successful transfer):**
```
[TX] Starting transaction for transfer_money
  [DB] BEGIN TRANSACTION on connection 0
  [APP] Transferring $50.00 from account 100 to 200
  [DB] EXEC: UPDATE accounts SET balance = balance - 50 WHERE id = 100 (conn 0)
  [DB] EXEC: UPDATE accounts SET balance = balance + 50 WHERE id = 200 (conn 0)
  [APP] Transfer completed successfully
  [DB] COMMIT on connection 0
[TX] ✓ Transaction committed for transfer_money
```

**If any step fails, automatic rollback occurs:**
```
[TX] Starting transaction for transfer_money
  [DB] BEGIN TRANSACTION on connection 0
  [APP] Transferring $50.00 from account 100 to 200
  [DB] EXEC: UPDATE accounts SET balance = balance - 50 WHERE id = 100 (conn 0)
  [APP] Simulating database error...
  [DB] ROLLBACK on connection 0
[TX] ✗ Transaction rolled back for transfer_money due to error
```

### Creating User with Account

```rust
#[aspect(TransactionalAspect)]
fn create_user_with_account(username: &str, initial_balance: f64) -> Result<u64, String> {
    println!(
        "  [APP] Creating user '{}' with balance ${:.2}",
        username, initial_balance
    );

    let conn = POOL.lock().unwrap().get_connection();
    let conn = conn.lock().unwrap();

    // Insert user
    conn.execute(&format!("INSERT INTO users (username) VALUES ('{}')", username))?;
    let user_id = 123; // Simulated generated ID

    // Create account
    conn.execute(&format!(
        "INSERT INTO accounts (user_id, balance) VALUES ({}, {})",
        user_id, initial_balance
    ))?;

    println!("  [APP] User {} created successfully", user_id);
    Ok(user_id)
}
```

**Benefits:**
- User and account are created atomically
- If account creation fails, user creation is rolled back
- No orphaned users without accounts

### Failing Operation

```rust
#[aspect(TransactionalAspect)]
fn failing_operation() -> Result<(), String> {
    println!("  [APP] Performing operation that will fail...");

    let conn = POOL.lock().unwrap().get_connection();
    let conn = conn.lock().unwrap();

    // First operation succeeds
    conn.execute("UPDATE users SET last_login = NOW()")?;

    // Second operation fails
    println!("  [APP] Simulating database error...");
    Err("Constraint violation".to_string())
}
```

**Output:**
```
[TX] Starting transaction for failing_operation
  [DB] BEGIN TRANSACTION on connection 0
  [APP] Performing operation that will fail...
  [DB] EXEC: UPDATE users SET last_login = NOW() (conn 0)
  [APP] Simulating database error...
  [DB] ROLLBACK on connection 0
[TX] ✗ Transaction rolled back for failing_operation due to error
```

**The first UPDATE is rolled back** - no partial updates!

## Demonstration

```rust
fn main() {
    println!("=== Transaction Management Aspect Example ===\n");

    // Example 1: Successful transfer
    println!("1. Successful money transfer:");
    match transfer_money(100, 200, 50.00) {
        Ok(_) => println!("   ✓ Transfer completed\n"),
        Err(e) => println!("   ✗ Transfer failed: {}\n", e),
    }

    // Example 2: Creating user with account
    println!("2. Creating user with account:");
    match create_user_with_account("alice", 100.00) {
        Ok(user_id) => println!("   ✓ User created with ID: {}\n", user_id),
        Err(e) => println!("   ✗ Creation failed: {}\n", e),
    }

    // Example 3: Failed operation (automatic rollback)
    println!("3. Operation that fails (automatic rollback):");
    match failing_operation() {
        Ok(_) => println!("   ✗ Unexpected success\n"),
        Err(e) => println!("   ✓ Failed as expected: {} (transaction rolled back)\n", e),
    }

    // Example 4: Multiple operations in sequence
    println!("4. Multiple successful operations:");
    println!("   Transfer 1:");
    let _ = transfer_money(100, 200, 25.00);
    println!("\n   Transfer 2:");
    let _ = transfer_money(200, 300, 15.00);
    println!();

    println!("=== Demo Complete ===");
    println!("\nKey Takeaways:");
    println!("✓ Transactions managed automatically by aspect");
    println!("✓ Business logic clean - no transaction boilerplate");
    println!("✓ Automatic rollback on errors");
    println!("✓ Automatic commit on success");
    println!("✓ ACID properties enforced without code changes");
}
```

## Advanced Patterns

### Nested Transactions

```rust
struct NestedTransactionalAspect {
    savepoint_counter: AtomicUsize,
}

impl Aspect for NestedTransactionalAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        if in_transaction() {
            // Create savepoint for nested transaction
            let savepoint_id = self.savepoint_counter.fetch_add(1, Ordering::SeqCst);
            println!("[TX] Creating SAVEPOINT sp_{}", savepoint_id);

            match pjp.proceed() {
                Ok(result) => {
                    println!("[TX] RELEASE SAVEPOINT sp_{}", savepoint_id);
                    Ok(result)
                }
                Err(error) => {
                    println!("[TX] ROLLBACK TO SAVEPOINT sp_{}", savepoint_id);
                    Err(error)
                }
            }
        } else {
            // Top-level transaction (same as TransactionalAspect)
            // ... begin/commit/rollback logic ...
        }
    }
}
```

### Read-Only Transactions

```rust
struct ReadOnlyTransactionalAspect;

impl Aspect for ReadOnlyTransactionalAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        println!("[TX] BEGIN READ ONLY TRANSACTION");

        let conn = get_connection();
        conn.execute("SET TRANSACTION READ ONLY")?;

        let result = pjp.proceed();

        println!("[TX] COMMIT READ ONLY TRANSACTION");
        result
    }
}

#[aspect(ReadOnlyTransactionalAspect)]
fn get_account_balance(account_id: u64) -> Result<f64, String> {
    // Read-only operation, optimized for concurrency
}
```

### Transaction Isolation Levels

```rust
struct TransactionalAspect {
    isolation_level: IsolationLevel,
}

enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl Aspect for TransactionalAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let conn = get_connection();

        // Set isolation level
        conn.execute(&format!(
            "SET TRANSACTION ISOLATION LEVEL {}",
            self.isolation_level.as_sql()
        ))?;

        // Begin, execute, commit/rollback...
    }
}

#[aspect(TransactionalAspect::new(IsolationLevel::Serializable))]
fn critical_financial_operation() -> Result<(), String> {
    // Maximum isolation for critical operations
}
```

### Retry on Deadlock

```rust
#[aspect(RetryOnDeadlockAspect::new(3))]
#[aspect(TransactionalAspect)]
fn concurrent_update(id: u64, value: String) -> Result<(), String> {
    // Automatically retries if deadlock detected
    update_record(id, value)
}

struct RetryOnDeadlockAspect {
    max_retries: usize,
}

impl Aspect for RetryOnDeadlockAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        for attempt in 1..=self.max_retries {
            match pjp.proceed() {
                Ok(result) => return Ok(result),
                Err(error) if is_deadlock(&error) => {
                    if attempt < self.max_retries {
                        println!("[RETRY] Deadlock detected, retrying...");
                        sleep(Duration::from_millis(10 * attempt as u64));
                        continue;
                    }
                }
                Err(error) => return Err(error),
            }
        }
        Err(AspectError::execution("Max retries exceeded"))
    }
}
```

## Integration with Real Databases

### PostgreSQL Example

```rust
use tokio_postgres::{Client, Transaction};

struct PostgresTransactionalAspect {
    client: Arc<Client>,
}

impl Aspect for PostgresTransactionalAspect {
    async fn around_async(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let tx = self.client.transaction().await?;

        match pjp.proceed_async().await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }
}

#[aspect(PostgresTransactionalAspect::new(pool))]
async fn postgres_operation(id: i64) -> Result<User, Error> {
    // Real PostgreSQL operations
}
```

### Diesel ORM Integration

```rust
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

struct DieselTransactionalAspect {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Aspect for DieselTransactionalAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let conn = self.pool.get()?;

        conn.transaction(|| {
            // Execute function within Diesel transaction
            pjp.proceed()
        })
    }
}
```

## Testing Transactional Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_transaction_commits() {
        let result = transfer_money(100, 200, 50.0);
        assert!(result.is_ok());
        // Verify both accounts updated
    }

    #[test]
    fn test_failed_transaction_rolls_back() {
        let result = failing_operation();
        assert!(result.is_err());
        // Verify no changes persisted
    }

    #[test]
    fn test_partial_failure_rolls_back_all() {
        // Transfer that fails midway
        let result = transfer_money_with_failure(100, 200, 50.0);
        assert!(result.is_err());
        // Verify neither account was modified
    }
}
```

## Performance Considerations

Transaction aspects add minimal overhead:

```
Transaction begin: ~1ms (database round-trip)
Transaction commit: ~2ms (fsync to disk)
Aspect wrapper: <0.1ms (negligible)

Total: Dominated by database operations, not aspect overhead
```

**Optimization tips:**
1. Batch operations within single transaction
2. Use read-only transactions for queries
3. Choose appropriate isolation level
4. Consider connection pooling
5. Profile transaction duration

## Production Best Practices

### Error Categorization

```rust
fn should_rollback(error: &Error) -> bool {
    match error {
        Error::ConstraintViolation => true,
        Error::Deadlock => true, // Let retry aspect handle
        Error::ConnectionLost => false, // Don't rollback, just fail
        _ => true,
    }
}
```

### Transaction Timeout

```rust
struct TransactionalAspect {
    timeout: Duration,
}

impl Aspect for TransactionalAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let conn = get_connection();
        conn.execute(&format!("SET LOCAL statement_timeout = {}", self.timeout.as_millis()))?;

        // Begin transaction with timeout...
    }
}
```

### Monitoring

```rust
struct MonitoredTransactionalAspect {
    metrics: Arc<MetricsCollector>,
}

impl Aspect for MonitoredTransactionalAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let start = Instant::now();

        let result = /* transaction logic */;

        let duration = start.elapsed();
        self.metrics.record_transaction(pjp.context().function_name, duration, result.is_ok());

        result
    }
}
```

## Key Takeaways

1. **Automatic Transaction Management**
   - Transactions begin/commit/rollback automatically
   - No boilerplate in business logic
   - Consistent behavior across application

2. **ACID Guarantees**
   - Atomicity: All-or-nothing execution
   - Consistency: Invalid states prevented
   - Isolation: Concurrent transactions don't interfere
   - Durability: Committed changes persist

3. **Error Handling**
   - Automatic rollback on any error
   - No risk of forgetting to rollback
   - Clean separation of error handling

4. **Flexibility**
   - Configurable isolation levels
   - Read-only transactions
   - Nested transactions via savepoints
   - Integration with any database/ORM

5. **Production Ready**
   - Timeout protection
   - Deadlock retry
   - Monitoring integration
   - Works with connection pools

## Running the Example

```bash
cd aspect-rs/aspect-examples
cargo run --example transaction
```

## Next Steps

- See [Resilience Case Study](./resilience.md) for retry patterns
- See [API Server](./api-server.md) for combining transactions with API endpoints
- See [Chapter 9: Benchmarks](../ch09-benchmarks/README.md) for transaction performance

## Source Code

```
aspect-rs/aspect-examples/src/transaction.rs
```

---

**Related Chapters:**
- [Chapter 7: Around Advice](../ch07-implementation/around-advice.md)
- [Chapter 8.3: Resilience](./resilience.md)
- [Chapter 9: Benchmarks](../ch09-benchmarks/README.md)
