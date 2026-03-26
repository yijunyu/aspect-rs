//! (D) v5 integration tests: live execution against ZeroClaw current codebase.
//!
//! These tests verify that aspectized tool wrappers produce identical behavior
//! to the original ZeroClaw tools when run against the v5 (current HEAD) codebase.

pub mod file_tools;
pub mod shell_tools;
pub mod approval_flow;
