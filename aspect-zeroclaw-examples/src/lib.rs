//! # aspect-zeroclaw-examples
//!
//! Working examples of aspect-rs patterns applied to ZeroClaw agent framework tools.
//!
//! Demonstrates all 14 RE2026 patterns against tool implementations shaped like ZeroClaw's
//! `Tool` trait (`async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult>`).
//!
//! ## Traditional Patterns (8)
//! - Logging, Timing, Caching, Metrics, Rate Limiting, Circuit Breaker, Authorization, Validation
//!
//! ## Agent-Specific Patterns (6)
//! - Tool Scope, Prompt Injection Defense, Token Budget, Tool Call Audit, Human Approval, Context Window
//!
//! ## Before/After Comparison
//! - `original/` — ZeroClaw tools with scattered crosscutting concerns inline
//! - `aspectized/` — same tools with concerns extracted to aspects

pub mod original;
pub mod aspectized;
pub mod tool_types;
