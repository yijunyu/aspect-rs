# Building a Production API Server with Aspects

This chapter demonstrates a comprehensive, production-ready API server implementation using aspect-oriented programming. We'll build a RESTful user management API with logging, performance monitoring, and error handling, all managed through aspects.

## Overview

The API server example showcases:

- **Multiple aspects working together** in a real-world scenario
- **Clean separation of concerns** between business logic and cross-cutting functionality
- **Minimal boilerplate** with maximum observability
- **Production patterns** for API development

By the end of this case study, you'll understand how to structure a complete application using aspects to handle common concerns like logging, timing, validation, and error handling.

## The Problem: Cross-Cutting Concerns in APIs

Traditional API implementations mix business logic with infrastructure concerns:

```rust
// Traditional approach - everything tangled together
pub fn get_user(db: Database, id: u64) -> Result<Option<User>, Error> {
    // Logging
    println!("[INFO] GET /users/{} called at {}", id, Instant::now());

    // Timing
    let start = Instant::now();

    // Actual business logic (buried in infrastructure code)
    let result = db.lock().unwrap().get(&id).cloned();

    // More timing
    let duration = start.elapsed();
    println!("[PERF] Request took {:?}", duration);

    // More logging
    println!("[INFO] GET /users/{} returned {:?}", id, result.is_some());

    Ok(result)
}
```

**Problems with this approach:**

1. Business logic is hard to find amid infrastructure code
2. Logging/timing code must be duplicated across all endpoints
3. Easy to forget instrumentation for new endpoints
4. Hard to change logging format or add new concerns
5. Testing business logic requires mocking infrastructure

## The Solution: Aspect-Oriented API Server

With aspects, we separate concerns cleanly:

```rust
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn get_user(db: Database, id: u64) -> Result<Option<User>, AspectError> {
    println!("  [HANDLER] GET /users/{}", id);
    Ok(db.lock().unwrap().get(&id).cloned())
}
```

The business logic is now clear and concise. All infrastructure concerns are handled by reusable aspects.

## Complete Implementation

### Domain Models

First, let's define our data structures:

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;
use aspect_std::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
}

// Thread-safe database abstraction
type Database = Arc<Mutex<HashMap<u64, User>>>;

fn init_database() -> Database {
    let db = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut users = db.lock().unwrap();
        users.insert(
            1,
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
            },
        );
        users.insert(
            2,
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        );
    }
    db
}
```

### API Handler Functions

Now let's implement our API endpoints with aspects:

#### GET /users/:id - Retrieve a User

```rust
/// GET /users/:id - with logging and timing
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn get_user(db: Database, id: u64) -> Result<Option<User>, AspectError> {
    println!("  [HANDLER] GET /users/{}", id);
    std::thread::sleep(std::time::Duration::from_millis(10)); // Simulate work
    Ok(db.lock().unwrap().get(&id).cloned())
}
```

**What happens when this runs:**

1. `LoggingAspect` executes `before()` - logs function entry
2. `TimingAspect` executes `before()` - records start time
3. Business logic runs - queries the database
4. `TimingAspect` executes `after()` - calculates duration
5. `LoggingAspect` executes `after()` - logs function exit

**Output:**
```
[LOG] → Entering: get_user
[TIMING] ⏱  Starting: get_user
  [HANDLER] GET /users/1
[TIMING] ✓ get_user completed in 10.2ms
[LOG] ← Exiting: get_user
```

#### POST /users - Create a New User

```rust
/// POST /users - with logging and timing
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn create_user(
    db: Database,
    id: u64,
    username: String,
    email: String,
) -> Result<User, AspectError> {
    println!("  [HANDLER] POST /users");

    // Validation
    if username.is_empty() {
        return Err(AspectError::execution("Username cannot be empty"));
    }
    if !email.contains('@') {
        return Err(AspectError::execution("Invalid email format"));
    }

    std::thread::sleep(std::time::Duration::from_millis(15)); // Simulate work

    let user = User {
        id,
        username,
        email,
    };
    db.lock().unwrap().insert(id, user.clone());
    Ok(user)
}
```

**Key features:**

- **Validation** is part of business logic (belongs in the function)
- **Logging and timing** are cross-cutting concerns (handled by aspects)
- **Error handling** integrates seamlessly with aspects
- When validation fails, aspects automatically log the error via `after_error()`

**Success output:**
```
[LOG] → Entering: create_user
[TIMING] ⏱  Starting: create_user
  [HANDLER] POST /users
[TIMING] ✓ create_user completed in 15.3ms
[LOG] ← Exiting: create_user
```

**Validation failure output:**
```
[LOG] → Entering: create_user
[TIMING] ⏱  Starting: create_user
  [HANDLER] POST /users
[TIMING] ✗ create_user failed after 0.1ms
[LOG] ✗ create_user failed with error: Invalid email format
```

#### GET /users - List All Users

```rust
/// GET /users - with logging and timing
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn list_users(db: Database) -> Result<Vec<User>, AspectError> {
    println!("  [HANDLER] GET /users");
    std::thread::sleep(std::time::Duration::from_millis(20)); // Simulate work
    Ok(db.lock().unwrap().values().cloned().collect())
}
```

**Notice the pattern:**

Every handler follows the same structure:
- Add `#[aspect(...)]` attributes for desired functionality
- Focus solely on business logic in the function body
- Let aspects handle infrastructure concerns

This consistency makes the codebase easier to understand and maintain.

#### DELETE /users/:id - Delete a User

```rust
/// DELETE /users/:id - with logging and timing
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
fn delete_user(db: Database, id: u64) -> Result<bool, AspectError> {
    println!("  [HANDLER] DELETE /users/{}", id);
    std::thread::sleep(std::time::Duration::from_millis(12)); // Simulate work
    Ok(db.lock().unwrap().remove(&id).is_some())
}
```

**Return value conventions:**

- `Result<bool, AspectError>` - `true` if deleted, `false` if not found
- Aspects log both successful deletions and "not found" cases
- Error handling is consistent across all endpoints

### Complete Application

Here's the main application that demonstrates all endpoints:

```rust
fn main() {
    println!("=== API Server with Aspects Demo ===\n");
    println!("This example shows multiple aspects applied to API handlers:");
    println!("- LoggingAspect: Tracks entry/exit of each handler");
    println!("- TimingAspect: Measures execution time\n");

    let db = init_database();

    // 1. GET /users/1 (existing user)
    println!("1. GET /users/1 (existing user)");
    match get_user(db.clone(), 1) {
        Ok(Some(user)) => println!("   Found: {} ({})\n", user.username, user.email),
        Ok(None) => println!("   Not found\n"),
        Err(e) => println!("   Error: {:?}\n", e),
    }

    // 2. GET /users/999 (non-existent user)
    println!("2. GET /users/999 (non-existent user)");
    match get_user(db.clone(), 999) {
        Ok(Some(user)) => println!("   Found: {}\n", user.username),
        Ok(None) => println!("   Not found\n"),
        Err(e) => println!("   Error: {:?}\n", e),
    }

    // 3. POST /users (create new user)
    println!("3. POST /users (create new user)");
    match create_user(
        db.clone(),
        3,
        "charlie".to_string(),
        "charlie@example.com".to_string(),
    ) {
        Ok(user) => println!("   Created: {} (ID: {})\n", user.username, user.id),
        Err(e) => println!("   Error: {:?}\n", e),
    }

    // 4. POST /users (invalid email)
    println!("4. POST /users (invalid email)");
    match create_user(db.clone(), 4, "dave".to_string(), "invalid-email".to_string()) {
        Ok(user) => println!("   Created: {}\n", user.username),
        Err(e) => println!("   Validation failed: {}\n", e),
    }

    // 5. GET /users (list all)
    println!("5. GET /users (list all)");
    match list_users(db.clone()) {
        Ok(users) => {
            println!("   Found {} users:", users.len());
            for user in users {
                println!("     - {} ({})", user.username, user.email);
            }
            println!();
        }
        Err(e) => println!("   Error: {:?}\n", e),
    }

    // 6. DELETE /users/2
    println!("6. DELETE /users/2");
    match delete_user(db.clone(), 2) {
        Ok(true) => println!("   Deleted successfully\n"),
        Ok(false) => println!("   User not found\n"),
        Err(e) => println!("   Error: {:?}\n", e),
    }

    println!("=== Demo Complete ===\n");
    println!("Key Takeaways:");
    println!("✓ Logging automatically applied to all handlers");
    println!("✓ Timing measured for each request");
    println!("✓ Error handling integrated with aspects");
    println!("✓ Clean separation of concerns");
    println!("✓ No manual instrumentation needed!");
}
```

## Running the Example

To run this complete example:

```bash
# Navigate to the examples directory
cd aspect-rs/aspect-examples

# Run the API server example
cargo run --example api_server
```

**Expected output:**

```
=== API Server with Aspects Demo ===

This example shows multiple aspects applied to API handlers:
- LoggingAspect: Tracks entry/exit of each handler
- TimingAspect: Measures execution time

1. GET /users/1 (existing user)
[LOG] → Entering: get_user
[TIMING] ⏱  Starting: get_user
  [HANDLER] GET /users/1
[TIMING] ✓ get_user completed in 10.2ms
[LOG] ← Exiting: get_user
   Found: alice (alice@example.com)

2. GET /users/999 (non-existent user)
[LOG] → Entering: get_user
[TIMING] ⏱  Starting: get_user
  [HANDLER] GET /users/999
[TIMING] ✓ get_user completed in 10.1ms
[LOG] ← Exiting: get_user
   Not found

3. POST /users (create new user)
[LOG] → Entering: create_user
[TIMING] ⏱  Starting: create_user
  [HANDLER] POST /users
[TIMING] ✓ create_user completed in 15.3ms
[LOG] ← Exiting: create_user
   Created: charlie (ID: 3)

4. POST /users (invalid email)
[LOG] → Entering: create_user
[TIMING] ⏱  Starting: create_user
  [HANDLER] POST /users
[TIMING] ✗ create_user failed after 0.1ms
[LOG] ✗ create_user failed with error: Invalid email format
   Validation failed: Invalid email format

5. GET /users (list all)
[LOG] → Entering: list_users
[TIMING] ⏱  Starting: list_users
  [HANDLER] GET /users
[TIMING] ✓ list_users completed in 20.4ms
[LOG] ← Exiting: list_users
   Found 3 users:
     - alice (alice@example.com)
     - charlie (charlie@example.com)
     - bob (bob@example.com)

6. DELETE /users/2
[LOG] → Entering: delete_user
[TIMING] ⏱  Starting: delete_user
  [HANDLER] DELETE /users/2
[TIMING] ✓ delete_user completed in 12.1ms
[LOG] ← Exiting: delete_user
   Deleted successfully

=== Demo Complete ===

Key Takeaways:
✓ Logging automatically applied to all handlers
✓ Timing measured for each request
✓ Error handling integrated with aspects
✓ Clean separation of concerns
✓ No manual instrumentation needed!
```

## Extending the Example

### Adding More Aspects

You can easily add additional cross-cutting concerns:

```rust
// Add caching
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(CachingAspect::new(Duration::from_secs(60)))]
fn get_user(db: Database, id: u64) -> Result<Option<User>, AspectError> {
    // Business logic unchanged!
    Ok(db.lock().unwrap().get(&id).cloned())
}

// Add rate limiting
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
#[aspect(RateLimitAspect::new(100, Duration::from_secs(60)))]
fn create_user(/* ... */) -> Result<User, AspectError> {
    // Business logic unchanged!
}
```

**No changes to business logic required!** Just add attributes.

### Custom Validation Aspect

You can create domain-specific aspects:

```rust
struct ValidationAspect;

impl Aspect for ValidationAspect {
    fn before(&self, ctx: &JoinPoint) {
        println!("[VALIDATE] Checking preconditions for {}", ctx.function_name);
        // Add custom validation logic
    }
}

#[aspect(ValidationAspect)]
#[aspect(LoggingAspect::new())]
fn create_user(/* ... */) -> Result<User, AspectError> {
    // Validation runs before logging
}
```

### Request/Response Middleware

Simulate HTTP middleware with aspects:

```rust
struct RequestIdAspect {
    counter: AtomicU64,
}

impl RequestIdAspect {
    fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }
}

impl Aspect for RequestIdAspect {
    fn before(&self, ctx: &JoinPoint) {
        let req_id = self.counter.fetch_add(1, Ordering::SeqCst);
        println!("[REQUEST-ID] {} - Request #{}", ctx.function_name, req_id);
    }
}

#[aspect(RequestIdAspect::new())]
#[aspect(LoggingAspect::new())]
fn get_user(/* ... */) -> Result<Option<User>, AspectError> {
    // Each request gets unique ID
}
```

## Performance Considerations

### Overhead Analysis

Based on the timing aspect output, we can measure overhead:

```
Business logic time: ~10-20ms (database + sleep simulation)
Aspect overhead: <0.1ms (logging + timing)
Total overhead: <1% of request time
```

For typical API operations that involve I/O, database queries, or computation, aspect overhead is **negligible**.

### When Aspects Make Sense for APIs

**Good use cases:**

- ✅ Request logging
- ✅ Performance monitoring
- ✅ Authentication/authorization
- ✅ Rate limiting
- ✅ Caching
- ✅ Metrics collection
- ✅ Error tracking

**Less ideal:**

- ❌ High-frequency in-memory operations (aspect overhead becomes significant)
- ❌ Tight loops (consider manual optimization)
- ❌ Real-time systems with microsecond budgets

### Optimization Tips

1. **Reuse aspect instances** - don't create new aspects per request
2. **Use async aspects** for I/O-heavy operations
3. **Batch logging** instead of per-request writes
4. **Profile first** before optimizing

## Integration with Real Frameworks

### Axum Integration

```rust
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

// Aspect-decorated handlers work with Axum
#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
async fn axum_get_user(
    State(db): State<Database>,
    Path(id): Path<u64>,
) -> Json<Option<User>> {
    Json(db.lock().unwrap().get(&id).cloned())
}

async fn main() {
    let app = Router::new()
        .route("/users/:id", get(axum_get_user))
        .with_state(init_database());

    // Run server...
}
```

### Actix-Web Integration

```rust
use actix_web::{web, App, HttpServer, Responder};

#[aspect(LoggingAspect::new())]
#[aspect(TimingAspect::new())]
async fn actix_get_user(
    db: web::Data<Database>,
    id: web::Path<u64>,
) -> impl Responder {
    web::Json(db.lock().unwrap().get(&id.into_inner()).cloned())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(init_database()))
            .route("/users/{id}", web::get().to(actix_get_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## Testing

### Unit Testing with Aspects

Aspects don't interfere with testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user() {
        let db = init_database();

        // Aspects run during test
        let result = get_user(db, 1).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "alice");
    }

    #[test]
    fn test_create_user_validation() {
        let db = init_database();

        // Test validation logic
        let result = create_user(db, 999, "test".to_string(), "invalid".to_string());

        assert!(result.is_err());
    }
}
```

### Mocking Aspects for Testing

You can disable aspects in tests if needed:

```rust
#[cfg(not(test))]
use aspect_std::prelude::*;

#[cfg(test)]
mod mock_aspects {
    pub struct LoggingAspect;
    impl LoggingAspect {
        pub fn new() -> Self { Self }
    }
    impl Aspect for LoggingAspect {}
}

#[cfg(test)]
use mock_aspects::*;
```

## Key Takeaways

After studying this API server example, you should understand:

1. **Separation of Concerns**
   - Business logic stays clean and focused
   - Infrastructure concerns are handled by reusable aspects
   - Easy to add/remove functionality without touching business code

2. **Composability**
   - Multiple aspects work together seamlessly
   - Aspects can be stacked in any order
   - Each aspect is independent and reusable

3. **Maintainability**
   - Consistent patterns across all endpoints
   - Changes to logging/timing affect all handlers automatically
   - Impossible to forget instrumentation for new endpoints

4. **Production Readiness**
   - Error handling integrates naturally
   - Performance overhead is negligible for typical APIs
   - Easy to integrate with existing web frameworks

5. **Developer Experience**
   - Less boilerplate code to write
   - Easier to understand and review
   - Faster to add new endpoints

## Next Steps

- See [Security Case Study](./security.md) for authentication/authorization patterns
- See [Resilience Case Study](./resilience.md) for retry and circuit breaker patterns
- See [Transaction Case Study](./transactions.md) for database transaction management
- See [Chapter 9: Benchmarks](../ch09-benchmarks/README.md) for performance analysis

## Source Code

The complete working code for this example is available at:
```
aspect-rs/aspect-examples/src/api_server.rs
```

Run it with:
```bash
cargo run --example api_server
```

---

**Related Chapters:**
- [Chapter 5: Usage Guide](../ch05-usage/README.md) - Basic aspect usage
- [Chapter 6: Architecture](../ch06-architecture/README.md) - Framework design
- [Chapter 8.2: Security](./security.md) - Authorization with aspects
- [Chapter 9: Benchmarks](../ch09-benchmarks/README.md) - Performance data
