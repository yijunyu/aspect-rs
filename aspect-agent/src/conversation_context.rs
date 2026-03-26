//! Conversation context window management aspect.
//!
//! LLM context windows have hard limits (e.g., 200K tokens for Claude).
//! This aspect tracks cumulative context size across an agent session and
//! enforces limits before each LLM call, preventing context overflow errors.
//!
//! Complements `TokenBudgetAspect` (cost focus) by focusing on context window limits
//! rather than API cost.

use aspect_core::prelude::*;
use aspect_core::AspectError;
use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// What to do when the context window is about to overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowStrategy {
    /// Return an error, preventing the call (safest option).
    Fail,
    /// Log a warning and proceed anyway (may cause API error).
    Warn,
}

/// Aspect that enforces context window size limits on LLM call functions.
///
/// Tracks cumulative token consumption and checks against the configured limit
/// before each call. After each call, the actual token count can be recorded
/// via an extractor closure.
///
/// # Example
///
/// ```rust,ignore
/// use aspect_agent::conversation_context::{ConversationContextAspect, OverflowStrategy};
/// use aspect_macros::aspect;
///
/// let ctx_guard = ConversationContextAspect::new(200_000, OverflowStrategy::Fail);
///
/// #[aspect(ctx_guard.clone())]
/// fn call_claude(messages: Vec<Message>) -> Result<Response, ApiError> {
///     // Context size is checked before this runs
/// }
/// ```
#[derive(Clone)]
pub struct ConversationContextAspect {
    /// Maximum context window size in tokens.
    max_tokens: usize,
    /// Current accumulated context size.
    current_tokens: Arc<AtomicUsize>,
    /// Strategy when limit is exceeded.
    overflow_strategy: OverflowStrategy,
    /// Extracts the token count from the return value of the decorated function.
    usage_extractor: Option<Arc<dyn Fn(&dyn Any) -> Option<usize> + Send + Sync>>,
    /// Estimates tokens per call when no extractor is available.
    estimated_tokens_per_call: usize,
}

impl ConversationContextAspect {
    /// Create a new context aspect with the given limit and overflow strategy.
    pub fn new(max_tokens: usize, overflow_strategy: OverflowStrategy) -> Self {
        Self {
            max_tokens,
            current_tokens: Arc::new(AtomicUsize::new(0)),
            overflow_strategy,
            usage_extractor: None,
            estimated_tokens_per_call: 2_000,
        }
    }

    /// Set the estimated tokens consumed per call (used when no extractor is set).
    pub fn with_estimated_tokens(mut self, tokens: usize) -> Self {
        self.estimated_tokens_per_call = tokens;
        self
    }

    /// Attach an extractor that reads actual token usage from the return value.
    pub fn with_usage_extractor<F>(mut self, f: F) -> Self
    where
        F: Fn(&dyn Any) -> Option<usize> + Send + Sync + 'static,
    {
        self.usage_extractor = Some(Arc::new(f));
        self
    }

    /// Get the current accumulated context token count.
    pub fn current_tokens(&self) -> usize {
        self.current_tokens.load(Ordering::Relaxed)
    }

    /// Reset the context window counter (e.g., when starting a new conversation).
    pub fn reset(&self) {
        self.current_tokens.store(0, Ordering::SeqCst);
    }

    /// Remaining tokens before the context window is full.
    pub fn remaining_tokens(&self) -> usize {
        self.max_tokens.saturating_sub(self.current_tokens())
    }

    fn check_overflow(&self) -> Result<(), AspectError> {
        let current = self.current_tokens();
        let projected = current + self.estimated_tokens_per_call;

        if projected > self.max_tokens {
            let msg = format!(
                "Context window overflow: {current} tokens used, limit is {}, projected next call would use {projected}",
                self.max_tokens
            );
            match self.overflow_strategy {
                OverflowStrategy::Fail => return Err(AspectError::execution(msg)),
                OverflowStrategy::Warn => eprintln!("WARNING [ConversationContext]: {msg}"),
            }
        }
        Ok(())
    }

    fn record_tokens(&self, tokens: usize) {
        self.current_tokens.fetch_add(tokens, Ordering::Relaxed);
    }
}

impl Aspect for ConversationContextAspect {
    fn before(&self, ctx: &JoinPoint) {
        if let Err(e) = self.check_overflow() {
            panic!(
                "[ConversationContext] Overflow for {}: {}",
                ctx.function_name, e
            );
        }
    }

    fn after(&self, _ctx: &JoinPoint, result: &dyn Any) {
        let tokens = if let Some(ref extractor) = self.usage_extractor {
            extractor(result).unwrap_or(self.estimated_tokens_per_call)
        } else {
            self.estimated_tokens_per_call
        };
        self.record_tokens(tokens);
    }

    fn after_error(&self, _ctx: &JoinPoint, _error: &AspectError) {
        // On error, still charge some context (prompt was sent)
        self.record_tokens(self.estimated_tokens_per_call / 4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aspect_core::prelude::*;

    fn make_jp() -> JoinPoint {
        JoinPoint::new("call_llm", "test", Location { file: "f.rs", line: 1 })
    }

    #[test]
    fn test_initial_state() {
        let ctx = ConversationContextAspect::new(200_000, OverflowStrategy::Fail);
        assert_eq!(ctx.current_tokens(), 0);
        assert_eq!(ctx.remaining_tokens(), 200_000);
    }

    #[test]
    fn test_after_increments_counter() {
        let ctx = ConversationContextAspect::new(200_000, OverflowStrategy::Fail)
            .with_estimated_tokens(1_000);
        let jp = make_jp();
        ctx.after(&jp, &());
        assert_eq!(ctx.current_tokens(), 1_000);
    }

    #[test]
    fn test_extractor_used_when_set() {
        let ctx = ConversationContextAspect::new(200_000, OverflowStrategy::Fail)
            .with_usage_extractor(|result: &dyn Any| result.downcast_ref::<usize>().copied());
        let jp = make_jp();
        let actual: usize = 3_500;
        ctx.after(&jp, &actual);
        assert_eq!(ctx.current_tokens(), 3_500);
    }

    #[test]
    fn test_before_passes_under_limit() {
        let ctx = ConversationContextAspect::new(200_000, OverflowStrategy::Fail)
            .with_estimated_tokens(1_000);
        let jp = make_jp();
        // Should not panic
        ctx.before(&jp);
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn test_before_panics_over_limit_fail_strategy() {
        let ctx = ConversationContextAspect::new(500, OverflowStrategy::Fail)
            .with_estimated_tokens(1_000);
        let jp = make_jp();
        ctx.before(&jp); // 1000 > 500 → panic
    }

    #[test]
    fn test_warn_strategy_does_not_panic() {
        let ctx = ConversationContextAspect::new(500, OverflowStrategy::Warn)
            .with_estimated_tokens(1_000);
        let jp = make_jp();
        // Should NOT panic (Warn mode)
        ctx.before(&jp);
    }

    #[test]
    fn test_reset_clears_counter() {
        let ctx = ConversationContextAspect::new(200_000, OverflowStrategy::Fail)
            .with_estimated_tokens(500);
        let jp = make_jp();
        ctx.after(&jp, &());
        assert_eq!(ctx.current_tokens(), 500);
        ctx.reset();
        assert_eq!(ctx.current_tokens(), 0);
    }

    #[test]
    fn test_multiple_calls_accumulate() {
        let ctx = ConversationContextAspect::new(200_000, OverflowStrategy::Fail)
            .with_estimated_tokens(100);
        let jp = make_jp();
        for _ in 0..10 {
            ctx.before(&jp);
            ctx.after(&jp, &());
        }
        assert_eq!(ctx.current_tokens(), 1_000);
    }

    #[test]
    fn test_remaining_tokens_decreases() {
        let ctx = ConversationContextAspect::new(5_000, OverflowStrategy::Fail)
            .with_estimated_tokens(1_000);
        let jp = make_jp();
        ctx.after(&jp, &());
        assert_eq!(ctx.remaining_tokens(), 4_000);
    }

    #[test]
    fn test_error_charges_quarter_estimated() {
        let ctx = ConversationContextAspect::new(200_000, OverflowStrategy::Fail)
            .with_estimated_tokens(400);
        let jp = make_jp();
        let err = AspectError::execution("api error");
        ctx.after_error(&jp, &err);
        assert_eq!(ctx.current_tokens(), 100); // 400/4
    }

    #[test]
    fn test_aspect_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ConversationContextAspect>();
    }
}
