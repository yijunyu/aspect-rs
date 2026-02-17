//! Pointcut matching logic.

use super::ast::Pointcut;
use super::pattern::{ExecutionPattern, ModulePattern};

/// Information about a function for pointcut matching.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,

    /// Module path (e.g., "crate::api::users")
    pub module_path: String,

    /// Visibility as a string ("pub", "pub(crate)", etc.)
    pub visibility: String,

    /// Return type as a string (simplified)
    pub return_type: Option<String>,
}

impl FunctionInfo {
    /// Create function info for testing.
    pub fn new(
        name: impl Into<String>,
        module_path: impl Into<String>,
        visibility: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            module_path: module_path.into(),
            visibility: visibility.into(),
            return_type: None,
        }
    }

    /// Set the return type.
    pub fn with_return_type(mut self, return_type: impl Into<String>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }
}

/// Matcher trait for evaluating pointcuts against functions.
pub trait Matcher {
    /// Check if this pointcut matches the given function.
    fn matches(&self, function: &FunctionInfo) -> bool;
}

impl Matcher for Pointcut {
    fn matches(&self, function: &FunctionInfo) -> bool {
        match self {
            Pointcut::Execution(pattern) => pattern.matches(function),
            Pointcut::Within(pattern) => pattern.matches(function),
            Pointcut::And(left, right) => left.matches(function) && right.matches(function),
            Pointcut::Or(left, right) => left.matches(function) || right.matches(function),
            Pointcut::Not(inner) => !inner.matches(function),
        }
    }
}

impl Matcher for ExecutionPattern {
    fn matches(&self, function: &FunctionInfo) -> bool {
        // Check visibility
        if let Some(ref vis) = self.visibility {
            if !vis.matches(&function.visibility) {
                return false;
            }
        }

        // Check name
        if !self.name.matches(&function.name) {
            return false;
        }

        // Check return type (simplified string matching for now)
        if let Some(ref expected_return) = self.return_type {
            match &function.return_type {
                Some(actual_return) if actual_return.contains(expected_return) => {}
                _ => return false,
            }
        }

        true
    }
}

impl Matcher for ModulePattern {
    fn matches(&self, function: &FunctionInfo) -> bool {
        self.matches_path(&function.module_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointcut::pattern::{NamePattern, Visibility};

    #[test]
    fn test_execution_pattern_matching() {
        let pattern = ExecutionPattern {
            visibility: Some(Visibility::Public),
            name: NamePattern::Exact("save_user".to_string()),
            return_type: None,
        };

        let func1 = FunctionInfo::new("save_user", "crate::api", "pub");
        assert!(pattern.matches(&func1));

        let func2 = FunctionInfo::new("update_user", "crate::api", "pub");
        assert!(!pattern.matches(&func2));

        let func3 = FunctionInfo::new("save_user", "crate::api", "");
        assert!(!pattern.matches(&func3)); // Not public
    }

    #[test]
    fn test_module_pattern_matching() {
        let pattern = ModulePattern::new("crate::api");

        let func1 = FunctionInfo::new("save", "crate::api", "pub");
        assert!(pattern.matches(&func1));

        let func2 = FunctionInfo::new("save", "crate::api::users", "pub");
        assert!(pattern.matches(&func2));

        let func3 = FunctionInfo::new("save", "crate::internal", "pub");
        assert!(!pattern.matches(&func3));
    }

    #[test]
    fn test_pointcut_and() {
        let exec = ExecutionPattern {
            visibility: Some(Visibility::Public),
            name: NamePattern::Wildcard,
            return_type: None,
        };

        let within = ModulePattern::new("crate::api");

        let pointcut = Pointcut::Execution(exec).and(Pointcut::Within(within));

        let func1 = FunctionInfo::new("save", "crate::api", "pub");
        assert!(pointcut.matches(&func1));

        let func2 = FunctionInfo::new("save", "crate::api", "");
        assert!(!pointcut.matches(&func2)); // Not public

        let func3 = FunctionInfo::new("save", "crate::internal", "pub");
        assert!(!pointcut.matches(&func3)); // Wrong module
    }

    #[test]
    fn test_pointcut_or() {
        let pattern1 = ExecutionPattern::named("save");
        let pattern2 = ExecutionPattern::named("update");

        let pointcut = Pointcut::Execution(pattern1).or(Pointcut::Execution(pattern2));

        let func1 = FunctionInfo::new("save", "crate::api", "pub");
        assert!(pointcut.matches(&func1));

        let func2 = FunctionInfo::new("update", "crate::api", "pub");
        assert!(pointcut.matches(&func2));

        let func3 = FunctionInfo::new("delete", "crate::api", "pub");
        assert!(!pointcut.matches(&func3));
    }

    #[test]
    fn test_pointcut_not() {
        let pattern = ExecutionPattern {
            visibility: Some(Visibility::Public),
            name: NamePattern::Wildcard,
            return_type: None,
        };

        let pointcut = Pointcut::Execution(pattern).not();

        let func1 = FunctionInfo::new("save", "crate::api", "pub");
        assert!(!pointcut.matches(&func1)); // Public functions excluded

        let func2 = FunctionInfo::new("save", "crate::internal", "");
        assert!(pointcut.matches(&func2)); // Private functions match
    }
}
