//! # aspect-macros
//!
//! Procedural macros for aspect-oriented programming in Rust.
//!
//! This crate provides the `#[aspect]` attribute macro that enables aspect weaving
//! at compile time.

use proc_macro::TokenStream;
use syn::{parse_macro_input, Expr, ItemFn};

mod advice_macro;
mod aspect_attr;
mod codegen;
mod parsing;

/// Applies an aspect to a function.
///
/// # Example
///
/// ```ignore
/// use aspect_macros::aspect;
/// use aspect_core::prelude::*;
///
/// #[derive(Default)]
/// struct Logger;
///
/// impl Aspect for Logger {
///     fn before(&self, ctx: &JoinPoint) {
///         println!("[ENTRY] {}", ctx.function_name);
///     }
/// }
///
/// #[aspect(Logger)]
/// fn my_function(x: i32) -> i32 {
///     x * 2
/// }
/// ```
#[proc_macro_attribute]
pub fn aspect(attr: TokenStream, item: TokenStream) -> TokenStream {
    let aspect_expr = parse_macro_input!(attr as Expr);
    let func = parse_macro_input!(item as ItemFn);

    aspect_attr::transform(aspect_expr, func)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Registers an aspect with a pointcut pattern for declarative aspect application.
///
/// # Example
///
/// ```ignore
/// use aspect_macros::advice;
/// use aspect_core::prelude::*;
///
/// #[advice(
///     pointcut = "execution(pub fn *(..)) && within(crate::api)",
///     advice = "around",
///     order = 10
/// )]
/// fn api_logger(pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
///     println!("API call: {}", pjp.context().function_name);
///     pjp.proceed()
/// }
/// ```
#[proc_macro_attribute]
pub fn advice(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as advice_macro::AdviceArgs);
    let func = parse_macro_input!(item as ItemFn);

    advice_macro::transform(args, func)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
