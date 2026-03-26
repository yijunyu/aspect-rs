//! Demo: HumanApprovalAspect applied to high-risk ZeroClaw operations.
//!
//! Shows how HumanApprovalAspect centralizes the approval/HITL concern that
//! was scattered across 17/192 ZeroClaw files (8.9%) — the worst centralization ratio.

use aspect_agent::human_approval::{ApprovalChannel, ApprovalResponse, HumanApprovalAspect, RiskLevel};
use aspect_core::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    println!("=== HumanApprovalAspect Demo ===\n");

    let approval_count = Arc::new(AtomicUsize::new(0));
    let ac = approval_count.clone();

    // Simulated approval handler (in production: Slack message, web UI, etc.)
    let gate = HumanApprovalAspect::new(ApprovalChannel::Handler(Arc::new(move |req| {
        let count = ac.fetch_add(1, Ordering::SeqCst) + 1;
        println!("  [APPROVAL REQUEST #{count}]");
        println!("  Function: {} (risk: {})", req.function_name, req.risk.label());
        println!("  Description: {}", req.description);
        println!("  → Auto-approving for demo");
        ApprovalResponse::Approve
    })))
    .require_approval("execute_shell", RiskLevel::High, "Execute arbitrary shell command")
    .require_approval("delete_workspace", RiskLevel::Critical, "Delete all workspace files")
    .require_approval("send_message", RiskLevel::Medium, "Send message to external channel");

    let functions = [
        ("execute_shell", "tools::shell", "shell.rs"),
        ("read_file", "tools::file_read", "file_read.rs"),      // NOT registered → passes freely
        ("delete_workspace", "tools::admin", "admin.rs"),
        ("send_message", "channels::slack", "slack.rs"),
        ("read_file", "tools::file_read", "file_read.rs"),      // Second call → still passes (not registered)
        ("execute_shell", "tools::shell", "shell.rs"),           // Second call → prompts again
    ];

    println!("Executing {} operations with HumanApprovalAspect:\n", functions.len());

    for (fn_name, module, file) in &functions {
        let jp = JoinPoint::new(fn_name, module, Location { file, line: 1 });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gate.before(&jp);
            "execution succeeded"
        }));

        match result {
            Ok(msg) => println!("  ✓ {fn_name}: {msg}"),
            Err(_) => println!("  ✗ {fn_name}: denied by human operator"),
        }
    }

    println!("\nTotal approval prompts sent: {}", approval_count.load(Ordering::SeqCst));
    println!("\nKey insight: HumanApprovalAspect centralizes the approval concern");
    println!("ZeroClaw had approval logic scattered in 17/192 files (only 1 in approval/ module)");
    println!("With this aspect: declarative approval gates, testable with AutoApprove/AutoDeny");
}
