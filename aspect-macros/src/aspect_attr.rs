//! Main transformation logic for the #[aspect] attribute macro.

use proc_macro2::TokenStream;
use syn::{Expr, ItemFn, Result};

use crate::codegen::generate_aspect_wrapper;
use crate::parsing::AspectInfo;

/// Transforms a function by applying aspect weaving.
///
/// This is the main entry point for the `#[aspect]` macro transformation.
pub fn transform(aspect_expr: Expr, func: ItemFn) -> Result<TokenStream> {
    // Parse the aspect information
    let aspect_info = AspectInfo::parse(aspect_expr)?;

    // Generate the wrapped code
    let output = generate_aspect_wrapper(&aspect_info, &func);

    Ok(output)
}
