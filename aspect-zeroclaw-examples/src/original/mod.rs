//! Original ZeroClaw tool implementations with scattered crosscutting concerns.
//!
//! These show the problem: security checks, rate limiting, logging, and path
//! validation are all inlined into each tool's execute() method.

pub mod file_read_original;
