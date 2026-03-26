//! Token budget aspect: prevent LLM cost overruns.
//!
//! Addresses the cost-tracking scattering in ZeroClaw where 41/192 files (21.4%)
//! reference token/cost vocabulary despite a dedicated `cost/` module holding only 2/41 files.
//!
//! This aspect centralizes token budget enforcement: before any LLM call, it checks
//! the remaining budget; after the call, it records actual usage.

use aspect_core::prelude::*;
use aspect_core::AspectError;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Token budget limits for controlling LLM API costs.
#[derive(Debug, Clone)]
pub struct BudgetLimits {
    /// Maximum tokens per day (0 = unlimited).
    pub daily_limit: u64,
    /// Maximum tokens per request (0 = unlimited).
    pub per_request_limit: u64,
    /// Estimated tokens per call when actual count is not known in advance.
    pub estimated_tokens_per_call: u64,
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            daily_limit: 100_000,
            per_request_limit: 4_096,
            estimated_tokens_per_call: 1_000,
        }
    }
}

/// Aspect that enforces token budget limits on LLM call functions.
///
/// Uses an `Arc<AtomicU64>` counter shared across all call sites so that the
/// budget is tracked globally. After each successful call, the actual token count
/// is extracted from the return value via a caller-supplied closure.
///
/// # Example
///
/// ```rust,ignore
/// use aspect_agent::TokenBudgetAspect;
/// use aspect_macros::aspect;
///
/// let budget = TokenBudgetAspect::new(BudgetLimits { daily_limit: 50_000, ..Default::default() });
///
/// #[aspect(budget.clone())]
/// fn call_llm(prompt: String) -> Result<LlmResponse, String> {
///     // ... actual LLM call
/// }
/// ```
#[derive(Clone)]
pub struct TokenBudgetAspect {
    limits: Arc<BudgetLimits>,
    /// Cumulative tokens used (shared across all instances created from the same Arc).
    tokens_used: Arc<AtomicU64>,
    /// Extracts actual token count from the function's return value (`&dyn Any`).
    /// When None, uses `limits.estimated_tokens_per_call` per invocation.
    usage_extractor: Option<Arc<dyn Fn(&dyn Any) -> Option<u64> + Send + Sync>>,
}

impl TokenBudgetAspect {
    /// Create a new budget aspect with given limits.
    pub fn new(limits: BudgetLimits) -> Self {
        Self {
            limits: Arc::new(limits),
            tokens_used: Arc::new(AtomicU64::new(0)),
            usage_extractor: None,
        }
    }

    /// Create with default limits (100K tokens/day).
    pub fn default_limits() -> Self {
        Self::new(BudgetLimits::default())
    }

    /// Attach a closure that extracts actual token usage from the return value.
    pub fn with_usage_extractor<F>(mut self, f: F) -> Self
    where
        F: Fn(&dyn Any) -> Option<u64> + Send + Sync + 'static,
    {
        self.usage_extractor = Some(Arc::new(f));
        self
    }

    /// Get the current total tokens used.
    pub fn tokens_used(&self) -> u64 {
        self.tokens_used.load(Ordering::Relaxed)
    }

    /// Reset the token counter (e.g., at the start of a new day).
    pub fn reset(&self) {
        self.tokens_used.store(0, Ordering::SeqCst);
    }

    /// Check if a call with estimated cost would exceed budget.
    fn check_budget(&self) -> Result<(), AspectError> {
        let current = self.tokens_used.load(Ordering::Relaxed);

        if self.limits.daily_limit > 0 {
            let projected = current + self.limits.estimated_tokens_per_call;
            if projected > self.limits.daily_limit {
                return Err(AspectError::execution(format!(
                    "Token budget exceeded: {current} used, limit is {} (projected {projected})",
                    self.limits.daily_limit
                )));
            }
        }

        Ok(())
    }

    fn record_usage(&self, actual: u64) {
        self.tokens_used.fetch_add(actual, Ordering::Relaxed);
    }
}

impl Aspect for TokenBudgetAspect {
    fn before(&self, ctx: &JoinPoint) {
        if let Err(e) = self.check_budget() {
            panic!(
                "[TokenBudget] Budget check failed for {}: {}",
                ctx.function_name, e
            );
        }
    }

    fn after(&self, _ctx: &JoinPoint, result: &dyn Any) {
        let tokens = if let Some(ref extractor) = self.usage_extractor {
            extractor(result).unwrap_or(self.limits.estimated_tokens_per_call)
        } else {
            self.limits.estimated_tokens_per_call
        };
        self.record_usage(tokens);
    }

    fn after_error(&self, _ctx: &JoinPoint, _error: &AspectError) {
        // On error, charge the estimated cost (partial processing likely occurred)
        self.record_usage(self.limits.estimated_tokens_per_call / 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aspect_core::prelude::*;

    fn make_jp() -> JoinPoint {
        JoinPoint::new("llm_call", "test", Location { file: "f.rs", line: 1 })
    }

    #[test]
    fn test_initial_usage_is_zero() {
        let budget = TokenBudgetAspect::default_limits();
        assert_eq!(budget.tokens_used(), 0);
    }

    #[test]
    fn test_after_records_estimated_usage() {
        let budget = TokenBudgetAspect::new(BudgetLimits {
            estimated_tokens_per_call: 500,
            ..Default::default()
        });
        let jp = make_jp();
        budget.after(&jp, &());
        assert_eq!(budget.tokens_used(), 500);
    }

    #[test]
    fn test_after_uses_extractor_when_set() {
        let budget = TokenBudgetAspect::default_limits()
            .with_usage_extractor(|result: &dyn Any| {
                result.downcast_ref::<u64>().copied()
            });
        let jp = make_jp();
        let actual_tokens: u64 = 1234;
        budget.after(&jp, &actual_tokens);
        assert_eq!(budget.tokens_used(), 1234);
    }

    #[test]
    fn test_before_passes_under_budget() {
        let budget = TokenBudgetAspect::new(BudgetLimits {
            daily_limit: 10_000,
            estimated_tokens_per_call: 100,
            ..Default::default()
        });
        let jp = make_jp();
        // Should not panic
        budget.before(&jp);
    }

    #[test]
    #[should_panic(expected = "Budget check failed")]
    fn test_before_panics_over_budget() {
        let budget = TokenBudgetAspect::new(BudgetLimits {
            daily_limit: 500,
            estimated_tokens_per_call: 1000,
            ..Default::default()
        });
        let jp = make_jp();
        budget.before(&jp);
    }

    #[test]
    fn test_reset_clears_counter() {
        let budget = TokenBudgetAspect::default_limits();
        let jp = make_jp();
        budget.after(&jp, &());
        assert!(budget.tokens_used() > 0);
        budget.reset();
        assert_eq!(budget.tokens_used(), 0);
    }

    #[test]
    fn test_cumulative_usage_across_calls() {
        let budget = TokenBudgetAspect::new(BudgetLimits {
            daily_limit: 100_000,
            estimated_tokens_per_call: 100,
            ..Default::default()
        });
        let jp = make_jp();
        for _ in 0..5 {
            budget.before(&jp);
            budget.after(&jp, &());
        }
        assert_eq!(budget.tokens_used(), 500);
    }

    #[test]
    fn test_aspect_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TokenBudgetAspect>();
    }

    #[test]
    fn test_error_charges_half_estimated() {
        let budget = TokenBudgetAspect::new(BudgetLimits {
            estimated_tokens_per_call: 200,
            ..Default::default()
        });
        let jp = make_jp();
        let err = AspectError::execution("llm timeout");
        budget.after_error(&jp, &err);
        assert_eq!(budget.tokens_used(), 100); // half of 200
    }
}
