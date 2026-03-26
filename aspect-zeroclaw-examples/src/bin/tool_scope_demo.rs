//! Demo: ToolScopeAspect applied to ZeroClaw-style tool execution.
//!
//! Shows how ToolScopeAspect replaces the inline path validation scattered
//! across 65/192 ZeroClaw files (33.9%).

use aspect_agent::tool_scope::{ToolScopeAspect, ToolScopePolicy, set_tool_path, clear_tool_path};
use aspect_core::prelude::*;
use std::path::PathBuf;

fn check_path(aspect: &ToolScopeAspect, path: &str, label: &str) {
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
        Ok(_) => println!("  ✓ ALLOWED  {label}: {path}"),
        Err(e) => {
            let msg = e.downcast_ref::<String>().cloned().unwrap_or_else(|| "denied".into());
            println!("  ✗ BLOCKED  {label}: {path}");
            println!("             → {msg}");
        }
    }
}

fn main() {
    let tmp = std::env::temp_dir();
    let workspace = tmp.join("zeroclaw_demo_workspace");
    std::fs::create_dir_all(&workspace).ok();
    let test_file = workspace.join("workspace_file.txt");
    std::fs::write(&test_file, "workspace content").ok();

    let policy = ToolScopePolicy::new(&workspace);
    let aspect = ToolScopeAspect::new(policy);

    println!("=== ToolScopeAspect Demo ===");
    println!("Workspace: {}", workspace.display());
    println!("\nPath access decisions:");

    // Workspace file — should be allowed
    check_path(&aspect, &test_file.to_string_lossy(), "workspace file");

    // Forbidden system paths — should all be blocked
    check_path(&aspect, "/etc/passwd", "forbidden system file");
    check_path(&aspect, "/proc/self/mem", "proc filesystem");
    check_path(&aspect, "/root/.ssh/id_rsa", "root SSH key");
    check_path(&aspect, "/sys/kernel/debug", "kernel debug");

    // Outside workspace — should be blocked
    check_path(&aspect, "/tmp/outside_workspace.txt", "outside workspace");

    println!("\nKey insight: ToolScopeAspect replaces ~12 LOC per tool of inline path validation");
    println!("ZeroClaw had path validation scattered in 65/192 files (33.9%)");
    println!("With this aspect: one canonical implementation, applied declaratively");
}
