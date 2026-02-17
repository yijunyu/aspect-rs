//! Type definitions for compiler metadata extraction.

/// Visibility level of a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Public (pub)
    Public,
    /// Public within crate (pub(crate))
    Crate,
    /// Public within module (pub(in path))
    Restricted,
    /// Private (no pub)
    Private,
}

/// Generic parameter information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParam {
    /// Parameter name (e.g., "T")
    pub name: String,
    /// Trait bounds (e.g., ["Clone", "Debug"])
    pub bounds: Vec<String>,
}

/// Source code location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
}

/// Complete metadata for a function extracted from MIR.
///
/// This contains all information needed for pointcut matching and
/// aspect weaving.
#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    /// Fully qualified function name (e.g., "my_crate::api::get_user")
    pub name: String,

    /// Simple function name without path (e.g., "get_user")
    pub simple_name: String,

    /// Module path (e.g., "my_crate::api")
    pub module_path: String,

    /// Visibility level
    pub visibility: Visibility,

    /// Whether the function is async
    pub is_async: bool,

    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Return type (as string for now)
    pub return_type: String,

    /// Source location
    pub location: SourceLocation,
}

impl FunctionMetadata {
    /// Check if this function matches a simple name pattern.
    ///
    /// Supports wildcards: "fetch_*" matches "fetch_user", "fetch_data", etc.
    pub fn matches_name_pattern(&self, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // Extract the simple function name (last component after ::)
        let simple_name = self.name.rsplit("::").next().unwrap_or(&self.name);

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            simple_name.starts_with(prefix) || self.name.contains(&format!("::{}", prefix))
        } else {
            simple_name == pattern || self.name.ends_with(&format!("::{}", pattern))
        }
    }

    /// Check if this function is in a specific module.
    pub fn is_in_module(&self, module: &str) -> bool {
        self.module_path == module || self.module_path.starts_with(&format!("{}::", module))
    }

    /// Check if this function is public (any form of pub).
    pub fn is_public(&self) -> bool {
        matches!(
            self.visibility,
            Visibility::Public | Visibility::Crate | Visibility::Restricted
        )
    }
}

/// Result of pointcut matching.
#[derive(Debug, Clone)]
pub struct MatchedFunction {
    /// The function metadata
    pub function: FunctionMetadata,

    /// Aspect to apply (type name)
    pub aspect: String,

    /// Pointcut expression that matched
    pub pointcut: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_function() -> FunctionMetadata {
        FunctionMetadata {
            name: "my_crate::api::fetch_user".to_string(),
            simple_name: "fetch_user".to_string(),
            module_path: "my_crate::api".to_string(),
            visibility: Visibility::Public,
            is_async: false,
            generics: vec![],
            return_type: "User".to_string(),
            location: SourceLocation {
                file: "src/api.rs".to_string(),
                line: 42,
                column: 1,
            },
        }
    }

    #[test]
    fn test_name_pattern_wildcard() {
        let func = sample_function();
        assert!(func.matches_name_pattern("*"));
        assert!(func.matches_name_pattern("fetch_*"));
        assert!(func.matches_name_pattern("fetch_user"));
        assert!(!func.matches_name_pattern("create_*"));
    }

    #[test]
    fn test_module_matching() {
        let func = sample_function();
        assert!(func.is_in_module("my_crate::api"));
        assert!(func.is_in_module("my_crate"));
        assert!(!func.is_in_module("other_crate"));
    }

    #[test]
    fn test_visibility() {
        let func = sample_function();
        assert!(func.is_public());

        let private_func = FunctionMetadata {
            visibility: Visibility::Private,
            ..func
        };
        assert!(!private_func.is_public());
    }

    #[test]
    fn test_generic_params() {
        let generic_param = GenericParam {
            name: "T".to_string(),
            bounds: vec!["Clone".to_string(), "Debug".to_string()],
        };

        assert_eq!(generic_param.name, "T");
        assert_eq!(generic_param.bounds.len(), 2);
    }
}
