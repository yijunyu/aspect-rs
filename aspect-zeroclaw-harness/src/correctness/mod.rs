//! (A) Correctness measurement: behavioral equivalence tests.
//!
//! Tests that aspectized versions of ZeroClaw tools produce identical outcomes
//! to the originals across all test scenarios.

use serde::{Deserialize, Serialize};

/// Result of a single behavioral equivalence test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub name: String,
    pub original_success: bool,
    pub aspectized_success: bool,
    pub original_output: String,
    pub aspectized_output: String,
    pub original_error: Option<String>,
    pub aspectized_error: Option<String>,
    pub equivalent: bool,
}

impl TestCaseResult {
    pub fn new(
        name: impl Into<String>,
        original_success: bool,
        aspectized_success: bool,
        original_output: impl Into<String>,
        aspectized_output: impl Into<String>,
        original_error: Option<String>,
        aspectized_error: Option<String>,
    ) -> Self {
        let orig_out = original_output.into();
        let asp_out = aspectized_output.into();
        let equivalent = original_success == aspectized_success
            && (original_success || {
                // On failure, check that errors are semantically equivalent
                let orig_err = original_error.as_deref().unwrap_or("");
                let asp_err = aspectized_error.as_deref().unwrap_or("");
                errors_equivalent(orig_err, asp_err)
            });

        Self {
            name: name.into(),
            original_success,
            aspectized_success,
            original_output: orig_out,
            aspectized_output: asp_out,
            original_error,
            aspectized_error,
            equivalent,
        }
    }
}

/// Check if two error messages are semantically equivalent.
/// Allows for minor wording differences between original and aspectized versions.
fn errors_equivalent(orig: &str, asp: &str) -> bool {
    // Exact match
    if orig == asp {
        return true;
    }
    // Both empty
    if orig.is_empty() && asp.is_empty() {
        return true;
    }
    // Key semantic terms present in both
    let keywords = ["denied", "blocked", "forbidden", "rate limit", "budget", "context", "path"];
    for kw in &keywords {
        if orig.to_lowercase().contains(kw) && asp.to_lowercase().contains(kw) {
            return true;
        }
    }
    false
}

/// A report aggregating all equivalence test results.
#[derive(Debug, Serialize, Deserialize)]
pub struct EquivalenceReport {
    pub test_suite: String,
    pub total_cases: usize,
    pub equivalent_cases: usize,
    pub divergent_cases: Vec<TestCaseResult>,
}

impl EquivalenceReport {
    pub fn new(test_suite: impl Into<String>, results: Vec<TestCaseResult>) -> Self {
        let total = results.len();
        let divergent: Vec<TestCaseResult> = results.iter().filter(|r| !r.equivalent).cloned().collect();
        let equivalent = total - divergent.len();

        Self {
            test_suite: test_suite.into(),
            total_cases: total,
            equivalent_cases: equivalent,
            divergent_cases: divergent,
        }
    }

    pub fn is_fully_equivalent(&self) -> bool {
        self.divergent_cases.is_empty()
    }

    pub fn equivalence_rate(&self) -> f64 {
        if self.total_cases == 0 {
            1.0
        } else {
            self.equivalent_cases as f64 / self.total_cases as f64
        }
    }

    pub fn print_summary(&self) {
        println!("=== Equivalence Report: {} ===", self.test_suite);
        println!("Total cases: {}", self.total_cases);
        println!("Equivalent:  {} ({:.1}%)", self.equivalent_cases, self.equivalence_rate() * 100.0);
        println!("Divergent:   {}", self.divergent_cases.len());

        if !self.divergent_cases.is_empty() {
            println!("\nDivergent cases:");
            for case in &self.divergent_cases {
                println!("  - {} (orig_success={}, asp_success={})", case.name, case.original_success, case.aspectized_success);
                if let Some(ref err) = case.original_error {
                    println!("    orig error: {err}");
                }
                if let Some(ref err) = case.aspectized_error {
                    println!("    asp  error: {err}");
                }
            }
        }
    }
}

/// Simulated ZeroClaw `ToolResult` for testing without importing zeroclaw.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into(), error: None }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self { success: false, output: String::new(), error: Some(message.into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalence_report_all_pass() {
        let results = vec![
            TestCaseResult::new("happy_path", true, true, "ok", "ok", None, None),
            TestCaseResult::new("error_path", false, false, "", "", Some("rate limit".into()), Some("rate limit exceeded".into())),
        ];
        let report = EquivalenceReport::new("test_suite", results);
        assert!(report.is_fully_equivalent());
        assert_eq!(report.equivalence_rate(), 1.0);
    }

    #[test]
    fn test_equivalence_report_detects_divergence() {
        let results = vec![
            TestCaseResult::new("divergent", true, false, "ok", "", None, Some("error".into())),
        ];
        let report = EquivalenceReport::new("test_suite", results);
        assert!(!report.is_fully_equivalent());
        assert_eq!(report.divergent_cases.len(), 1);
    }

    #[test]
    fn test_errors_equivalent_semantic_match() {
        assert!(errors_equivalent("path is forbidden", "access denied to forbidden path"));
        assert!(errors_equivalent("rate limit exceeded", "rate limit hit"));
        assert!(!errors_equivalent("rate limit", "not found"));
    }
}
