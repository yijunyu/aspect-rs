# The Problem: Crosscutting Concerns

## What Are Crosscutting Concerns?

**Crosscutting concerns** are aspects of a program that affect multiple modules but don't fit neatly into a single component. They "cut across" the modularity of your application.

Common examples:
- **Logging** - Every function needs entry/exit logs
- **Performance monitoring** - Measure execution time everywhere
- **Security** - Authorization checks across API endpoints
- **Caching** - Memoize expensive computations
- **Transactions** - Database transaction management
- **Error handling** - Retry logic, circuit breakers
- **Validation** - Input validation across public APIs

## The Traditional Approach: Scattered Code

Without AOP, you manually add crosscutting code to every function:

```rust
fn fetch_user(id: u64) -> Result<User, Error> {
    // Logging
    log::info!("Entering fetch_user({})", id);
    let start = Instant::now();

    // Authorization
    if !check_permission("read_user") {
        log::error!("Unauthorized access to fetch_user");
        return Err(Error::Unauthorized);
    }

    // Metrics
    metrics::increment("fetch_user.calls");

    // Business logic (buried in crosscutting code!)
    let result = database::query_user(id);

    // More logging
    match &result {
        Ok(_) => log::info!("fetch_user succeeded in {:?}", start.elapsed()),
        Err(e) => log::error!("fetch_user failed: {}", e),
    }

    // More metrics
    metrics::record("fetch_user.latency", start.elapsed());

    result
}
```

**Problems:**

1. **Noise** - Business logic (line 15) is buried in 20 lines of boilerplate
2. **Duplication** - Same logging/metrics code repeated in every function
3. **Error-prone** - Forgetting to add authorization is a security vulnerability
4. **Maintenance nightmare** - Changing log format requires touching every function
5. **Testing difficulty** - Can't test business logic independently

## Real-World Impact

Consider a microservice with 50 API endpoints:

- **Manual approach**: ~1,500 lines of repeated logging/metrics/auth code
- **Maintenance**: Updating logging format touches 50 files
- **Bugs**: Missed authorization check on 1 endpoint = security breach
- **Testing**: Must mock logging/metrics in every unit test

### Example: Adding Caching

You decide to add caching to 10 expensive database queries:

```rust
// Before caching - simple
fn get_user_profile(id: u64) -> Profile {
    database::query("SELECT * FROM profiles WHERE user_id = ?", id)
}
```

```rust
// After caching - complexity explosion
fn get_user_profile(id: u64) -> Profile {
    // Check cache
    let cache_key = format!("profile:{}", id);
    if let Some(cached) = CACHE.get(&cache_key) {
        metrics::increment("cache.hit");
        return cached;
    }

    // Cache miss - query database
    metrics::increment("cache.miss");
    let profile = database::query("SELECT * FROM profiles WHERE user_id = ?", id);

    // Store in cache
    CACHE.set(&cache_key, &profile, Duration::from_secs(300));

    profile
}
```

**Multiply by 10 functions** = 150+ lines of duplicated cache logic!

## The Root Cause

The problem is that **traditional programming languages force you to mix orthogonal concerns**:

- **Horizontal concern**: Business logic (what the function does)
- **Vertical concerns**: Logging, metrics, caching (how it's observed/optimized)

These concerns should be **separate**, but they're tangled together.

## What We Need

An ideal solution would:

1. ✅ **Separate concerns** - Business logic lives alone
2. ✅ **Reuse code** - Write logging once, apply everywhere
3. ✅ **Maintain safety** - Can't forget to apply authorization
4. ✅ **Zero overhead** - No runtime cost
5. ✅ **Easy to change** - Update logging in one place

This is exactly what **Aspect-Oriented Programming** provides. Let's see how in the next section.
