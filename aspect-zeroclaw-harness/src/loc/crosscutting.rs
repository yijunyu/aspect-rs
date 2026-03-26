//! Crosscutting concern identification and classification.
//!
//! Defines the concern types identified in the ZeroClaw analysis report and
//! provides pattern-matching to classify code as belonging to a concern.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Crosscutting concern categories identified in ZeroClaw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcernType {
    /// Error handling (80.7% of files in ZeroClaw v3)
    ErrorHandling,
    /// Configuration access (63.5%)
    Configuration,
    /// Security/authorization (50.0%)
    SecurityAuth,
    /// Logging (34.4%)
    Logging,
    /// Path/filesystem validation (33.9%)
    PathValidation,
    /// Retry/resilience logic (29.2%)
    RetryResilience,
    /// Cost/token tracking (21.4%)
    CostTracking,
    /// Hook/event dispatch (18.8%)
    HookDispatch,
    /// Rate limiting (14.6%)
    RateLimiting,
    /// Approval/human-in-the-loop (8.9%)
    ApprovalHitl,
    /// Telemetry/metrics (10.4%)
    TelemetryMetrics,
}

impl ConcernType {
    pub fn all() -> &'static [ConcernType] {
        &[
            ConcernType::ErrorHandling,
            ConcernType::Configuration,
            ConcernType::SecurityAuth,
            ConcernType::Logging,
            ConcernType::PathValidation,
            ConcernType::RetryResilience,
            ConcernType::CostTracking,
            ConcernType::HookDispatch,
            ConcernType::RateLimiting,
            ConcernType::ApprovalHitl,
            ConcernType::TelemetryMetrics,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ErrorHandling => "error_handling",
            Self::Configuration => "configuration",
            Self::SecurityAuth => "security_auth",
            Self::Logging => "logging",
            Self::PathValidation => "path_validation",
            Self::RetryResilience => "retry_resilience",
            Self::CostTracking => "cost_tracking",
            Self::HookDispatch => "hook_dispatch",
            Self::RateLimiting => "rate_limiting",
            Self::ApprovalHitl => "approval_hitl",
            Self::TelemetryMetrics => "telemetry_metrics",
        }
    }

    /// ZeroClaw v3 baseline scattering score (files_affected / 192).
    pub fn baseline_scattering(&self) -> f64 {
        match self {
            Self::ErrorHandling => 0.807,
            Self::Configuration => 0.635,
            Self::SecurityAuth => 0.500,
            Self::Logging => 0.344,
            Self::PathValidation => 0.339,
            Self::RetryResilience => 0.292,
            Self::CostTracking => 0.214,
            Self::HookDispatch => 0.188,
            Self::RateLimiting => 0.146,
            Self::TelemetryMetrics => 0.104,
            Self::ApprovalHitl => 0.089,
        }
    }
}

/// Statistics for a single crosscutting concern in a codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcernStats {
    pub concern: ConcernType,
    /// Number of files containing at least one match for this concern.
    pub files_affected: usize,
    /// Total files in the analyzed codebase.
    pub total_files: usize,
    /// Total lines matching this concern's patterns.
    pub matching_lines: usize,
    /// Total lines in the codebase.
    pub total_lines: usize,
}

impl ConcernStats {
    /// Fraction of files containing this concern (scattering score).
    pub fn scattering_score(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            self.files_affected as f64 / self.total_files as f64
        }
    }

    /// Fraction of total LOC attributable to this concern (tangling degree).
    pub fn tangling_degree(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            self.matching_lines as f64 / self.total_lines as f64
        }
    }

    /// Estimated LOC savings if this concern were centralized via an aspect.
    /// Approximation: lines_in_business_logic = matching_lines - (one canonical implementation)
    pub fn estimated_loc_savings(&self) -> usize {
        // Assume each distinct concern needs ~50 LOC to implement once as an aspect.
        // All remaining scattered LOC would be eliminated.
        const ASPECT_IMPLEMENTATION_LOC: usize = 50;
        self.matching_lines.saturating_sub(ASPECT_IMPLEMENTATION_LOC)
    }
}

/// Patterns used to detect each concern in Rust source code.
pub struct ConcernPatterns {
    pub concern: ConcernType,
    /// Line-level regex patterns. A file is "affected" if any line matches.
    pub line_patterns: Vec<Regex>,
}

impl ConcernPatterns {
    pub fn for_type(concern: ConcernType) -> Self {
        let patterns: Vec<&str> = match concern {
            ConcernType::ErrorHandling => vec![
                r"anyhow::anyhow!",
                r"anyhow::bail!",
                r"\.map_err\(",
                r"\.context\(",
                r"Result<",
                r"Err\(",
            ],
            ConcernType::Configuration => vec![
                r"\bConfig\b",
                r"\bconfig\b",
            ],
            ConcernType::SecurityAuth => vec![
                r"is_rate_limited",
                r"is_path_allowed",
                r"forbidden",
                r"SecurityPolicy",
                r"check_authorization",
                r"pairing",
                r"auth_token",
                r"secret",
            ],
            ConcernType::Logging => vec![
                r"tracing::info!",
                r"tracing::debug!",
                r"tracing::warn!",
                r"tracing::error!",
                r"tracing::trace!",
                r"log::info!",
                r"log::debug!",
                r"log::warn!",
                r"log::error!",
            ],
            ConcernType::PathValidation => vec![
                r"canonicalize",
                r"starts_with.*workspace",
                r"is_path_allowed",
                r"forbidden_path",
            ],
            ConcernType::RetryResilience => vec![
                r"\bretry\b",
                r"\bbackoff\b",
                r"timeout",
                r"CircuitBreaker",
            ],
            ConcernType::CostTracking => vec![
                r"token_count",
                r"tokens_used",
                r"check_budget",
                r"record_cost",
                r"CostTracker",
                r"budget",
            ],
            ConcernType::HookDispatch => vec![
                r"\bhook\b",
                r"\bHook\b",
                r"emit\(",
                r"dispatch\(",
            ],
            ConcernType::RateLimiting => vec![
                r"rate_limit",
                r"RateLimit",
                r"throttl",
                r"ActionTracker",
                r"SlidingWindow",
            ],
            ConcernType::ApprovalHitl => vec![
                r"approval",
                r"approve",
                r"human_review",
                r"await_approval",
                r"needs_approval",
            ],
            ConcernType::TelemetryMetrics => vec![
                r"\bmetric\b",
                r"\bcounter\b",
                r"\bhistogram\b",
                r"\bgauge\b",
                r"prometheus",
                r"opentelemetry",
            ],
        };

        let compiled = patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self { concern, line_patterns: compiled }
    }

    /// Check if a line matches this concern's patterns.
    pub fn matches_line(&self, line: &str) -> bool {
        self.line_patterns.iter().any(|p| p.is_match(line))
    }
}

/// Cache of compiled patterns for all concern types.
static ALL_PATTERNS: OnceLock<HashMap<ConcernType, ConcernPatterns>> = OnceLock::new();

pub fn get_patterns() -> &'static HashMap<ConcernType, ConcernPatterns> {
    ALL_PATTERNS.get_or_init(|| {
        ConcernType::all()
            .iter()
            .map(|&ct| (ct, ConcernPatterns::for_type(ct)))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detects_rate_limiting() {
        let patterns = ConcernPatterns::for_type(ConcernType::RateLimiting);
        assert!(patterns.matches_line("if self.security.is_rate_limited() {"));
        assert!(patterns.matches_line("let limiter = SlidingWindowRateLimiter::new(10);"));
        assert!(!patterns.matches_line("fn normal_function() {}"));
    }

    #[test]
    fn test_pattern_detects_path_validation() {
        let patterns = ConcernPatterns::for_type(ConcernType::PathValidation);
        assert!(patterns.matches_line("let resolved = tokio::fs::canonicalize(&path).await?;"));
        assert!(!patterns.matches_line("let x = 42;"));
    }

    #[test]
    fn test_pattern_detects_logging() {
        let patterns = ConcernPatterns::for_type(ConcernType::Logging);
        assert!(patterns.matches_line("tracing::info!(\"Starting tool execution\");"));
        assert!(patterns.matches_line("tracing::warn!(\"Rate limit approaching\");"));
        assert!(!patterns.matches_line("let value = compute();"));
    }

    #[test]
    fn test_scattering_score() {
        let stats = ConcernStats {
            concern: ConcernType::ErrorHandling,
            files_affected: 155,
            total_files: 192,
            matching_lines: 1000,
            total_lines: 129040,
        };
        let score = stats.scattering_score();
        assert!((score - 0.807).abs() < 0.01);
    }

    #[test]
    fn test_all_concerns_have_patterns() {
        for concern in ConcernType::all() {
            let patterns = ConcernPatterns::for_type(*concern);
            assert!(!patterns.line_patterns.is_empty(), "No patterns for {:?}", concern);
        }
    }
}
