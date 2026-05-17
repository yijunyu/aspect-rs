//! Authorization aspects.
//!
//! This module provides two complementary authorization aspects:
//!
//! - [`AuthorizationAspect`] — role-based access control (RBAC). Given the
//!   set of roles the caller holds, admit or deny based on a required-role
//!   policy. The classical AspectJ / Spring Security shape.
//!
//! - [`AllowlistAspect`] — identity-based allowlisting. Given an identity
//!   string (user ID, phone number, email, public key, etc.), admit if it
//!   matches a configured set, with optional wildcard and normalization
//!   hooks. This is the shape that recurs across messaging-channel
//!   integrations: each channel has its own per-channel allowlist of who is
//!   permitted to talk to the bot.

use aspect_core::{Aspect, AspectError, JoinPoint, ProceedingJoinPoint};
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

/// Role-based access control aspect.
///
/// Enforces authorization checks before function execution based on
/// required roles or permissions.
///
/// # Example
///
/// ```rust,ignore
/// use aspect_std::AuthorizationAspect;
/// use aspect_macros::aspect;
///
/// // Require "admin" role
/// let auth = AuthorizationAspect::require_role("admin", || {
///     get_current_user_roles()
/// });
///
/// #[aspect(auth)]
/// fn delete_user(user_id: u64) -> Result<(), String> {
///     // Only admins can delete users
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct AuthorizationAspect {
    required_roles: Arc<HashSet<String>>,
    role_provider: Arc<dyn Fn() -> HashSet<String> + Send + Sync>,
    mode: AuthMode,
}

/// Authorization mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthMode {
    /// Require ALL specified roles.
    RequireAll,
    /// Require ANY of the specified roles.
    RequireAny,
}

impl AuthorizationAspect {
    /// Create an authorization aspect that requires a specific role.
    ///
    /// # Arguments
    /// * `role` - The required role
    /// * `role_provider` - Function that returns the current user's roles
    ///
    /// # Example
    /// ```rust,ignore
    /// let auth = AuthorizationAspect::require_role("admin", || {
    ///     vec!["admin".to_string()].into_iter().collect()
    /// });
    /// ```
    pub fn require_role<F>(role: &str, role_provider: F) -> Self
    where
        F: Fn() -> HashSet<String> + Send + Sync + 'static,
    {
        let mut roles = HashSet::new();
        roles.insert(role.to_string());

        Self {
            required_roles: Arc::new(roles),
            role_provider: Arc::new(role_provider),
            mode: AuthMode::RequireAll,
        }
    }

    /// Create an authorization aspect that requires multiple roles.
    ///
    /// # Arguments
    /// * `roles` - The required roles
    /// * `role_provider` - Function that returns the current user's roles
    /// * `mode` - Whether to require ALL or ANY of the roles
    ///
    /// # Example
    /// ```rust,ignore
    /// let auth = AuthorizationAspect::require_roles(
    ///     &["admin", "moderator"],
    ///     || get_current_roles(),
    ///     AuthMode::RequireAny
    /// );
    /// ```
    pub fn require_roles<F>(roles: &[&str], role_provider: F, mode: AuthMode) -> Self
    where
        F: Fn() -> HashSet<String> + Send + Sync + 'static,
    {
        let role_set: HashSet<String> = roles.iter().map(|r| r.to_string()).collect();

        Self {
            required_roles: Arc::new(role_set),
            role_provider: Arc::new(role_provider),
            mode,
        }
    }

    /// Check if the current user is authorized.
    fn check_authorization(&self) -> Result<(), String> {
        let current_roles = (self.role_provider)();

        let authorized = match self.mode {
            AuthMode::RequireAll => {
                // User must have ALL required roles
                self.required_roles.iter().all(|r| current_roles.contains(r))
            }
            AuthMode::RequireAny => {
                // User must have ANY of the required roles
                self.required_roles.iter().any(|r| current_roles.contains(r))
            }
        };

        if authorized {
            Ok(())
        } else {
            let required: Vec<_> = self.required_roles.iter().cloned().collect();
            let mode_str = match self.mode {
                AuthMode::RequireAll => "all",
                AuthMode::RequireAny => "any",
            };
            Err(format!(
                "Access denied: requires {} of roles {:?}",
                mode_str, required
            ))
        }
    }
}

impl Aspect for AuthorizationAspect {
    fn before(&self, ctx: &JoinPoint) {
        if let Err(msg) = self.check_authorization() {
            panic!("Authorization failed for {}: {}", ctx.function_name, msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_roles(roles: Vec<&str>) -> HashSet<String> {
        roles.into_iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_require_role_success() {
        let auth = AuthorizationAspect::require_role("admin", || mock_roles(vec!["admin"]));

        assert!(auth.check_authorization().is_ok());
    }

    #[test]
    fn test_require_role_failure() {
        let auth = AuthorizationAspect::require_role("admin", || mock_roles(vec!["user"]));

        assert!(auth.check_authorization().is_err());
    }

    #[test]
    fn test_require_all_success() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["admin", "moderator", "user"]),
            AuthMode::RequireAll,
        );

        assert!(auth.check_authorization().is_ok());
    }

    #[test]
    fn test_require_all_failure() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["admin"]),
            AuthMode::RequireAll,
        );

        assert!(auth.check_authorization().is_err());
    }

    #[test]
    fn test_require_any_success() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["moderator"]),
            AuthMode::RequireAny,
        );

        assert!(auth.check_authorization().is_ok());
    }

    #[test]
    fn test_require_any_failure() {
        let auth = AuthorizationAspect::require_roles(
            &["admin", "moderator"],
            || mock_roles(vec!["user"]),
            AuthMode::RequireAny,
        );

        assert!(auth.check_authorization().is_err());
    }

    #[test]
    fn test_empty_roles() {
        let auth = AuthorizationAspect::require_role("admin", || mock_roles(vec![]));

        assert!(auth.check_authorization().is_err());
    }

    #[test]
    fn test_multiple_roles_user() {
        let auth = AuthorizationAspect::require_role("admin", || {
            mock_roles(vec!["user", "moderator", "admin"])
        });

        assert!(auth.check_authorization().is_ok());
    }
}
// ──────────────────────────────────────────────────────────────────────────
// AllowlistAspect
// ──────────────────────────────────────────────────────────────────────────

/// Identity-based allowlist aspect.
///
/// Admits a call when the supplied identity string is present in a
/// configured allowlist, optionally honoring a wildcard entry (`"*"`) and an
/// optional normalization hook for identity comparison.
///
/// This shape recurs across messaging-channel integrations in real
/// codebases: each channel maintains its own list of who is permitted to
/// talk to the bot. The classical role-based [`AuthorizationAspect`] does
/// not fit because there are no roles — there is a set of literal identity
/// strings and a yes/no decision.
///
/// # Example
///
/// ```rust
/// use aspect_std::AllowlistAspect;
///
/// let policy = AllowlistAspect::new(["user_a".to_string(), "user_b".to_string()]);
/// assert!(policy.is_allowed("user_a"));
/// assert!(!policy.is_allowed("intruder"));
///
/// // Wildcard.
/// let open = AllowlistAspect::new(["*".to_string()]);
/// assert!(open.is_allowed("anyone"));
///
/// // Empty list = deny everyone.
/// let closed = AllowlistAspect::new(Vec::<String>::new());
/// assert!(!closed.is_allowed("user_a"));
/// ```
///
/// # Normalization
///
/// Some identity spaces require canonicalization before comparison: phone
/// numbers (E.164), emails (case-insensitive), Telegram usernames
/// (`@`-stripped), Matrix MXIDs (case-insensitive). Pass a normalization
/// function via [`AllowlistAspect::with_normalizer`].
///
/// # Mutability
///
/// The allowlist is stored behind an `RwLock` and can be amended at runtime
/// via [`AllowlistAspect::add`] / [`AllowlistAspect::set`]. This matches
/// runtime-pairing flows where new identities are added without restart.
#[derive(Clone)]
pub struct AllowlistAspect {
    inner: Arc<AllowlistInner>,
}

struct AllowlistInner {
    entries: RwLock<Vec<String>>,
    normalizer: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
    matcher: Option<Box<dyn Fn(&str, &str) -> bool + Send + Sync>>,
}

impl AllowlistAspect {
    /// Build an allowlist aspect from an initial set of entries.
    ///
    /// The wildcard entry `"*"` admits everyone. An empty list denies
    /// everyone (fail-closed). Identity comparison is byte-equal by
    /// default; see [`AllowlistAspect::with_normalizer`] for case-
    /// insensitive or canonicalized comparison.
    pub fn new<I, S>(entries: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            inner: Arc::new(AllowlistInner {
                entries: RwLock::new(entries.into_iter().map(Into::into).collect()),
                normalizer: None,
                matcher: None,
            }),
        }
    }

    /// Attach a normalization function applied to both the allowlist entries
    /// and the queried identity before comparison.
    ///
    /// Consumes and returns `self`. The normalizer must be deterministic and
    /// referentially transparent: calling it twice on the same input must
    /// produce identical output.
    ///
    /// Mutually exclusive with [`AllowlistAspect::with_matcher`]: if a
    /// matcher is installed, the normalizer is bypassed because the
    /// matcher owns the entry-vs-identity comparison end-to-end.
    pub fn with_normalizer<F>(mut self, normalizer: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        // We are the sole owner immediately after `new`, so `make_mut`
        // returns the unique pointee. If a normalizer is added later
        // (after clones exist), `make_mut` clones the inner — at which
        // point the normalizer field on the old `Inner` is dropped
        // (because the trait-object closure is not `Clone`); see the
        // `Clone for AllowlistInner` impl below for the documented
        // behavior.
        let inner = Arc::make_mut(&mut self.inner);
        inner.normalizer = Some(Box::new(normalizer));
        self
    }

    /// Attach a custom match predicate `(entry, identity) -> bool` that
    /// replaces the default byte-equal (or normalized-equal) comparison.
    ///
    /// Consumes and returns `self`. The wildcard `"*"` and empty-list
    /// short-circuits still apply *before* the matcher runs — the matcher
    /// is only consulted for non-`"*"` entries against a non-empty list.
    /// The matcher must be referentially transparent.
    ///
    /// Use this when entries and identities are not directly comparable
    /// — e.g. allowlist entries that describe a *class* of identity
    /// (`"@example.com"` matches any user at that domain) rather than a
    /// single literal one. When a matcher is set the normalizer is
    /// bypassed; the matcher owns the comparison.
    pub fn with_matcher<F>(mut self, matcher: F) -> Self
    where
        F: Fn(&str, &str) -> bool + Send + Sync + 'static,
    {
        let inner = Arc::make_mut(&mut self.inner);
        inner.matcher = Some(Box::new(matcher));
        self
    }

    /// Returns `true` if `identity` is admitted by the allowlist.
    ///
    /// Semantics: empty list ⇒ deny; `"*"` present ⇒ admit; otherwise
    /// admit iff the (possibly normalized) identity equals some
    /// (possibly normalized) entry.
    pub fn is_allowed(&self, identity: &str) -> bool {
        let entries = self.inner.entries.read();
        if entries.is_empty() {
            return false;
        }
        if let Some(matcher) = self.inner.matcher.as_ref() {
            for entry in entries.iter() {
                if entry == "*" {
                    return true;
                }
                if matcher(entry, identity) {
                    return true;
                }
            }
            return false;
        }
        let norm = self.inner.normalizer.as_ref();
        let needle = norm.map(|f| f(identity));
        for entry in entries.iter() {
            if entry == "*" {
                return true;
            }
            match (&needle, norm) {
                (Some(needle), Some(f)) => {
                    if f(entry) == *needle {
                        return true;
                    }
                }
                _ => {
                    if entry == identity {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Append `identity` to the allowlist if absent. No-op when `identity`
    /// is already present (byte-equal, *not* normalized).
    pub fn add(&self, identity: impl Into<String>) {
        let identity = identity.into();
        let mut entries = self.inner.entries.write();
        if !entries.iter().any(|e| e == &identity) {
            entries.push(identity);
        }
    }

    /// Replace the entire allowlist atomically.
    pub fn set<I, S>(&self, entries: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        *self.inner.entries.write() = entries.into_iter().map(Into::into).collect();
    }

    /// Snapshot the current allowlist as an owned `Vec`. Order-preserving
    /// relative to insertion / construction.
    pub fn snapshot(&self) -> Vec<String> {
        self.inner.entries.read().clone()
    }

    /// Number of entries currently in the list (including the wildcard
    /// entry if present).
    pub fn len(&self) -> usize {
        self.inner.entries.read().len()
    }

    /// Returns true when the allowlist is empty (i.e. denies everyone).
    pub fn is_empty(&self) -> bool {
        self.inner.entries.read().is_empty()
    }
}

// Workaround: `Arc::make_mut` needs `Clone` on the pointee. We never call
// it in practice — `with_normalizer` is only meant to be used immediately
// after `new`, before any clones exist — so we never actually trigger the
// clone path. Implement `Clone` defensively for the unlikely case.
impl Clone for AllowlistInner {
    fn clone(&self) -> Self {
        Self {
            entries: RwLock::new(self.entries.read().clone()),
            // A `Fn` trait object is not `Clone`. If a caller insists on
            // cloning an `AllowlistAspect` that already has a normalizer
            // or matcher installed after cloning, those fields are
            // dropped. This is documented; the canonical use is to call
            // `with_normalizer` / `with_matcher` immediately after
            // `new`, before any clones exist.
            normalizer: None,
            matcher: None,
        }
    }
}

/// Per-call context bound to the most recent guarded join point. Set by
/// the host before `proceed()` is invoked; cleared by `release`.
#[derive(Clone, Debug)]
pub struct AllowlistCall {
    /// The identity being checked at this join point.
    pub identity: String,
}

impl Aspect for AllowlistAspect {
    fn before(&self, _ctx: &JoinPoint) {
        // No-op. Denial happens in `around`, where we can short-circuit.
    }

    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        // The around path is provided so the aspect can be applied via
        // `#[aspect(...)]` weaving when the migration is ready to use that
        // path. The current zeroclaw migration uses the direct
        // `is_allowed(...)` query API instead and does not go through
        // `around`; we keep the implementation honest by returning the
        // proceed result unchanged here. A future macro-driven migration
        // would extend `ProceedingJoinPoint` to carry the identity to
        // check, at which point this path would call `is_allowed`.
        pjp.proceed()
    }
}

#[cfg(test)]
mod allowlist_tests {
    use super::*;

    #[test]
    fn empty_list_denies_everyone() {
        let policy = AllowlistAspect::new(Vec::<String>::new());
        assert!(!policy.is_allowed("alice"));
        assert!(!policy.is_allowed(""));
    }

    #[test]
    fn wildcard_admits_everyone() {
        let policy = AllowlistAspect::new(["*".to_string()]);
        assert!(policy.is_allowed("alice"));
        assert!(policy.is_allowed("anyone"));
        assert!(policy.is_allowed(""));
    }

    #[test]
    fn specific_identity() {
        let policy = AllowlistAspect::new(["alice".to_string(), "bob".to_string()]);
        assert!(policy.is_allowed("alice"));
        assert!(policy.is_allowed("bob"));
        assert!(!policy.is_allowed("charlie"));
    }

    #[test]
    fn wildcard_mixed_with_specific() {
        // Matches existing zeroclaw behavior: wildcard wins regardless of
        // position.
        let policy = AllowlistAspect::new(["alice".to_string(), "*".to_string()]);
        assert!(policy.is_allowed("alice"));
        assert!(policy.is_allowed("charlie"));
    }

    #[test]
    fn normalizer_case_insensitive() {
        let policy = AllowlistAspect::new(["Alice@Example.com".to_string()])
            .with_normalizer(|s| s.to_ascii_lowercase());
        assert!(policy.is_allowed("alice@example.com"));
        assert!(policy.is_allowed("ALICE@EXAMPLE.COM"));
        assert!(!policy.is_allowed("bob@example.com"));
    }

    #[test]
    fn normalizer_phone_strip() {
        // Mirrors how zeroclaw's whatsapp_web normalizes E.164: strip
        // everything but digits and the leading '+'.
        let policy =
            AllowlistAspect::new(["+1-555-0100".to_string()]).with_normalizer(|s| {
                let mut out = String::new();
                let mut chars = s.chars();
                if let Some('+') = chars.clone().next() {
                    out.push('+');
                    chars.next();
                }
                for c in chars {
                    if c.is_ascii_digit() {
                        out.push(c);
                    }
                }
                out
            });
        assert!(policy.is_allowed("+15550100"));
        assert!(policy.is_allowed("+1 555 0100"));
        assert!(!policy.is_allowed("+15550101"));
    }

    #[test]
    fn add_appends_unique() {
        let policy = AllowlistAspect::new(["alice".to_string()]);
        policy.add("bob");
        policy.add("bob"); // duplicate ignored
        assert_eq!(policy.len(), 2);
        assert!(policy.is_allowed("bob"));
    }

    #[test]
    fn set_replaces_atomically() {
        let policy = AllowlistAspect::new(["alice".to_string()]);
        policy.set(["bob".to_string(), "carol".to_string()]);
        assert!(!policy.is_allowed("alice"));
        assert!(policy.is_allowed("bob"));
        assert!(policy.is_allowed("carol"));
        assert_eq!(policy.len(), 2);
    }

    #[test]
    fn snapshot_returns_owned_copy() {
        let policy = AllowlistAspect::new(["alice".to_string(), "bob".to_string()]);
        let snap = policy.snapshot();
        assert_eq!(snap, vec!["alice".to_string(), "bob".to_string()]);
        // Mutating the policy does not affect the snapshot.
        policy.add("carol");
        assert_eq!(snap.len(), 2);
    }

    #[test]
    fn clone_shares_underlying_list() {
        let policy_a = AllowlistAspect::new(["alice".to_string()]);
        let policy_b = policy_a.clone();
        policy_b.add("bob");
        // Both views see the addition — clones share `Arc<Inner>`.
        assert!(policy_a.is_allowed("bob"));
    }

    /// Parity test: matches the exact semantics of zeroclaw's existing
    /// archetype-A `is_user_allowed` shape (`allowed.iter().any(|u| u ==
    /// "*" || u == identity)`). The aspect must match this Boolean
    /// truth table for every (allowlist, identity) pair below, so the
    /// migration is provably behavior-preserving.
    #[test]
    fn parity_with_zeroclaw_archetype_a() {
        let cases: Vec<(Vec<&str>, &str, bool)> = vec![
            (vec![], "alice", false),
            (vec![], "", false),
            (vec!["*"], "alice", true),
            (vec!["*"], "", true),
            (vec!["alice"], "alice", true),
            (vec!["alice"], "bob", false),
            (vec!["alice", "bob"], "bob", true),
            (vec!["alice", "*"], "anyone", true),
            (vec!["*", "alice"], "anyone", true),
            (vec!["alice"], "ALICE", false), // case-sensitive by default
            (vec!["alice"], "alice ", false), // whitespace-sensitive by default
        ];
        for (entries, identity, expected) in cases {
            let policy = AllowlistAspect::new(entries.iter().map(|s| s.to_string()));
            let baseline = entries.iter().any(|u| *u == "*" || *u == identity);
            assert_eq!(
                baseline, expected,
                "baseline semantics drifted for entries={entries:?} identity={identity:?}"
            );
            assert_eq!(
                policy.is_allowed(identity),
                expected,
                "AllowlistAspect mismatched zeroclaw archetype-A for entries={entries:?} identity={identity:?}"
            );
        }
    }

    /// The email/gmail_push shape: entries may be a full email
    /// ("alice@example.com"), a domain with `@` prefix
    /// ("@example.com"), or a bare domain ("example.com"). All
    /// comparisons are ASCII-case-insensitive.
    fn email_domain_or_full_match(entry: &str, email: &str) -> bool {
        let email_lower = email.to_ascii_lowercase();
        if let Some(domain) = entry.strip_prefix('@') {
            email_lower.ends_with(&format!("@{}", domain.to_ascii_lowercase()))
        } else if entry.contains('@') {
            entry.eq_ignore_ascii_case(email)
        } else {
            email_lower.ends_with(&format!("@{}", entry.to_ascii_lowercase()))
        }
    }

    #[test]
    fn matcher_email_domain_or_full() {
        let policy = AllowlistAspect::new([
            "alice@example.com".to_string(),
            "@allowed.com".to_string(),
            "bare.com".to_string(),
        ])
        .with_matcher(email_domain_or_full_match);
        assert!(policy.is_allowed("alice@example.com"));
        assert!(policy.is_allowed("ALICE@Example.COM")); // case-insensitive full match
        assert!(policy.is_allowed("anyone@allowed.com"));
        assert!(policy.is_allowed("anyone@bare.com"));
        assert!(!policy.is_allowed("bob@example.com")); // wrong local part, no domain rule
        assert!(!policy.is_allowed("anyone@other.com"));
    }

    #[test]
    fn matcher_respects_wildcard_and_empty_shortcircuits() {
        // Empty list still denies even with a permissive matcher.
        let empty =
            AllowlistAspect::new(Vec::<String>::new()).with_matcher(|_e, _i| true);
        assert!(!empty.is_allowed("anyone"));

        // Wildcard still admits without ever calling the matcher.
        let wild = AllowlistAspect::new(["*".to_string()])
            .with_matcher(|_e, _i| panic!("matcher must not run when '*' present"));
        assert!(wild.is_allowed("anyone"));
    }

    #[test]
    fn matcher_takes_precedence_over_normalizer() {
        // Normalizer would map both sides to lowercase, but the matcher
        // owns the comparison and chooses to be strict-equal. The
        // normalizer must be bypassed.
        let policy = AllowlistAspect::new(["Alice".to_string()])
            .with_normalizer(|s| s.to_ascii_lowercase())
            .with_matcher(|e, i| e == i);
        assert!(policy.is_allowed("Alice"));
        assert!(!policy.is_allowed("alice"));
    }
}
