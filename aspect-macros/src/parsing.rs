//! Parsing utilities for aspect macro attributes.

use syn::{Expr, Result};

/// Information about the aspect to apply.
pub struct AspectInfo {
    /// The expression that evaluates to the aspect instance
    pub aspect_expr: Expr,
}

impl AspectInfo {
    /// Parse aspect information from the attribute syntax.
    pub fn parse(aspect_expr: Expr) -> Result<Self> {
        Ok(Self { aspect_expr })
    }
}
