//! # aspect-core
//!
//! Core abstractions for aspect-oriented programming in Rust.
//!
//! This crate provides the fundamental traits and types for building and using
//! aspects in Rust. Aspects help modularize cross-cutting concerns like logging,
//! performance monitoring, caching, security, and more.
//!
//! ## Core Concepts
//!
//! - **Aspect**: A module that encapsulates a cross-cutting concern
//! - **JoinPoint**: A point in program execution where an aspect can be applied
//! - **Advice**: The action taken by an aspect at a joinpoint
//!
//! ## Example
//!
//! ```rust
//! use aspect_core::prelude::*;
//! use std::any::Any;
//!
//! // Define an aspect
//! #[derive(Default)]
//! struct Logger;
//!
//! impl Aspect for Logger {
//!     fn before(&self, ctx: &JoinPoint) {
//!         println!("[ENTRY] {}", ctx.function_name);
//!     }
//!
//!     fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
//!         println!("[EXIT] {}", ctx.function_name);
//!     }
//! }
//! ```
//!
//! ## Advice Types
//!
//! - **before**: Runs before the target function
//! - **after**: Runs after successful execution
//! - **after_error**: Runs when an error occurs
//! - **around**: Wraps the entire function execution
//!
//! ## Thread Safety
//!
//! All aspects must implement `Send + Sync` to be used across thread boundaries.

#![deny(missing_docs)]

pub mod aspect;
pub mod error;
pub mod joinpoint;
pub mod pointcut;

// Re-export core types
pub use aspect::Aspect;
pub use error::AspectError;
pub use joinpoint::{JoinPoint, Location, ProceedingJoinPoint};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::aspect::Aspect;
    pub use crate::joinpoint::{JoinPoint, Location, ProceedingJoinPoint};
    pub use crate::error::AspectError;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct TestAspect {
        called: Arc<Mutex<Vec<String>>>,
    }

    impl Default for TestAspect {
        fn default() -> Self {
            Self {
                called: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Aspect for TestAspect {
        fn before(&self, ctx: &JoinPoint) {
            self.called
                .lock()
                .unwrap()
                .push(format!("before:{}", ctx.function_name));
        }

        fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
            self.called
                .lock()
                .unwrap()
                .push(format!("after:{}", ctx.function_name));
        }
    }

    #[test]
    fn test_aspect_trait() {
        let aspect = TestAspect::default();
        let ctx = JoinPoint {
            function_name: "test_function",
            module_path: "test::module",
            location: Location {
                file: "test.rs",
                line: 42,
            },
        };

        aspect.before(&ctx);
        aspect.after(&ctx, &42);

        let calls = aspect.called.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], "before:test_function");
        assert_eq!(calls[1], "after:test_function");
    }

    #[test]
    fn test_joinpoint_creation() {
        let jp = JoinPoint {
            function_name: "my_function",
            module_path: "my::module",
            location: Location {
                file: "src/lib.rs",
                line: 100,
            },
        };

        assert_eq!(jp.function_name, "my_function");
        assert_eq!(jp.module_path, "my::module");
        assert_eq!(jp.location.file, "src/lib.rs");
        assert_eq!(jp.location.line, 100);
    }
}
