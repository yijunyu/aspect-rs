//! Pointcut matching engine for aspect weaving.
//!
//! This module matches FunctionMetadata against pointcut expressions to
//! determine which aspects should be applied to which functions.

use crate::types::{FunctionMetadata, MatchedFunction, Visibility};
use std::collections::HashMap;

/// Aspect registry entry.
#[derive(Debug, Clone)]
pub struct RegisteredAspect {
    /// Aspect type name
    pub aspect_name: String,

    /// Pointcut expression
    pub pointcut: String,

    /// Advice type (before, after, around)
    pub advice_type: AdviceType,

    /// Priority (higher = runs first)
    pub priority: i32,
}

/// Type of advice to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdviceType {
    /// Run before function execution
    Before,
    /// Run after function execution
    After,
    /// Wrap function execution
    Around,
}

/// Pointcut matching engine.
pub struct PointcutMatcher {
    /// Registered aspects to match against
    aspects: Vec<RegisteredAspect>,
}

impl PointcutMatcher {
    /// Create a new pointcut matcher.
    pub fn new() -> Self {
        Self {
            aspects: Vec::new(),
        }
    }

    /// Register an aspect with a pointcut.
    pub fn register(&mut self, aspect: RegisteredAspect) {
        self.aspects.push(aspect);
    }

    /// Match all registered aspects against a function.
    ///
    /// Returns all aspects that match the function, sorted by priority.
    pub fn match_function(&self, function: &FunctionMetadata) -> Vec<MatchedFunction> {
        let mut matches = Vec::new();

        for aspect in &self.aspects {
            if self.matches_pointcut(function, &aspect.pointcut) {
                matches.push(MatchedFunction {
                    function: function.clone(),
                    aspect: aspect.aspect_name.clone(),
                    pointcut: aspect.pointcut.clone(),
                });
            }
        }

        // Sort by priority (higher first)
        matches.sort_by(|a, b| {
            let a_priority = self.get_priority(&a.aspect);
            let b_priority = self.get_priority(&b.aspect);
            b_priority.cmp(&a_priority)
        });

        matches
    }

    /// Match a function against a pointcut expression.
    fn matches_pointcut(&self, function: &FunctionMetadata, pointcut: &str) -> bool {
        // Parse and evaluate pointcut
        match parse_pointcut(pointcut) {
            Ok(expr) => self.evaluate_pointcut(&expr, function),
            Err(_) => false, // Invalid pointcut doesn't match
        }
    }

    /// Evaluate a parsed pointcut expression.
    fn evaluate_pointcut(&self, expr: &PointcutExpr, function: &FunctionMetadata) -> bool {
        match expr {
            PointcutExpr::Execution(pattern) => self.matches_execution(function, pattern),
            PointcutExpr::Within(pattern) => self.matches_within(function, pattern),
            PointcutExpr::Name(pattern) => self.matches_name(function, pattern),
            PointcutExpr::And(left, right) => {
                self.evaluate_pointcut(left, function) && self.evaluate_pointcut(right, function)
            }
            PointcutExpr::Or(left, right) => {
                self.evaluate_pointcut(left, function) || self.evaluate_pointcut(right, function)
            }
            PointcutExpr::Not(inner) => !self.evaluate_pointcut(inner, function),
        }
    }

    /// Match execution pattern.
    fn matches_execution(&self, function: &FunctionMetadata, pattern: &str) -> bool {
        // Parse pattern: "pub fn *(..)" -> visibility="pub", name="*"
        let parts: Vec<&str> = pattern.split_whitespace().collect();

        // Check visibility
        if parts.contains(&"pub") && !function.is_public() {
            return false;
        }

        // Check function keyword
        if !parts.contains(&"fn") {
            return false;
        }

        // Check name pattern
        if let Some(name_part) = parts.iter().find(|p| p.contains('(') || p.ends_with('*')) {
            let name = name_part.trim_end_matches("(..)");
            if !function.matches_name_pattern(name) {
                return false;
            }
        }

        true
    }

    /// Match within pattern (module path).
    fn matches_within(&self, function: &FunctionMetadata, pattern: &str) -> bool {
        function.is_in_module(pattern)
    }

    /// Match name pattern.
    fn matches_name(&self, function: &FunctionMetadata, pattern: &str) -> bool {
        function.matches_name_pattern(pattern)
    }

    /// Get priority for an aspect.
    fn get_priority(&self, aspect_name: &str) -> i32 {
        self.aspects
            .iter()
            .find(|a| a.aspect_name == aspect_name)
            .map(|a| a.priority)
            .unwrap_or(0)
    }
}

impl Default for PointcutMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed pointcut expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PointcutExpr {
    /// execution(pattern)
    Execution(String),
    /// within(pattern)
    Within(String),
    /// name(pattern)
    Name(String),
    /// expr1 && expr2
    And(Box<PointcutExpr>, Box<PointcutExpr>),
    /// expr1 || expr2
    Or(Box<PointcutExpr>, Box<PointcutExpr>),
    /// !expr
    Not(Box<PointcutExpr>),
}

/// Parse a pointcut expression.
///
/// Supports:
/// - `execution(pub fn *(..))`
/// - `within(crate::module)`
/// - `name("fetch_*")`
/// - `expr1 && expr2`
/// - `expr1 || expr2`
/// - `!expr`
pub fn parse_pointcut(input: &str) -> Result<PointcutExpr, String> {
    let input = input.trim();

    // Handle NOT
    if input.starts_with('!') {
        let inner = parse_pointcut(&input[1..].trim())?;
        return Ok(PointcutExpr::Not(Box::new(inner)));
    }

    // Handle AND/OR
    if let Some(pos) = find_operator(input, "&&") {
        let left = parse_pointcut(&input[..pos].trim())?;
        let right = parse_pointcut(&input[pos + 2..].trim())?;
        return Ok(PointcutExpr::And(Box::new(left), Box::new(right)));
    }

    if let Some(pos) = find_operator(input, "||") {
        let left = parse_pointcut(&input[..pos].trim())?;
        let right = parse_pointcut(&input[pos + 2..].trim())?;
        return Ok(PointcutExpr::Or(Box::new(left), Box::new(right)));
    }

    // Handle parentheses
    if input.starts_with('(') && input.ends_with(')') {
        return parse_pointcut(&input[1..input.len() - 1]);
    }

    // Handle primitive patterns
    if input.starts_with("execution(") {
        let pattern = extract_pattern(input, "execution")?;
        Ok(PointcutExpr::Execution(pattern))
    } else if input.starts_with("within(") {
        let pattern = extract_pattern(input, "within")?;
        Ok(PointcutExpr::Within(pattern))
    } else if input.starts_with("name(") {
        let pattern = extract_pattern(input, "name")?;
        Ok(PointcutExpr::Name(pattern))
    } else {
        Err(format!("Unknown pointcut pattern: {}", input))
    }
}

/// Find operator position outside of parentheses.
fn find_operator(input: &str, operator: &str) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<char> = input.chars().collect();
    let op_chars: Vec<char> = operator.chars().collect();

    for i in 0..chars.len() {
        if chars[i] == '(' {
            depth += 1;
        } else if chars[i] == ')' {
            depth -= 1;
        } else if depth == 0 && i + op_chars.len() <= chars.len() {
            let slice: String = chars[i..i + op_chars.len()].iter().collect();
            if slice == operator {
                return Some(i);
            }
        }
    }

    None
}

/// Extract pattern from function call syntax.
fn extract_pattern(input: &str, prefix: &str) -> Result<String, String> {
    if !input.starts_with(prefix) || !input.contains('(') {
        return Err(format!("Invalid {} pattern", prefix));
    }

    let start = input.find('(').unwrap() + 1;
    let end = input.rfind(')').ok_or("Missing closing parenthesis")?;

    if start >= end {
        return Err("Empty pattern".to_string());
    }

    Ok(input[start..end].trim().trim_matches('"').to_string())
}

/// Load registered aspects from the global registry.
///
/// This would integrate with aspect-runtime's AspectRegistry.
///
/// # Example (Conceptual)
///
/// ```ignore
/// use aspect_runtime::AspectRegistry;
///
/// pub fn load_from_registry() -> Vec<RegisteredAspect> {
///     let registry = AspectRegistry::global();
///     registry.list_aspects()
///         .into_iter()
///         .map(|a| RegisteredAspect {
///             aspect_name: a.type_name,
///             pointcut: a.pointcut,
///             advice_type: a.advice_type,
///             priority: a.priority,
///         })
///         .collect()
/// }
/// ```
pub fn load_from_registry() -> Vec<RegisteredAspect> {
    // Placeholder - would load from aspect-runtime
    Vec::new()
}

/// Match all functions against all registered aspects.
pub fn match_all(
    functions: &[FunctionMetadata],
    aspects: &[RegisteredAspect],
) -> HashMap<String, Vec<MatchedFunction>> {
    let mut matcher = PointcutMatcher::new();

    for aspect in aspects {
        matcher.register(aspect.clone());
    }

    let mut results = HashMap::new();

    for function in functions {
        let matches = matcher.match_function(function);
        if !matches.is_empty() {
            results.insert(function.name.clone(), matches);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceLocation;

    fn sample_function(name: &str, visibility: Visibility, module: &str) -> FunctionMetadata {
        // Extract simple name (last component after ::)
        let simple_name = name.rsplit("::").next().unwrap_or(name).to_string();

        FunctionMetadata {
            name: name.to_string(),
            simple_name,
            module_path: module.to_string(),
            visibility,
            is_async: false,
            generics: vec![],
            return_type: "()".to_string(),
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 1,
                column: 1,
            },
        }
    }

    #[test]
    fn test_parse_execution() {
        let expr = parse_pointcut("execution(pub fn *(..))").unwrap();
        assert!(matches!(expr, PointcutExpr::Execution(_)));
    }

    #[test]
    fn test_parse_within() {
        let expr = parse_pointcut("within(crate::api)").unwrap();
        assert!(matches!(expr, PointcutExpr::Within(_)));
    }

    #[test]
    fn test_parse_and() {
        let expr = parse_pointcut("execution(pub fn *(..)) && within(crate::api)").unwrap();
        assert!(matches!(expr, PointcutExpr::And(_, _)));
    }

    #[test]
    fn test_parse_not() {
        let expr = parse_pointcut("!within(crate::internal)").unwrap();
        assert!(matches!(expr, PointcutExpr::Not(_)));
    }

    #[test]
    fn test_match_execution() {
        let mut matcher = PointcutMatcher::new();
        matcher.register(RegisteredAspect {
            aspect_name: "Logger".to_string(),
            pointcut: "execution(pub fn *(..))".to_string(),
            advice_type: AdviceType::Before,
            priority: 0,
        });

        let public_fn = sample_function("test_fn", Visibility::Public, "crate");
        let private_fn = sample_function("test_fn", Visibility::Private, "crate");

        let matches = matcher.match_function(&public_fn);
        assert_eq!(matches.len(), 1);

        let matches = matcher.match_function(&private_fn);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_match_within() {
        let mut matcher = PointcutMatcher::new();
        matcher.register(RegisteredAspect {
            aspect_name: "ApiLogger".to_string(),
            pointcut: "within(crate::api)".to_string(),
            advice_type: AdviceType::Before,
            priority: 0,
        });

        let api_fn = sample_function("fetch", Visibility::Public, "crate::api");
        let other_fn = sample_function("fetch", Visibility::Public, "crate::internal");

        let matches = matcher.match_function(&api_fn);
        assert_eq!(matches.len(), 1);

        let matches = matcher.match_function(&other_fn);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_match_combined() {
        let mut matcher = PointcutMatcher::new();
        matcher.register(RegisteredAspect {
            aspect_name: "Combined".to_string(),
            pointcut: "execution(pub fn *(..)) && within(crate::api)".to_string(),
            advice_type: AdviceType::Around,
            priority: 0,
        });

        let api_public = sample_function("fetch", Visibility::Public, "crate::api");
        let api_private = sample_function("fetch", Visibility::Private, "crate::api");
        let other_public = sample_function("fetch", Visibility::Public, "crate::other");

        assert_eq!(matcher.match_function(&api_public).len(), 1);
        assert_eq!(matcher.match_function(&api_private).len(), 0);
        assert_eq!(matcher.match_function(&other_public).len(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut matcher = PointcutMatcher::new();
        matcher.register(RegisteredAspect {
            aspect_name: "Low".to_string(),
            pointcut: "execution(pub fn *(..))".to_string(),
            advice_type: AdviceType::Before,
            priority: 1,
        });
        matcher.register(RegisteredAspect {
            aspect_name: "High".to_string(),
            pointcut: "execution(pub fn *(..))".to_string(),
            advice_type: AdviceType::Before,
            priority: 10,
        });

        let func = sample_function("test", Visibility::Public, "crate");
        let matches = matcher.match_function(&func);

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].aspect, "High"); // Higher priority first
        assert_eq!(matches[1].aspect, "Low");
    }

    #[test]
    fn test_extract_pattern() {
        let pattern = extract_pattern("execution(pub fn *(..))", "execution").unwrap();
        assert_eq!(pattern, "pub fn *(..)");

        let pattern = extract_pattern("within(crate::api)", "within").unwrap();
        assert_eq!(pattern, "crate::api");
    }

    #[test]
    fn test_find_operator() {
        let input = "a && b";
        assert_eq!(find_operator(input, "&&"), Some(2));

        let input = "(a && b) && c";
        assert_eq!(find_operator(input, "&&"), Some(9));

        let input = "execution(pub && fn) && within(a)";
        assert_eq!(find_operator(input, "&&"), Some(21));
    }
}
