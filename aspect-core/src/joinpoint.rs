//! JoinPoint and related types.
//!
//! A joinpoint represents a specific point in program execution where an aspect
//! can be applied, such as a function call.

use crate::error::AspectError;
use std::any::Any;
use std::fmt;

/// Information about a specific point in program execution.
///
/// A `JoinPoint` provides context about where an aspect is being applied,
/// including the function name, module path, and source location.
///
/// # Example
///
/// ```rust
/// use aspect_core::prelude::*;
///
/// let jp = JoinPoint {
///     function_name: "process_data",
///     module_path: "my_app::data",
///     location: Location {
///         file: "src/data.rs",
///         line: 42,
///     },
/// };
///
/// println!("Executing: {} at {}:{}",
///     jp.function_name,
///     jp.location.file,
///     jp.location.line);
/// ```
#[derive(Debug, Clone)]
pub struct JoinPoint {
    /// The name of the function being called
    pub function_name: &'static str,

    /// The module path containing the function
    pub module_path: &'static str,

    /// Source code location information
    pub location: Location,
}

impl JoinPoint {
    /// Creates a new JoinPoint.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aspect_core::prelude::*;
    ///
    /// let jp = JoinPoint::new(
    ///     "my_function",
    ///     "my::module",
    ///     Location { file: "src/lib.rs", line: 100 },
    /// );
    /// ```
    pub fn new(
        function_name: &'static str,
        module_path: &'static str,
        location: Location,
    ) -> Self {
        Self {
            function_name,
            module_path,
            location,
        }
    }

    /// Returns the fully qualified name of the function.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aspect_core::prelude::*;
    /// # let jp = JoinPoint::new("func", "my::mod", Location { file: "a.rs", line: 1 });
    /// assert_eq!(jp.qualified_name(), "my::mod::func");
    /// ```
    pub fn qualified_name(&self) -> String {
        format!("{}::{}", self.module_path, self.function_name)
    }
}

impl fmt::Display for JoinPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}::{}@{}:{}",
            self.module_path, self.function_name, self.location.file, self.location.line
        )
    }
}

/// Source code location information.
///
/// Indicates where in the source code a joinpoint occurs.
#[derive(Debug, Clone, Copy)]
pub struct Location {
    /// The source file path
    pub file: &'static str,

    /// The line number in the source file
    pub line: u32,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

/// A proceeding joinpoint that can be used in "around" advice.
///
/// This type wraps the original function execution and allows aspects to
/// control when (or if) the function runs.
///
/// # Example
///
/// ```rust
/// use aspect_core::prelude::*;
/// use std::any::Any;
///
/// struct TimingAspect;
///
/// impl Aspect for TimingAspect {
///     fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
///         let start = std::time::Instant::now();
///         let result = pjp.proceed()?;
///         let elapsed = start.elapsed();
///         println!("Execution took: {:?}", elapsed);
///         Ok(result)
///     }
/// }
/// ```
pub struct ProceedingJoinPoint<'a> {
    /// The original function to execute
    inner: Box<dyn FnOnce() -> Result<Box<dyn Any>, AspectError> + 'a>,

    /// Context information about this joinpoint
    context: JoinPoint,
}

impl<'a> ProceedingJoinPoint<'a> {
    /// Creates a new ProceedingJoinPoint.
    ///
    /// # Parameters
    ///
    /// - `f`: The original function to execute
    /// - `context`: Information about the joinpoint
    pub fn new<F>(f: F, context: JoinPoint) -> Self
    where
        F: FnOnce() -> Result<Box<dyn Any>, AspectError> + 'a,
    {
        Self {
            inner: Box::new(f),
            context,
        }
    }

    /// Proceeds with the original function execution.
    ///
    /// This consumes the ProceedingJoinPoint and executes the wrapped function.
    ///
    /// # Returns
    ///
    /// The result of the function execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aspect_core::prelude::*;
    /// # use std::any::Any;
    /// # struct MyAspect;
    /// # impl Aspect for MyAspect {
    /// fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    ///     println!("Before proceed");
    ///     let result = pjp.proceed()?;
    ///     println!("After proceed");
    ///     Ok(result)
    /// }
    /// # }
    /// ```
    pub fn proceed(self) -> Result<Box<dyn Any>, AspectError> {
        (self.inner)()
    }

    /// Returns a reference to the joinpoint context.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aspect_core::prelude::*;
    /// # use std::any::Any;
    /// # struct MyAspect;
    /// # impl Aspect for MyAspect {
    /// fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
    ///     let ctx = pjp.context();
    ///     println!("Calling: {}", ctx.function_name);
    ///     pjp.proceed()
    /// }
    /// # }
    /// ```
    pub fn context(&self) -> &JoinPoint {
        &self.context
    }
}

impl<'a> fmt::Debug for ProceedingJoinPoint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProceedingJoinPoint")
            .field("context", &self.context)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joinpoint_qualified_name() {
        let jp = JoinPoint {
            function_name: "my_func",
            module_path: "crate::module",
            location: Location {
                file: "src/lib.rs",
                line: 10,
            },
        };

        assert_eq!(jp.qualified_name(), "crate::module::my_func");
    }

    #[test]
    fn test_joinpoint_display() {
        let jp = JoinPoint {
            function_name: "test",
            module_path: "mod",
            location: Location {
                file: "test.rs",
                line: 42,
            },
        };

        let display = format!("{}", jp);
        assert!(display.contains("test"));
        assert!(display.contains("mod"));
        assert!(display.contains("test.rs"));
        assert!(display.contains("42"));
    }

    #[test]
    fn test_proceeding_joinpoint() {
        let jp = JoinPoint {
            function_name: "test",
            module_path: "test",
            location: Location {
                file: "test.rs",
                line: 1,
            },
        };

        let pjp = ProceedingJoinPoint::new(
            || Ok(Box::new(42) as Box<dyn Any>),
            jp,
        );

        assert_eq!(pjp.context().function_name, "test");

        let result = pjp.proceed().unwrap();
        let value = result.downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 42);
    }
}
