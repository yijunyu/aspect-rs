//! Aspect trait definition.
//!
//! The `Aspect` trait is the core abstraction for defining cross-cutting concerns
//! in aspect-oriented programming.

use crate::error::AspectError;
use crate::joinpoint::{JoinPoint, ProceedingJoinPoint};
use std::any::Any;

/// The core trait for defining aspects.
///
/// An aspect encapsulates cross-cutting concerns that can be applied to multiple
/// joinpoints in your program. Implement this trait to define custom aspects.
///
/// # Thread Safety
///
/// Aspects must be `Send + Sync` to be used across thread boundaries. This ensures
/// that aspects can be safely shared between threads.
///
/// # Advice Methods
///
/// - **`before`**: Executed before the target function runs
/// - **`after`**: Executed after successful completion
/// - **`after_error`**: Executed when an error occurs
/// - **`around`**: Wraps the entire execution, allowing you to control when/if
///   the target function runs
///
/// # Example
///
/// ```rust
/// use aspect_core::prelude::*;
/// use std::any::Any;
///
/// #[derive(Default)]
/// struct LoggingAspect;
///
/// impl Aspect for LoggingAspect {
///     fn before(&self, ctx: &JoinPoint) {
///         println!("[LOG] Entering function: {}", ctx.function_name);
///     }
///
///     fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
///         println!("[LOG] Exiting function: {}", ctx.function_name);
///     }
///
///     fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
///         eprintln!("[ERROR] Function {} failed: {:?}", ctx.function_name, error);
///     }
/// }
/// ```
pub trait Aspect: Send + Sync {
    /// Advice executed before the target function runs.
    ///
    /// # Parameters
    ///
    /// - `ctx`: Context information about the joinpoint
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aspect_core::prelude::*;
    /// # struct MyAspect;
    /// # impl Aspect for MyAspect {
    /// fn before(&self, ctx: &JoinPoint) {
    ///     println!("About to call: {}", ctx.function_name);
    /// }
    /// # }
    /// ```
    fn before(&self, _ctx: &JoinPoint) {}

    /// Advice executed after the target function completes successfully.
    ///
    /// # Parameters
    ///
    /// - `ctx`: Context information about the joinpoint
    /// - `result`: The return value of the function (as `&dyn Any`)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aspect_core::prelude::*;
    /// # use std::any::Any;
    /// # struct MyAspect;
    /// # impl Aspect for MyAspect {
    /// fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
    ///     println!("Function {} completed", ctx.function_name);
    ///     // You can downcast result to access the actual value
    ///     if let Some(value) = result.downcast_ref::<i32>() {
    ///         println!("Returned value: {}", value);
    ///     }
    /// }
    /// # }
    /// ```
    fn after(&self, _ctx: &JoinPoint, _result: &dyn Any) {}

    /// Advice executed when the target function encounters an error.
    ///
    /// # Parameters
    ///
    /// - `ctx`: Context information about the joinpoint
    /// - `error`: The error that occurred
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aspect_core::prelude::*;
    /// # struct MyAspect;
    /// # impl Aspect for MyAspect {
    /// fn after_error(&self, ctx: &JoinPoint, error: &AspectError) {
    ///     eprintln!("Error in {}: {:?}", ctx.function_name, error);
    ///     // Log to monitoring system, send alerts, etc.
    /// }
    /// # }
    /// ```
    fn after_error(&self, _ctx: &JoinPoint, _error: &AspectError) {}

    /// Advice that wraps the entire target function execution.
    ///
    /// This is the most powerful advice type, allowing you to:
    /// - Control whether the target function runs
    /// - Modify the execution flow
    /// - Implement retry logic, caching, etc.
    ///
    /// The default implementation simply proceeds with the original function.
    ///
    /// # Parameters
    ///
    /// - `pjp`: A proceeding joinpoint that can be used to execute the target function
    ///
    /// # Returns
    ///
    /// The result of the function execution (or a modified result)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aspect_core::prelude::*;
    /// # use std::any::Any;
    /// # struct CachingAspect;
    /// # impl Aspect for CachingAspect {
    /// fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    ///     let function_name = pjp.context().function_name;
    ///     println!("Before: {}", function_name);
    ///
    ///     // Execute the function
    ///     let result = pjp.proceed();
    ///
    ///     println!("After: {}", function_name);
    ///     result
    /// }
    /// # }
    /// ```
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        // Default implementation: call before, proceed, then after/after_error
        let ctx = pjp.context().clone();

        self.before(&ctx);

        let result = pjp.proceed();

        match &result {
            Ok(value) => {
                self.after(&ctx, value.as_ref());
            }
            Err(error) => {
                self.after_error(&ctx, error);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct CountingAspect {
        before_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        after_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl Aspect for CountingAspect {
        fn before(&self, _ctx: &JoinPoint) {
            self.before_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        fn after(&self, _ctx: &JoinPoint, _result: &dyn Any) {
            self.after_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn test_aspect_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<CountingAspect>();
        assert_sync::<CountingAspect>();
    }
}
