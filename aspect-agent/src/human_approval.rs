//! Human-in-the-loop approval gate aspect.
//!
//! Addresses the approval/HITL scattering in ZeroClaw where the `approval/` module
//! holds only 1/17 files (5.9%) with approval vocabulary — the worst centralization
//! ratio of any identified concern.
//!
//! This aspect enforces a human approval gate before executing critical functions.
//! It mirrors ZeroClaw's `approval/mod.rs` approval workflow but applied declaratively.

use aspect_core::prelude::*;
use aspect_core::AspectError;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

/// A request for human approval.
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    /// The function being called.
    pub function_name: String,
    /// The module containing the function.
    pub module_path: String,
    /// Human-readable description of what the function does.
    pub description: String,
    /// Risk level of the operation.
    pub risk: RiskLevel,
}

/// Risk level of an operation requiring approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Response from a human approval gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalResponse {
    /// Approve this specific call.
    Approve,
    /// Deny this specific call.
    Deny,
    /// Approve all future calls from this session (adds to session allowlist).
    ApproveSession,
}

/// Channel through which approval requests are sent and responses received.
pub enum ApprovalChannel {
    /// Blocks on stdin for human input (suitable for CLI tools, mirrors ZeroClaw CLI mode).
    Stdio,
    /// Calls a closure synchronously (suitable for tests and async runtimes).
    Handler(Arc<dyn Fn(ApprovalRequest) -> ApprovalResponse + Send + Sync>),
    /// Auto-approve everything (useful for integration tests and CI).
    AutoApprove,
    /// Auto-deny everything.
    AutoDeny,
}

impl ApprovalChannel {
    fn request(&self, req: ApprovalRequest) -> ApprovalResponse {
        match self {
            Self::AutoApprove => ApprovalResponse::Approve,
            Self::AutoDeny => ApprovalResponse::Deny,
            Self::Handler(f) => f(req),
            Self::Stdio => {
                eprintln!(
                    "\n[HumanApproval] Approval required for '{}' (risk: {})",
                    req.function_name,
                    req.risk.label()
                );
                eprintln!("  Module: {}", req.module_path);
                eprintln!("  Description: {}", req.description);
                eprint!("  Allow? [y/N/session]: ");

                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    match input.trim().to_lowercase().as_str() {
                        "y" | "yes" => ApprovalResponse::Approve,
                        "s" | "session" => ApprovalResponse::ApproveSession,
                        _ => ApprovalResponse::Deny,
                    }
                } else {
                    ApprovalResponse::Deny
                }
            }
        }
    }
}

/// Metadata about a function that requires human approval.
#[derive(Debug, Clone)]
struct ApprovalTarget {
    description: String,
    risk: RiskLevel,
}

/// Aspect that gates execution of critical functions behind human approval.
///
/// Functions listed in `always_ask` require explicit human approval on every call.
/// Functions listed in `session_auto_approve` are pre-approved for this session.
///
/// # Example
///
/// ```rust,ignore
/// use aspect_agent::human_approval::{ApprovalChannel, HumanApprovalAspect, RiskLevel};
/// use aspect_macros::aspect;
///
/// // In tests: auto-approve
/// let gate = HumanApprovalAspect::new(ApprovalChannel::AutoApprove)
///     .require_approval("execute_shell", RiskLevel::High, "Execute shell command");
///
/// // In production: stdio prompt
/// let gate = HumanApprovalAspect::new(ApprovalChannel::Stdio)
///     .require_approval("delete_database", RiskLevel::Critical, "Drop all tables");
///
/// #[aspect(gate.clone())]
/// fn execute_shell(cmd: String) -> Result<String, String> {
///     // Will prompt for approval before running
///     std::process::Command::new("sh").arg("-c").arg(&cmd)
///         .output()
///         .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
///         .map_err(|e| e.to_string())
/// }
/// ```
#[derive(Clone)]
pub struct HumanApprovalAspect {
    channel: Arc<ApprovalChannel>,
    /// Functions (by name) that always require approval.
    approval_targets: Arc<std::collections::HashMap<String, ApprovalTarget>>,
    /// Functions approved for the duration of this session.
    session_allowlist: Arc<Mutex<HashSet<String>>>,
}

impl HumanApprovalAspect {
    /// Create a new aspect using the given approval channel.
    pub fn new(channel: ApprovalChannel) -> Self {
        Self {
            channel: Arc::new(channel),
            approval_targets: Arc::new(std::collections::HashMap::new()),
            session_allowlist: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Register a function as requiring human approval.
    pub fn require_approval(
        self,
        function_name: impl Into<String>,
        risk: RiskLevel,
        description: impl Into<String>,
    ) -> Self {
        let mut targets = (*self.approval_targets).clone();
        targets.insert(
            function_name.into(),
            ApprovalTarget {
                description: description.into(),
                risk,
            },
        );
        Self {
            approval_targets: Arc::new(targets),
            ..self
        }
    }

    /// Pre-approve a function for this session (bypasses future approval prompts).
    pub fn pre_approve(&self, function_name: impl Into<String>) {
        self.session_allowlist.lock().insert(function_name.into());
    }
}

impl Aspect for HumanApprovalAspect {
    fn before(&self, ctx: &JoinPoint) {
        let fname = ctx.function_name;

        // Check if session-approved
        if self.session_allowlist.lock().contains(fname) {
            return;
        }

        // Check if this function requires approval
        if let Some(target) = self.approval_targets.get(fname) {
            let req = ApprovalRequest {
                function_name: fname.to_string(),
                module_path: ctx.module_path.to_string(),
                description: target.description.clone(),
                risk: target.risk,
            };

            match self.channel.request(req) {
                ApprovalResponse::Approve => {
                    // One-time approval — proceed
                }
                ApprovalResponse::ApproveSession => {
                    // Add to session allowlist and proceed
                    self.session_allowlist.lock().insert(fname.to_string());
                }
                ApprovalResponse::Deny => {
                    panic!(
                        "[HumanApproval] Execution of '{}' denied by human operator",
                        fname
                    );
                }
            }
        }
    }

    fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
        eprintln!(
            "[HumanApproval] Approved function '{}' failed: {}",
            ctx.function_name, error
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aspect_core::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn make_jp(name: &'static str) -> JoinPoint {
        JoinPoint::new(name, "test", Location { file: "f.rs", line: 1 })
    }

    #[test]
    fn test_auto_approve_passes() {
        let gate = HumanApprovalAspect::new(ApprovalChannel::AutoApprove)
            .require_approval("dangerous_fn", RiskLevel::High, "Does something risky");
        let jp = make_jp("dangerous_fn");
        // Should not panic
        gate.before(&jp);
    }

    #[test]
    #[should_panic(expected = "denied by human operator")]
    fn test_auto_deny_blocks() {
        let gate = HumanApprovalAspect::new(ApprovalChannel::AutoDeny)
            .require_approval("dangerous_fn", RiskLevel::High, "Does something risky");
        let jp = make_jp("dangerous_fn");
        gate.before(&jp);
    }

    #[test]
    fn test_non_registered_function_passes_without_prompt() {
        // Functions not in approval_targets pass without any check
        let gate = HumanApprovalAspect::new(ApprovalChannel::AutoDeny);
        let jp = make_jp("safe_fn");
        // Should not panic even with AutoDeny channel
        gate.before(&jp);
    }

    #[test]
    fn test_handler_channel_called() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let cc = call_count.clone();
        let gate = HumanApprovalAspect::new(ApprovalChannel::Handler(Arc::new(move |_req| {
            cc.fetch_add(1, Ordering::SeqCst);
            ApprovalResponse::Approve
        })))
        .require_approval("fn_a", RiskLevel::Medium, "Test function");

        let jp = make_jp("fn_a");
        gate.before(&jp);
        gate.before(&jp); // second call

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_session_approve_skips_second_prompt() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let cc = call_count.clone();
        let gate = HumanApprovalAspect::new(ApprovalChannel::Handler(Arc::new(move |_req| {
            cc.fetch_add(1, Ordering::SeqCst);
            ApprovalResponse::ApproveSession
        })))
        .require_approval("fn_b", RiskLevel::Low, "Session test");

        let jp = make_jp("fn_b");
        gate.before(&jp); // first call → ApproveSession
        gate.before(&jp); // second call → should use session allowlist, NOT call handler

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_pre_approve_bypasses_channel() {
        let gate = HumanApprovalAspect::new(ApprovalChannel::AutoDeny)
            .require_approval("fn_c", RiskLevel::Critical, "Pre-approved");

        gate.pre_approve("fn_c");

        let jp = make_jp("fn_c");
        // Should not panic despite AutoDeny channel
        gate.before(&jp);
    }

    #[test]
    fn test_aspect_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HumanApprovalAspect>();
    }

    #[test]
    fn test_multiple_functions_registered() {
        let gate = HumanApprovalAspect::new(ApprovalChannel::AutoApprove)
            .require_approval("fn_critical", RiskLevel::Critical, "Critical op")
            .require_approval("fn_high", RiskLevel::High, "High risk op");

        // Both should pass with AutoApprove
        gate.before(&make_jp("fn_critical"));
        gate.before(&make_jp("fn_high"));
        // Unregistered function also passes
        gate.before(&make_jp("fn_safe"));
    }
}
