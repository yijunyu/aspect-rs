//! Shared tool types mirroring ZeroClaw's `tools/traits.rs`.
//!
//! These types are compatible with ZeroClaw's actual `ToolResult` and `Tool` trait,
//! making the examples realistic without requiring a direct ZeroClaw dependency.

use serde::{Deserialize, Serialize};

/// Result of a tool execution — mirrors ZeroClaw `tools/traits.rs::ToolResult`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into(), error: None }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self { success: false, output: String::new(), error: Some(message.into()) }
    }
}

/// Simulated security policy for demonstrating aspect extraction.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub workspace_dir: std::path::PathBuf,
    pub max_actions_per_hour: usize,
    action_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl SecurityPolicy {
    pub fn new(workspace_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            workspace_dir: workspace_dir.into(),
            max_actions_per_hour: 20,
            action_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Mirrors ZeroClaw `security/policy.rs::is_rate_limited()`
    pub fn is_rate_limited(&self) -> bool {
        let count = self.action_count.load(std::sync::atomic::Ordering::Relaxed);
        count >= self.max_actions_per_hour
    }

    /// Mirrors ZeroClaw `security/policy.rs::record_action()`
    pub fn record_action(&self) -> bool {
        let count = self.action_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        count < self.max_actions_per_hour
    }

    /// Mirrors ZeroClaw `security/policy.rs::is_resolved_path_allowed()`
    pub fn is_path_allowed(&self, path: &std::path::Path) -> bool {
        let forbidden = ["/etc", "/root", "/proc", "/sys", "/dev", "/boot"];
        for prefix in &forbidden {
            if path.starts_with(prefix) {
                return false;
            }
        }
        // Check workspace containment
        let canonical_ws = self.workspace_dir.canonicalize()
            .unwrap_or_else(|_| self.workspace_dir.clone());
        path.starts_with(&canonical_ws)
    }
}
