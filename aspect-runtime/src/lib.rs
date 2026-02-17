//! # aspect-runtime
//!
//! Runtime utilities for aspect-oriented programming in Rust.
//!
//! This crate provides runtime support for aspects, including:
//! - Global aspect registry for managing aspect-pointcut bindings
//! - Dynamic aspect application based on pointcut patterns
//! - Aspect ordering and composition
//!
//! # Example
//!
//! ```rust
//! use aspect_runtime::registry::global_registry;
//! use aspect_core::pointcut::Pointcut;
//! use std::sync::Arc;
//!
//! // Register an aspect globally
//! let pointcut = Pointcut::parse("execution(pub fn *(..))").unwrap();
//! // global_registry().register(Arc::new(my_aspect), pointcut, 0, Some("logger".into()));
//! ```

pub mod registry;

// Re-export commonly used items
pub use registry::{global_registry, AspectRegistry, RegisteredAspect, GLOBAL_REGISTRY};

// Re-export once_cell for use in generated code
pub use once_cell;
