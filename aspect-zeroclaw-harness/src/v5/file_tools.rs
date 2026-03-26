//! v5 integration tests: file tool aspects vs. original ZeroClaw security behavior.
//!
//! Tests that ToolScopeAspect + RateLimitAspect produce the same access control
//! decisions as ZeroClaw's SecurityPolicy when applied to file operations.

use aspect_agent::tool_scope::{ToolScopeAspect, ToolScopePolicy, set_tool_path, clear_tool_path};
use aspect_core::prelude::*;
use crate::correctness::{EquivalenceReport, TestCaseResult, ToolResult};
use std::path::Path;
use std::sync::Arc;

/// Simulates ZeroClaw's original SecurityPolicy path check behavior.
fn original_path_check(path: &str, workspace: &str) -> ToolResult {
    let path = Path::new(path);
    let workspace = Path::new(workspace);

    // Simulate ZeroClaw security/policy.rs::is_path_allowed
    let forbidden = ["/etc", "/root", "/proc", "/sys", "/dev", "/boot"];

    for prefix in &forbidden {
        if path.starts_with(prefix) {
            return ToolResult::err(format!(
                "Access to forbidden path denied: {}",
                path.display()
            ));
        }
    }

    // Simulate workspace containment check
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let canonical_ws = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());

    if !canonical_path.starts_with(&canonical_ws) {
        return ToolResult::err(format!(
            "Resolved path escapes workspace boundary: {}",
            path.display()
        ));
    }

    ToolResult::ok(format!("File access allowed: {}", path.display()))
}

/// Simulates the aspectized path check using ToolScopeAspect.
fn aspectized_path_check(path: &str, workspace: &str) -> ToolResult {
    let policy = ToolScopePolicy::new(workspace);
    let aspect = ToolScopeAspect::new(policy);

    let jp = JoinPoint::new(
        "execute",
        "tools::file_read",
        Location { file: "file_read.rs", line: 1 },
    );

    set_tool_path(path);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        aspect.before(&jp);
    }));

    clear_tool_path();

    match result {
        Ok(_) => ToolResult::ok(format!("File access allowed: {path}")),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Access denied".to_string()
            };
            ToolResult::err(msg)
        }
    }
}

/// Generate the full file tool equivalence report.
pub fn run_file_tool_equivalence() -> EquivalenceReport {
    let workspace = std::env::temp_dir().join("zeroclaw_v5_workspace");
    std::fs::create_dir_all(&workspace).ok();
    let ws_str = workspace.to_string_lossy().to_string();

    // Create a test file inside the workspace
    let test_file = workspace.join("test.txt");
    std::fs::write(&test_file, "test content").ok();
    let test_file_str = test_file.to_string_lossy().to_string();

    let cases = vec![
        // Happy path: file inside workspace
        {
            let orig = original_path_check(&test_file_str, &ws_str);
            let asp = aspectized_path_check(&test_file_str, &ws_str);
            TestCaseResult::new(
                "workspace_file_allowed",
                orig.success,
                asp.success,
                &orig.output,
                &asp.output,
                orig.error.clone(),
                asp.error.clone(),
            )
        },
        // Security: /etc/passwd blocked
        {
            let orig = original_path_check("/etc/passwd", &ws_str);
            let asp = aspectized_path_check("/etc/passwd", &ws_str);
            TestCaseResult::new(
                "etc_passwd_blocked",
                orig.success,
                asp.success,
                &orig.output,
                &asp.output,
                orig.error.clone(),
                asp.error.clone(),
            )
        },
        // Security: /proc/self/mem blocked
        {
            let orig = original_path_check("/proc/self/mem", &ws_str);
            let asp = aspectized_path_check("/proc/self/mem", &ws_str);
            TestCaseResult::new(
                "proc_mem_blocked",
                orig.success,
                asp.success,
                &orig.output,
                &asp.output,
                orig.error.clone(),
                asp.error.clone(),
            )
        },
        // Security: /root directory blocked
        {
            let orig = original_path_check("/root/.bashrc", &ws_str);
            let asp = aspectized_path_check("/root/.bashrc", &ws_str);
            TestCaseResult::new(
                "root_bashrc_blocked",
                orig.success,
                asp.success,
                &orig.output,
                &asp.output,
                orig.error.clone(),
                asp.error.clone(),
            )
        },
        // Workspace escape: path outside workspace
        {
            let outside = std::env::temp_dir().join("outside_workspace").join("file.txt");
            let outside_str = outside.to_string_lossy().to_string();
            let orig = original_path_check(&outside_str, &ws_str);
            let asp = aspectized_path_check(&outside_str, &ws_str);
            TestCaseResult::new(
                "workspace_escape_blocked",
                orig.success,
                asp.success,
                &orig.output,
                &asp.output,
                orig.error.clone(),
                asp.error.clone(),
            )
        },
    ];

    EquivalenceReport::new("file_tool_v5", cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_tool_equivalence_all_pass() {
        let report = run_file_tool_equivalence();
        report.print_summary();
        assert!(
            report.is_fully_equivalent(),
            "File tool equivalence failed: {:?}",
            report.divergent_cases
        );
    }

    #[test]
    fn test_aspectized_allows_workspace_file() {
        let workspace = std::env::temp_dir().join("test_allow_ws");
        std::fs::create_dir_all(&workspace).ok();
        let test_file = workspace.join("allowed.txt");
        std::fs::write(&test_file, "content").ok();

        let result = aspectized_path_check(
            &test_file.to_string_lossy(),
            &workspace.to_string_lossy(),
        );
        assert!(result.success, "Expected allowed, got: {:?}", result.error);
    }

    #[test]
    fn test_aspectized_blocks_etc() {
        let result = aspectized_path_check("/etc/shadow", "/tmp/workspace");
        assert!(!result.success);
        assert!(result.error.as_deref().unwrap_or("").contains("denied")
            || result.error.as_deref().unwrap_or("").contains("forbidden"));
    }

    #[test]
    fn test_original_and_aspectized_agree_on_etc() {
        let ws = "/tmp/workspace";
        let orig = original_path_check("/etc/passwd", ws);
        let asp = aspectized_path_check("/etc/passwd", ws);
        assert_eq!(orig.success, asp.success);
    }
}
