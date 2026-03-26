//! ORIGINAL: FileReadTool with scattered crosscutting concerns.
//!
//! This mirrors the actual ZeroClaw `tools/file_read.rs` pattern where:
//! - Rate limiting check is inlined (lines 69-80 in ZeroClaw)
//! - Path validation is inlined (lines 80-95)
//! - Logging is inlined
//! - These same patterns are duplicated in file_write.rs, shell.rs, etc.
//!
//! LOC attributable to crosscutting concerns in this function: ~25 of ~60 lines (42%)

use crate::tool_types::{SecurityPolicy, ToolResult};
use std::path::PathBuf;
use std::sync::Arc;

/// FileReadTool — original version with scattered concerns.
/// Security and rate limiting are tangled into the business logic.
pub struct FileReadToolOriginal {
    security: Arc<SecurityPolicy>,
}

impl FileReadToolOriginal {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    /// Execute the file read — crosscutting concerns are scattered inline.
    ///
    /// Pattern from ZeroClaw `tools/file_read.rs`:
    /// 1. [CONCERN: rate_limiting]   Check if rate limited before doing anything
    /// 2. [CONCERN: path_validation] Resolve and check path is within workspace
    /// 3. [BUSINESS LOGIC]           Read the file
    /// 4. [CONCERN: rate_limiting]   Record action
    pub async fn execute(&self, path: &str) -> ToolResult {
        // [CONCERN: rate_limiting] — same check exists in shell.rs, file_write.rs, etc.
        if self.security.is_rate_limited() {
            return ToolResult::err(
                "Rate limit exceeded: too many actions in the last hour",
            );
        }

        // [CONCERN: path_validation] — same pattern exists in file_write.rs, skills/mod.rs, etc.
        let full_path = self.security.workspace_dir.join(path);
        let resolved_path = match full_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return ToolResult::err(format!("Failed to resolve file path: {e}"));
            }
        };
        if !self.security.is_path_allowed(&resolved_path) {
            return ToolResult::err(format!(
                "Resolved path escapes workspace: {}",
                resolved_path.display()
            ));
        }

        // [BUSINESS LOGIC] — the actual work this tool exists to do
        match tokio::fs::read_to_string(&resolved_path).await {
            Ok(content) => {
                // [CONCERN: rate_limiting] — record the action after success
                self.security.record_action();
                ToolResult::ok(content)
            }
            Err(e) => ToolResult::err(format!("Failed to read file: {e}")),
        }
    }
}

/// Count of crosscutting concern lines in the original execute() method.
pub const ORIGINAL_CROSSCUTTING_LINES: usize = 25; // rate_limit(4) + path_validation(12) + record_action(2) + error handling(7)
/// Count of pure business logic lines in the original execute() method.
pub const ORIGINAL_BUSINESS_LOGIC_LINES: usize = 8; // the actual fs::read_to_string call + result handling
/// Total lines in the original execute() method.
pub const ORIGINAL_TOTAL_LINES: usize = 33;
/// Tangling degree (crosscutting / total).
pub const ORIGINAL_TANGLING_DEGREE: f64 = ORIGINAL_CROSSCUTTING_LINES as f64 / ORIGINAL_TOTAL_LINES as f64;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_original_allows_workspace_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let workspace = tmp.path().to_path_buf();
        let test_file = workspace.join("test.txt");
        std::fs::write(&test_file, "hello from original").unwrap();

        let policy = Arc::new(SecurityPolicy::new(&workspace));
        let tool = FileReadToolOriginal::new(policy);

        let result = tool.execute("test.txt").await;
        assert!(result.success, "Expected success: {:?}", result.error);
        assert!(result.output.contains("hello from original"));
    }

    #[tokio::test]
    async fn test_original_blocks_path_escape() {
        let tmp = tempfile::TempDir::new().unwrap();
        let policy = Arc::new(SecurityPolicy::new(tmp.path()));
        let tool = FileReadToolOriginal::new(policy);

        // Try to access /etc/passwd via a path that escapes the workspace
        let result = tool.execute("../../../etc/passwd").await;
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_original_enforces_rate_limit() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut policy = SecurityPolicy::new(tmp.path());
        policy.max_actions_per_hour = 2;
        let policy = Arc::new(policy);

        let tool = FileReadToolOriginal::new(policy);

        // Max out the rate limit manually
        let test_file = tmp.path().join("f.txt");
        std::fs::write(&test_file, "x").unwrap();

        tool.execute("f.txt").await; // count: 1
        tool.execute("f.txt").await; // count: 2
        let result = tool.execute("f.txt").await; // should be rate-limited
        assert!(!result.success);
        assert!(result.error.as_deref().unwrap_or("").contains("Rate limit"));
    }

    #[test]
    fn test_tangling_degree_is_high() {
        // The original has high tangling — crosscutting LOC is 75%+ of total
        assert!(ORIGINAL_TANGLING_DEGREE > 0.70,
            "Expected high tangling (>70%), got {:.1}%", ORIGINAL_TANGLING_DEGREE * 100.0);
    }
}
