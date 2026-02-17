//! Pointcut expressions for pattern-based aspect weaving.
//!
//! Pointcuts allow you to declaratively specify which functions should have
//! aspects applied based on patterns like function name, visibility, module path, etc.
//!
//! # Example
//!
//! ```rust
//! use aspect_core::pointcut::*;
//!
//! // Match all public functions
//! let pc = Pointcut::parse("execution(pub fn *(..))").unwrap();
//!
//! // Match functions in a specific module
//! let pc = Pointcut::parse("within(crate::api)").unwrap();
//!
//! // Combine with boolean logic
//! let pc = Pointcut::parse("execution(pub fn *(..)) && within(crate::api)").unwrap();
//! ```

pub mod ast;
pub mod matcher;
pub mod parser;
pub mod pattern;

pub use ast::Pointcut;
pub use matcher::{FunctionInfo, Matcher};
pub use parser::parse_pointcut;
pub use pattern::{ExecutionPattern, ModulePattern, NamePattern, Visibility};
