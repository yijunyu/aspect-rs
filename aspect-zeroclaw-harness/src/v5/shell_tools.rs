//! v5 integration tests: shell tool rate limiting aspect vs. original ActionTracker.
//!
//! Tests that RateLimitAspect produces identical rejection behavior to ZeroClaw's
//! security/policy.rs ActionTracker when applied to shell tool execution.

use aspect_std::RateLimitAspect;
use aspect_core::prelude::*;
use aspect_core::AspectError;
use crate::correctness::{EquivalenceReport, TestCaseResult, ToolResult};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Simulates ZeroClaw's original ActionTracker rate limiting behavior.
struct OriginalActionTracker {
    max_per_hour: usize,
    call_count: Arc<Mutex<usize>>,
}

impl OriginalActionTracker {
    fn new(max_per_hour: usize) -> Self {
        Self {
            max_per_hour,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Mirrors ZeroClaw's shell.rs inline rate limit check.
    fn check_and_record(&self) -> ToolResult {
        let mut count = self.call_count.lock().unwrap();
        if *count >= self.max_per_hour {
            return ToolResult::err(
                "Rate limit exceeded: too many actions in the last hour",
            );
        }
        *count += 1;
        ToolResult::ok(format!("Action recorded (count: {})", *count))
    }
}

/// Simulates the aspectized rate limit using RateLimitAspect.
fn call_with_rate_limit_aspect(
    aspect: &RateLimitAspect,
    call_num: usize,
) -> ToolResult {
    let jp = JoinPoint::new(
        "execute",
        "tools::shell",
        Location { file: "shell.rs", line: 1 },
    );

    let pjp = ProceedingJoinPoint::new(
        || Ok(Box::new(format!("Shell execution allowed (call: {call_num})")) as Box<dyn std::any::Any>),
        jp,
    );

    match aspect.around(pjp) {
        Ok(_) => ToolResult::ok(format!("Shell execution allowed (call: {call_num})")),
        Err(e) => ToolResult::err(format!("{e}")),
    }
}

/// Generate the shell tool rate limiting equivalence report.
pub fn run_shell_tool_equivalence() -> EquivalenceReport {
    let max_per_hour = 5;
    let original = OriginalActionTracker::new(max_per_hour);
    let aspect = RateLimitAspect::new(max_per_hour as u64, Duration::from_secs(60));

    let mut cases = Vec::new();

    // Test calls 1 through max+2
    for i in 1..=(max_per_hour + 2) {
        let orig_result = original.check_and_record();
        let asp_result = call_with_rate_limit_aspect(&aspect, i);

        cases.push(TestCaseResult::new(
            format!("call_{i}"),
            orig_result.success,
            asp_result.success,
            &orig_result.output,
            &asp_result.output,
            orig_result.error.clone(),
            asp_result.error.clone(),
        ));
    }

    EquivalenceReport::new("shell_tool_rate_limit_v5", cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_rate_limit_equivalence() {
        let report = run_shell_tool_equivalence();
        report.print_summary();
        assert!(
            report.is_fully_equivalent(),
            "Shell tool rate limit equivalence failed: {:?}",
            report.divergent_cases
        );
    }

    #[test]
    fn test_rate_limit_allows_under_threshold() {
        let aspect = RateLimitAspect::new(10, Duration::from_secs(3600));
        // First 10 calls should succeed
        for i in 1..=10 {
            let result = call_with_rate_limit_aspect(&aspect, i);
            assert!(result.success, "Call {i} should be allowed");
        }
    }

    #[test]
    fn test_rate_limit_blocks_at_threshold() {
        let aspect = RateLimitAspect::new(3, Duration::from_secs(3600));
        // First 3 pass
        for i in 1..=3 {
            let result = call_with_rate_limit_aspect(&aspect, i);
            assert!(result.success, "Call {i} should be allowed");
        }
        // 4th is rejected
        let result = call_with_rate_limit_aspect(&aspect, 4);
        assert!(!result.success, "Call 4 should be rejected");
    }

    #[test]
    fn test_original_and_aspect_agree_on_threshold() {
        let max = 3;
        let original = OriginalActionTracker::new(max);
        let aspect = RateLimitAspect::new(max as u64, Duration::from_secs(3600));

        for i in 1..=(max + 2) {
            let orig_success = original.check_and_record().success;
            let asp_success = call_with_rate_limit_aspect(&aspect, i).success;
            assert_eq!(
                orig_success, asp_success,
                "Call {i}: original={orig_success} aspect={asp_success}"
            );
        }
    }
}
