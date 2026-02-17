# Security and Authorization with Aspects

This case study demonstrates how to implement role-based access control (RBAC) and audit logging using aspect-oriented programming. We'll build a comprehensive security system that enforces authorization policies declaratively, without cluttering business logic.

## Overview

Security is a classic cross-cutting concern that affects many parts of an application:

- **Authorization checks** must be performed consistently across all protected operations
- **Audit logging** is required for compliance and security monitoring
- **Security policies** need to be centralized and easy to update
- **Business logic** should remain focused on functionality, not security details

This example shows how aspects can address all these requirements elegantly.

## The Problem: Security Boilerplate

Traditional authorization implementations mix security checks with business logic:

```rust
// Traditional approach - security mixed with business logic
pub fn delete_user(current_user: &User, user_id: u64) -> Result<(), String> {
    // Security check (repeated in every function)
    if !current_user.has_role("admin") {
        log_audit("DENIED", current_user, "delete_user");
        return Err("Access denied: admin role required");
    }

    // Audit log (repeated in every function)
    log_audit("ATTEMPT", current_user, "delete_user");

    // Actual business logic (buried)
    database::delete(user_id)?;

    // More audit logging
    log_audit("SUCCESS", current_user, "delete_user");
    Ok(())
}
```

**Problems:**

1. Security checks must be duplicated in every protected function
2. Easy to forget authorization for new features
3. Hard to change security policies globally
4. Business logic is obscured by security boilerplate
5. Testing business logic requires mocking security framework

## The Solution: Declarative Security with Aspects

With aspects, security becomes a declarative concern:

```rust
#[aspect(AuthorizationAspect::require_role("admin"))]
#[aspect(AuditAspect::default())]
fn delete_user(user_id: u64) -> Result<(), String> {
    // Just business logic - clean and focused!
    println!("  [SYSTEM] Deleting user {}", user_id);
    Ok(())
}
```

Security is declared via attributes, enforced automatically, and impossible to forget.

## Complete Implementation

### User and Role Model

First, let's define our security model:

```rust
use aspect_core::prelude::*;
use aspect_macros::aspect;
use std::any::Any;
use std::sync::RwLock;

/// Simple user representation
#[derive(Debug, Clone)]
struct User {
    username: String,
    roles: Vec<String>,
}

impl User {
    fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }
}
```

### Security Context Management

We need to track the current user making requests:

```rust
/// Thread-local current user context (simulated)
static CURRENT_USER: RwLock<Option<User>> = RwLock::new(None);

fn set_current_user(user: User) {
    *CURRENT_USER.write().unwrap() = Some(user);
}

fn get_current_user() -> Option<User> {
    CURRENT_USER.read().unwrap().clone()
}

fn clear_current_user() {
    *CURRENT_USER.write().unwrap() = None;
}
```

In production, you'd use proper thread-local storage or async context propagation.

### Authorization Aspect

Now let's implement the authorization aspect:

```rust
/// Authorization aspect that checks user roles before execution
struct AuthorizationAspect {
    required_roles: Vec<String>,
    require_all: bool, // true = all roles required, false = any role required
}

impl AuthorizationAspect {
    /// Require a single specific role
    fn require_role(role: &str) -> Self {
        Self {
            required_roles: vec![role.to_string()],
            require_all: false,
        }
    }

    /// Require at least one of the specified roles
    fn require_any_role(roles: &[&str]) -> Self {
        Self {
            required_roles: roles.iter().map(|r| r.to_string()).collect(),
            require_all: false,
        }
    }

    /// Require all of the specified roles
    fn require_all_roles(roles: &[&str]) -> Self {
        Self {
            required_roles: roles.iter().map(|r| r.to_string()).collect(),
            require_all: true,
        }
    }

    /// Check if user meets authorization requirements
    fn check_authorization(&self, user: &User) -> Result<(), String> {
        if self.require_all {
            // All roles required
            for role in &self.required_roles {
                if !user.has_role(role) {
                    return Err(format!(
                        "Access denied: user '{}' missing required role '{}'",
                        user.username, role
                    ));
                }
            }
            Ok(())
        } else {
            // Any role is sufficient
            let role_refs: Vec<&str> = self.required_roles.iter().map(|s| s.as_str()).collect();
            if user.has_any_role(&role_refs) {
                Ok(())
            } else {
                Err(format!(
                    "Access denied: user '{}' needs one of: {}",
                    user.username,
                    self.required_roles.join(", ")
                ))
            }
        }
    }
}

impl Aspect for AuthorizationAspect {
    fn before(&self, ctx: &JoinPoint) {
        match get_current_user() {
            None => {
                panic!(
                    "Authorization failed for {}: No user logged in",
                    ctx.function_name
                );
            }
            Some(user) => {
                if let Err(msg) = self.check_authorization(&user) {
                    panic!("Authorization failed for {}: {}", ctx.function_name, msg);
                }
                println!(
                    "[AUTH] ✓ User '{}' authorized for {}",
                    user.username, ctx.function_name
                );
            }
        }
    }
}
```

**Key design decisions:**

1. **Fail-fast**: Authorization failures panic, preventing unauthorized execution
2. **Clear messages**: Users know exactly why access was denied
3. **Flexible policies**: Support single role, any-of, or all-of requirements
4. **Context-aware**: Uses JoinPoint to report which function failed authorization

### Audit Aspect

Security requires comprehensive audit logging:

```rust
/// Audit aspect that logs all security-sensitive operations
#[derive(Default)]
struct AuditAspect;

impl Aspect for AuditAspect {
    fn before(&self, ctx: &JoinPoint) {
        let user = get_current_user()
            .map(|u| u.username)
            .unwrap_or_else(|| "anonymous".to_string());

        println!(
            "[AUDIT] User '{}' accessing {} at {}:{}",
            user, ctx.function_name, ctx.location.file, ctx.location.line
        );
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        let user = get_current_user()
            .map(|u| u.username)
            .unwrap_or_else(|| "anonymous".to_string());

        println!("[AUDIT] User '{}' completed {}", user, ctx.function_name);
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        let user = get_current_user()
            .map(|u| u.username)
            .unwrap_or_else(|| "anonymous".to_string());

        println!(
            "[AUDIT] ⚠ User '{}' failed {}: {:?}",
            user, ctx.function_name, error
        );
    }
}
```

**Audit trails capture:**

- Who performed the operation (username)
- What operation was attempted (function name)
- When it occurred (implicit in log timestamps)
- Where in code (file and line number)
- Whether it succeeded or failed
- Error details if failed

This provides complete traceability for compliance and security investigations.

## Protected Operations

Now let's define some protected business operations:

### Admin-Only Operation

```rust
#[aspect(AuthorizationAspect::require_role("admin"))]
#[aspect(AuditAspect::default())]
fn delete_user(user_id: u64) -> Result<(), String> {
    println!("  [SYSTEM] Deleting user {}", user_id);
    Ok(())
}
```

**Behavior:**
- Only users with "admin" role can execute
- All attempts are audited (success or failure)
- Business logic is clean and focused

### Multi-Role Operation

```rust
#[aspect(AuthorizationAspect::require_any_role(&["admin", "moderator"]))]
#[aspect(AuditAspect::default())]
fn ban_user(user_id: u64, reason: &str) -> Result<(), String> {
    println!("  [SYSTEM] Banning user {} (reason: {})", user_id, reason);
    Ok(())
}
```

**Behavior:**
- Users with "admin" OR "moderator" role can execute
- Flexible policy without code changes
- Same clean business logic

### Regular User Operation

```rust
#[aspect(AuthorizationAspect::require_role("user"))]
#[aspect(AuditAspect::default())]
fn view_profile(user_id: u64) -> Result<String, String> {
    println!("  [SYSTEM] Fetching profile for user {}", user_id);
    Ok(format!("Profile data for user {}", user_id))
}
```

**Behavior:**
- Any authenticated user can execute
- Still audited for security monitoring
- Clear separation between auth and logic

### Public Operation

```rust
#[aspect(AuditAspect::default())]
fn public_endpoint() -> String {
    println!("  [SYSTEM] Public endpoint accessed");
    "Public data".to_string()
}
```

**Behavior:**
- No authorization required
- Still audited to track usage
- Demonstrates selective aspect application

## Demonstration

Let's see the security system in action:

```rust
fn main() {
    println!("=== Security & Authorization Aspect Example ===\n");

    // Example 1: Admin user accessing admin function
    println!("1. Admin user deleting a user:");
    set_current_user(User {
        username: "admin_user".to_string(),
        roles: vec!["admin".to_string(), "user".to_string()],
    });

    match delete_user(42) {
        Ok(_) => println!("   ✓ Operation succeeded\n"),
        Err(e) => println!("   ✗ Operation failed: {}\n", e),
    }

    // Example 2: Moderator banning a user
    println!("2. Moderator banning a user:");
    set_current_user(User {
        username: "mod_user".to_string(),
        roles: vec!["moderator".to_string(), "user".to_string()],
    });

    match ban_user(99, "spam") {
        Ok(_) => println!("   ✓ Operation succeeded\n"),
        Err(e) => println!("   ✗ Operation failed: {}\n", e),
    }

    // Example 3: Regular user viewing profile
    println!("3. Regular user viewing profile:");
    set_current_user(User {
        username: "regular_user".to_string(),
        roles: vec!["user".to_string()],
    });

    match view_profile(42) {
        Ok(data) => println!("   ✓ Got: {}\n", data),
        Err(e) => println!("   ✗ Failed: {}\n", e),
    }

    // Example 4: Public endpoint (no auth required)
    println!("4. Public endpoint (no authorization):");
    let result = public_endpoint();
    println!("   ✓ Got: {}\n", result);

    // Example 5: Unauthorized access attempt
    println!("5. Regular user trying to delete (will panic):");
    println!("   Attempting unauthorized operation...");

    let result = std::panic::catch_unwind(|| {
        delete_user(123)
    });

    match result {
        Ok(_) => println!("   ✗ Unexpected success!"),
        Err(_) => println!("   ✓ Access denied as expected (caught panic)\n"),
    }

    // Example 6: No user logged in
    println!("6. No user logged in (will panic):");
    clear_current_user();

    let result = std::panic::catch_unwind(|| {
        view_profile(42)
    });

    match result {
        Ok(_) => println!("   ✗ Unexpected success!"),
        Err(_) => println!("   ✓ Denied as expected (caught panic)\n"),
    }

    println!("=== Demo Complete ===");
    println!("\nKey Takeaways:");
    println!("✓ Authorization logic separated from business code");
    println!("✓ Role-based access control enforced declaratively");
    println!("✓ Audit logging automatic for all protected functions");
    println!("✓ Multiple aspects compose cleanly (auth + audit)");
    println!("✓ Security policies centralized and reusable");
}
```

## Running the Example

```bash
cd aspect-rs/aspect-examples
cargo run --example security
```

**Expected Output:**

```
=== Security & Authorization Aspect Example ===

1. Admin user deleting a user:
[AUDIT] User 'admin_user' accessing delete_user at src/security.rs:161
[AUTH] ✓ User 'admin_user' authorized for delete_user
  [SYSTEM] Deleting user 42
[AUDIT] User 'admin_user' completed delete_user
   ✓ Operation succeeded

2. Moderator banning a user:
[AUDIT] User 'mod_user' accessing ban_user at src/security.rs:168
[AUTH] ✓ User 'mod_user' authorized for ban_user
  [SYSTEM] Banning user 99 (reason: spam)
[AUDIT] User 'mod_user' completed ban_user
   ✓ Operation succeeded

3. Regular user viewing profile:
[AUDIT] User 'regular_user' accessing view_profile at src/security.rs:175
[AUTH] ✓ User 'regular_user' authorized for view_profile
  [SYSTEM] Fetching profile for user 42
[AUDIT] User 'regular_user' completed view_profile
   ✓ Got: Profile data for user 42

4. Public endpoint (no authorization):
[AUDIT] User 'regular_user' accessing public_endpoint at src/security.rs:181
  [SYSTEM] Public endpoint accessed
[AUDIT] User 'regular_user' completed public_endpoint
   ✓ Got: Public data

5. Regular user trying to delete (will panic):
   Attempting unauthorized operation...
[AUDIT] User 'regular_user' accessing delete_user at src/security.rs:161
   ✓ Access denied as expected (caught panic)

6. No user logged in (will panic):
[AUDIT] User 'anonymous' accessing view_profile at src/security.rs:175
   ✓ Denied as expected (caught panic)

=== Demo Complete ===

Key Takeaways:
✓ Authorization logic separated from business code
✓ Role-based access control enforced declaratively
✓ Audit logging automatic for all protected functions
✓ Multiple aspects compose cleanly (auth + audit)
✓ Security policies centralized and reusable
```

## Advanced Patterns

### Fine-Grained Permissions

```rust
struct PermissionAspect {
    required_permission: String,
}

impl PermissionAspect {
    fn require_permission(perm: &str) -> Self {
        Self {
            required_permission: perm.to_string(),
        }
    }
}

impl Aspect for PermissionAspect {
    fn before(&self, ctx: &JoinPoint) {
        let user = get_current_user().expect("No user context");
        if !user.has_permission(&self.required_permission) {
            panic!("Missing permission: {}", self.required_permission);
        }
    }
}

#[aspect(PermissionAspect::require_permission("users.delete"))]
fn delete_user(user_id: u64) -> Result<(), String> {
    // Business logic
}
```

### Resource-Level Authorization

```rust
struct ResourceOwnerAspect;

impl Aspect for ResourceOwnerAspect {
    fn before(&self, ctx: &JoinPoint) {
        // Check if current user owns the resource
        let user = get_current_user().expect("No user");
        let resource_id = extract_resource_id(ctx);

        if !user.owns_resource(resource_id) {
            panic!("Access denied: not resource owner");
        }
    }
}

#[aspect(ResourceOwnerAspect)]
fn edit_profile(user_id: u64, data: ProfileData) -> Result<(), String> {
    // Only the profile owner can edit
}
```

### Time-Based Access Control

```rust
struct BusinessHoursAspect;

impl Aspect for BusinessHoursAspect {
    fn before(&self, ctx: &JoinPoint) {
        let hour = chrono::Local::now().hour();
        if hour < 9 || hour >= 17 {
            panic!("Operation {} not allowed outside business hours", ctx.function_name);
        }
    }
}

#[aspect(BusinessHoursAspect)]
#[aspect(AuthorizationAspect::require_role("admin"))]
fn financial_transaction(amount: f64) -> Result<(), String> {
    // Restricted to business hours
}
```

### Rate Limiting by User

```rust
struct UserRateLimitAspect {
    max_requests: usize,
    window: Duration,
    tracker: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl Aspect for UserRateLimitAspect {
    fn before(&self, ctx: &JoinPoint) {
        let user = get_current_user().expect("No user");
        let mut tracker = self.tracker.lock().unwrap();
        let requests = tracker.entry(user.username.clone()).or_insert_with(VecDeque::new);

        // Remove old requests outside window
        let cutoff = Instant::now() - self.window;
        while requests.front().map_or(false, |&t| t < cutoff) {
            requests.pop_front();
        }

        if requests.len() >= self.max_requests {
            panic!("Rate limit exceeded for user {}", user.username);
        }

        requests.push_back(Instant::now());
    }
}
```

## Integration with Authentication Systems

### JWT Token Validation

```rust
struct JwtValidationAspect {
    secret: Vec<u8>,
}

impl Aspect for JwtValidationAspect {
    fn before(&self, ctx: &JoinPoint) {
        let token = get_auth_header().expect("No auth header");
        let claims = validate_jwt(&token, &self.secret)
            .expect("Invalid token");

        set_current_user(User::from_claims(claims));
    }
}

#[aspect(JwtValidationAspect::new(secret))]
#[aspect(AuthorizationAspect::require_role("user"))]
fn protected_endpoint() -> String {
    "Protected data".to_string()
}
```

### OAuth2 Integration

```rust
struct OAuth2Aspect {
    required_scopes: Vec<String>,
}

impl Aspect for OAuth2Aspect {
    fn before(&self, ctx: &JoinPoint) {
        let token = get_bearer_token().expect("No token");
        let token_info = introspect_token(&token)
            .expect("Token introspection failed");

        if !has_required_scopes(&token_info, &self.required_scopes) {
            panic!("Insufficient scopes");
        }

        set_current_user(User::from_token_info(token_info));
    }
}
```

## Testing Security Aspects

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_can_delete() {
        set_current_user(User {
            username: "admin".to_string(),
            roles: vec!["admin".to_string()],
        });

        let result = delete_user(123);
        assert!(result.is_ok());
    }

    #[test]
    #[should_panic(expected = "Access denied")]
    fn test_user_cannot_delete() {
        set_current_user(User {
            username: "user".to_string(),
            roles: vec!["user".to_string()],
        });

        delete_user(123).unwrap(); // Should panic
    }

    #[test]
    fn test_moderator_can_ban() {
        set_current_user(User {
            username: "mod".to_string(),
            roles: vec!["moderator".to_string()],
        });

        let result = ban_user(456, "spam");
        assert!(result.is_ok());
    }
}
```

## Performance Considerations

Authorization aspects add minimal overhead:

```
Authorization check: ~1-5µs (role lookup + comparison)
Audit logging: ~10-50µs (formatting + I/O)
Total overhead: <100µs per request

For typical API requests (10-100ms), security overhead is <0.1%
```

**Optimization tips:**

1. Cache user permissions in memory
2. Batch audit logs (don't write per request)
3. Use async I/O for audit writes
4. Pre-compile role checks at compile time (Phase 3)

## Production Deployment

### Centralized Policy Management

```rust
// policy_config.rs
pub struct SecurityPolicy {
    pub role_permissions: HashMap<String, Vec<String>>,
    pub protected_operations: HashMap<String, Vec<String>>,
}

impl SecurityPolicy {
    pub fn from_file(path: &str) -> Self {
        // Load from YAML/JSON configuration
    }
}

// Use in aspects
struct ConfigurableAuthAspect {
    policy: Arc<SecurityPolicy>,
}
```

### Monitoring and Alerting

```rust
struct SecurityMonitoringAspect {
    alerting: Arc<dyn AlertingService>,
}

impl Aspect for SecurityMonitoringAspect {
    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        if is_authorization_error(error) {
            self.alerting.send_alert(Alert {
                severity: Severity::High,
                message: format!("Authorization failure in {}", ctx.function_name),
                user: get_current_user(),
                timestamp: Instant::now(),
            });
        }
    }
}
```

## Key Takeaways

1. **Declarative Security**
   - Security policies defined with attributes
   - No security boilerplate in business logic
   - Centralized and consistent enforcement

2. **Comprehensive Auditing**
   - Automatic audit trails for all protected operations
   - Complete traceability: who, what, when, where, result
   - Compliance-ready logging

3. **Flexible Authorization**
   - Role-based access control (RBAC)
   - Permission-based access control
   - Resource-level authorization
   - Time-based restrictions
   - Custom policies

4. **Composability**
   - Authorization + Audit aspects work together
   - Can add monitoring, rate limiting, etc.
   - Each aspect is independent

5. **Maintainability**
   - Security policies in one place
   - Easy to update globally
   - Impossible to forget authorization
   - Clear audit trail for debugging

## Next Steps

- See [API Server Case Study](./api-server.md) for applying security to APIs
- See [Resilience Case Study](./resilience.md) for error handling patterns
- See [Chapter 10: Phase 3](../ch10-phase3/README.md) for automatic security weaving

## Source Code

Complete working example:
```
aspect-rs/aspect-examples/src/security.rs
```

Run with:
```bash
cargo run --example security
```

---

**Related Chapters:**
- [Chapter 5.3: Multiple Aspects](../ch05-usage/multiple-aspects.md)
- [Chapter 8.1: API Server](./api-server.md)
- [Chapter 9: Benchmarks](../ch09-benchmarks/README.md)
