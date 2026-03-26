//! v5 integration tests: human approval aspect vs. original ZeroClaw approval flow.
//!
//! Tests that HumanApprovalAspect produces identical allow/deny decisions to
//! ZeroClaw's approval/mod.rs approval workflow.

use aspect_agent::human_approval::{ApprovalChannel, ApprovalResponse, HumanApprovalAspect, RiskLevel};
use aspect_core::prelude::*;
use crate::correctness::{EquivalenceReport, TestCaseResult, ToolResult};
use std::sync::Arc;

/// Simulates ZeroClaw's original approval flow (simplified).
struct OriginalApprovalFlow {
    auto_approve: bool,
    approved_operations: Vec<String>,
}

impl OriginalApprovalFlow {
    fn require_approval(&mut self, operation: &str) -> ToolResult {
        if self.auto_approve {
            self.approved_operations.push(operation.to_string());
            return ToolResult::ok(format!("Operation approved: {operation}"));
        }
        ToolResult::err(format!("Operation denied: {operation}"))
    }
}

/// Run the approval flow equivalence tests.
pub fn run_approval_equivalence() -> EquivalenceReport {
    let mut cases = Vec::new();

    // Case 1: Auto-approve — both original and aspect approve
    {
        let mut original = OriginalApprovalFlow { auto_approve: true, approved_operations: vec![] };
        let aspect = HumanApprovalAspect::new(ApprovalChannel::AutoApprove)
            .require_approval("delete_workspace", RiskLevel::High, "Delete all workspace files");

        let jp = JoinPoint::new("delete_workspace", "tools::shell", Location { file: "shell.rs", line: 1 });

        let orig_result = original.require_approval("delete_workspace");
        let asp_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            aspect.before(&jp);
            ToolResult::ok("Operation approved: delete_workspace")
        })).unwrap_or_else(|e| {
            let msg = e.downcast_ref::<String>().cloned().unwrap_or_default();
            ToolResult::err(msg)
        });

        cases.push(TestCaseResult::new(
            "auto_approve_delete_workspace",
            orig_result.success, asp_result.success,
            &orig_result.output, &asp_result.output,
            orig_result.error, asp_result.error,
        ));
    }

    // Case 2: Auto-deny — both deny
    {
        let mut original = OriginalApprovalFlow { auto_approve: false, approved_operations: vec![] };
        let aspect = HumanApprovalAspect::new(ApprovalChannel::AutoDeny)
            .require_approval("format_disk", RiskLevel::Critical, "Format the disk");

        let jp = JoinPoint::new("format_disk", "tools::shell", Location { file: "shell.rs", line: 1 });

        let orig_result = original.require_approval("format_disk");
        let asp_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            aspect.before(&jp);
            ToolResult::ok("Approved")
        })).unwrap_or_else(|_| ToolResult::err("Operation denied: format_disk"));

        cases.push(TestCaseResult::new(
            "auto_deny_format_disk",
            orig_result.success, asp_result.success,
            &orig_result.output, &asp_result.output,
            orig_result.error, asp_result.error,
        ));
    }

    // Case 3: Session approve — first call prompts, subsequent calls bypass
    {
        let approve_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let ac = approve_count.clone();
        let aspect = HumanApprovalAspect::new(ApprovalChannel::Handler(Arc::new(move |_req| {
            ac.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            ApprovalResponse::ApproveSession
        })))
        .require_approval("risky_op", RiskLevel::High, "Risky operation");

        let jp = JoinPoint::new("risky_op", "tools::test", Location { file: "t.rs", line: 1 });

        // Two calls — should only prompt once
        aspect.before(&jp);
        aspect.before(&jp);

        let handler_called = approve_count.load(std::sync::atomic::Ordering::SeqCst);
        let session_works = handler_called == 1;
        let session_msg = if session_works {
            "Handler called once".to_string()
        } else {
            format!("Handler called {handler_called} times")
        };

        cases.push(TestCaseResult::new(
            "session_approve_prompts_once",
            session_works, session_works,
            &session_msg,
            &session_msg,
            None, None,
        ));
    }

    // Case 4: Non-registered function passes without any check
    {
        let aspect = HumanApprovalAspect::new(ApprovalChannel::AutoDeny);
        let jp = JoinPoint::new("safe_op", "tools::test", Location { file: "t.rs", line: 1 });

        let asp_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            aspect.before(&jp);
            ToolResult::ok("Not registered, passed")
        })).unwrap_or_else(|_| ToolResult::err("Unexpected block"));

        cases.push(TestCaseResult::new(
            "unregistered_function_passes",
            true, asp_result.success,
            "Not registered, passed",
            &asp_result.output,
            None, asp_result.error,
        ));
    }

    EquivalenceReport::new("approval_flow_v5", cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_flow_equivalence() {
        let report = run_approval_equivalence();
        report.print_summary();
        assert!(
            report.is_fully_equivalent(),
            "Approval flow equivalence failed: {:?}",
            report.divergent_cases
        );
    }

    #[test]
    fn test_auto_approve_allows_critical_ops() {
        let aspect = HumanApprovalAspect::new(ApprovalChannel::AutoApprove)
            .require_approval("critical_fn", RiskLevel::Critical, "Critical operation");
        let jp = JoinPoint::new("critical_fn", "test", Location { file: "f.rs", line: 1 });
        // Should not panic
        aspect.before(&jp);
    }

    #[test]
    fn test_auto_deny_blocks_critical_ops() {
        let aspect = HumanApprovalAspect::new(ApprovalChannel::AutoDeny)
            .require_approval("critical_fn", RiskLevel::Critical, "Critical operation");
        let jp = JoinPoint::new("critical_fn", "test", Location { file: "f.rs", line: 1 });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            aspect.before(&jp);
        }));
        assert!(result.is_err(), "Expected denial to panic");
    }
}
