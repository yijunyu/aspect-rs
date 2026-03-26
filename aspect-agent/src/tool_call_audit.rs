//! Tool call audit trail aspect.
//!
//! Provides a compliance-grade audit trail for every agent tool invocation.
//! Addresses the approval/HITL scattering in ZeroClaw where 17/192 files (8.9%)
//! contain approval vocabulary but only 1/17 files is inside the designated module.
//!
//! Every decorated function gets a timestamped audit entry recording who called it,
//! when, the outcome (success/failure), and how long it took.

use aspect_core::prelude::*;
use aspect_core::AspectError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Outcome of an audited tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    /// Call completed successfully.
    Success,
    /// Call failed with an error message.
    Failure(String),
    /// Call was blocked before execution (e.g., by another aspect).
    Blocked(String),
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unix timestamp (milliseconds) of when the call was initiated.
    pub timestamp_ms: u64,
    /// The name of the function that was called.
    pub function_name: String,
    /// The module path containing the function.
    pub module_path: String,
    /// The actor (e.g., user ID or session token) that initiated the call, if available.
    pub actor: Option<String>,
    /// Outcome of the call.
    pub outcome: AuditOutcome,
    /// Duration of the call in milliseconds (0 if unknown).
    pub duration_ms: u64,
}

/// Storage backend for audit entries.
pub trait AuditStorage: Send + Sync {
    /// Append a new audit entry.
    fn append(&self, entry: AuditEntry);
    /// Retrieve all stored entries.
    fn entries(&self) -> Vec<AuditEntry>;
    /// Number of entries stored.
    fn len(&self) -> usize;
    /// Whether the storage is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory audit storage (default). Not persistent across restarts.
#[derive(Default)]
pub struct InMemoryAuditStorage {
    entries: Mutex<Vec<AuditEntry>>,
}

impl AuditStorage for InMemoryAuditStorage {
    fn append(&self, entry: AuditEntry) {
        self.entries.lock().push(entry);
    }

    fn entries(&self) -> Vec<AuditEntry> {
        self.entries.lock().clone()
    }

    fn len(&self) -> usize {
        self.entries.lock().len()
    }
}

/// Aspect that records an audit entry for every function invocation.
///
/// # Example
///
/// ```rust,ignore
/// use aspect_agent::{ToolCallAuditAspect, InMemoryAuditStorage};
/// use aspect_macros::aspect;
/// use std::sync::Arc;
///
/// let storage = Arc::new(InMemoryAuditStorage::default());
/// let audit = ToolCallAuditAspect::new(storage.clone());
///
/// #[aspect(audit.clone())]
/// fn delete_file(path: String) -> Result<(), String> {
///     std::fs::remove_file(&path).map_err(|e| e.to_string())
/// }
///
/// // After calls, inspect the audit log:
/// for entry in storage.entries() {
///     println!("{}: {:?}", entry.function_name, entry.outcome);
/// }
/// ```
#[derive(Clone)]
pub struct ToolCallAuditAspect {
    storage: Arc<dyn AuditStorage>,
    /// Returns the current actor identifier (user ID, session, etc.).
    actor_provider: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    /// Per-call start timestamp stored in thread-local to compute duration.
    start_times: Arc<Mutex<std::collections::HashMap<String, SystemTime>>>,
}

impl ToolCallAuditAspect {
    /// Create a new audit aspect writing to the given storage.
    pub fn new(storage: Arc<dyn AuditStorage>) -> Self {
        Self {
            storage,
            actor_provider: Arc::new(|| None),
            start_times: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Set an actor provider closure that returns the current user/session identifier.
    pub fn with_actor_provider<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Option<String> + Send + Sync + 'static,
    {
        self.actor_provider = Arc::new(f);
        self
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64
    }

    fn key(ctx: &JoinPoint) -> String {
        format!("{}::{}", ctx.module_path, ctx.function_name)
    }
}

impl Aspect for ToolCallAuditAspect {
    fn before(&self, ctx: &JoinPoint) {
        let key = Self::key(ctx);
        self.start_times.lock().insert(key, SystemTime::now());
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        let key = Self::key(ctx);
        let duration_ms = self
            .start_times
            .lock()
            .remove(&key)
            .and_then(|start| start.elapsed().ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.storage.append(AuditEntry {
            timestamp_ms: Self::now_ms(),
            function_name: ctx.function_name.to_string(),
            module_path: ctx.module_path.to_string(),
            actor: (self.actor_provider)(),
            outcome: AuditOutcome::Success,
            duration_ms,
        });
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        let key = Self::key(ctx);
        let duration_ms = self
            .start_times
            .lock()
            .remove(&key)
            .and_then(|start| start.elapsed().ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.storage.append(AuditEntry {
            timestamp_ms: Self::now_ms(),
            function_name: ctx.function_name.to_string(),
            module_path: ctx.module_path.to_string(),
            actor: (self.actor_provider)(),
            outcome: AuditOutcome::Failure(error.to_string()),
            duration_ms,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aspect_core::prelude::*;

    fn make_storage() -> Arc<InMemoryAuditStorage> {
        Arc::new(InMemoryAuditStorage::default())
    }

    fn make_jp(name: &'static str) -> JoinPoint {
        JoinPoint::new(name, "test::module", Location { file: "f.rs", line: 1 })
    }

    #[test]
    fn test_after_records_success_entry() {
        let storage = make_storage();
        let audit = ToolCallAuditAspect::new(storage.clone());
        let jp = make_jp("delete_file");

        audit.before(&jp);
        audit.after(&jp, &());

        let entries = storage.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].function_name, "delete_file");
        assert!(matches!(entries[0].outcome, AuditOutcome::Success));
    }

    #[test]
    fn test_after_error_records_failure_entry() {
        let storage = make_storage();
        let audit = ToolCallAuditAspect::new(storage.clone());
        let jp = make_jp("shell_exec");

        audit.before(&jp);
        let err = AspectError::execution("permission denied");
        audit.after_error(&jp, &err);

        let entries = storage.entries();
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0].outcome, AuditOutcome::Failure(_)));
    }

    #[test]
    fn test_actor_provider_included_in_entry() {
        let storage = make_storage();
        let audit = ToolCallAuditAspect::new(storage.clone())
            .with_actor_provider(|| Some("user_42".into()));
        let jp = make_jp("read_secret");

        audit.before(&jp);
        audit.after(&jp, &());

        let entries = storage.entries();
        assert_eq!(entries[0].actor, Some("user_42".into()));
    }

    #[test]
    fn test_multiple_calls_recorded() {
        let storage = make_storage();
        let audit = ToolCallAuditAspect::new(storage.clone());

        for i in 0..5 {
            let jp = make_jp("fn_a");
            audit.before(&jp);
            let _ = i; // suppress warning
            audit.after(&jp, &());
        }

        assert_eq!(storage.len(), 5);
    }

    #[test]
    fn test_duration_is_recorded() {
        let storage = make_storage();
        let audit = ToolCallAuditAspect::new(storage.clone());
        let jp = make_jp("slow_fn");

        audit.before(&jp);
        std::thread::sleep(std::time::Duration::from_millis(10));
        audit.after(&jp, &());

        let entries = storage.entries();
        assert!(entries[0].duration_ms >= 1, "duration should be non-zero");
    }

    #[test]
    fn test_empty_storage() {
        let storage = make_storage();
        assert!(storage.is_empty());
    }

    #[test]
    fn test_aspect_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ToolCallAuditAspect>();
        assert_send_sync::<InMemoryAuditStorage>();
    }

    #[test]
    fn test_audit_entries_contain_module_path() {
        let storage = make_storage();
        let audit = ToolCallAuditAspect::new(storage.clone());
        let jp = make_jp("fn_b");
        audit.before(&jp);
        audit.after(&jp, &());
        assert_eq!(storage.entries()[0].module_path, "test::module");
    }
}
