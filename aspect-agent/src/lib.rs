//! # aspect-agent
//!
//! Agent-specific aspects for agentic AI systems.
//!
//! Provides 6 patterns from the RE2026 research on Aspect-Oriented Pattern Language
//! for Agentic AI Systems, addressing crosscutting concerns unique to LLM agent frameworks:
//!
//! - **[`ToolScopeAspect`]** — filesystem/network sandboxing (inspired by ZeroClaw `security/policy.rs`)
//! - **[`PromptInjectionAspect`]** — detect and block malicious LLM inputs
//! - **[`TokenBudgetAspect`]** — prevent token cost overruns
//! - **[`ToolCallAuditAspect`]** — compliance-grade audit trail for every tool invocation
//! - **[`HumanApprovalAspect`]** — human-in-the-loop gates for critical operations
//! - **[`ConversationContextAspect`]** — context window size enforcement
//!
//! ## Recommended Composition Order
//!
//! When stacking multiple aspects on a function, apply them outermost-first:
//! ```text
//! ToolScope → RateLimit → HumanApproval → TokenBudget → Audit → Logging → fn
//! ```
//!
//! ## Async Compatibility
//!
//! All aspects implement the synchronous [`Aspect`](aspect_core::Aspect) trait. When applied to
//! `async fn` via `#[aspect(...)]`, the macro uses `before`/`after`/`after_error` advice
//! (not `around`). Only `ToolScopeAspect` uses `around` for early-return semantics; it performs
//! its path check synchronously before any async work begins.

pub mod conversation_context;
pub mod human_approval;
pub mod prompt_injection;
pub mod token_budget;
pub mod tool_call_audit;
pub mod tool_scope;

pub use conversation_context::ConversationContextAspect;
pub use human_approval::{ApprovalChannel, ApprovalRequest, ApprovalResponse, HumanApprovalAspect};
pub use prompt_injection::{InjectionAction, PromptInjectionAspect};
pub use token_budget::TokenBudgetAspect;
pub use tool_call_audit::{
    AuditEntry, AuditOutcome, AuditStorage, InMemoryAuditStorage, ToolCallAuditAspect,
};
pub use tool_scope::{ToolScopeAspect, ToolScopePolicy};

pub mod prelude {
    pub use crate::{
        ConversationContextAspect, HumanApprovalAspect, InjectionAction, PromptInjectionAspect,
        TokenBudgetAspect, ToolCallAuditAspect, ToolScopeAspect, ToolScopePolicy,
    };
}
