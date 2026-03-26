//! Aspectized ZeroClaw tool implementations with concerns extracted.
//!
//! These show the solution: crosscutting concerns are declared via `#[aspect(...)]`
//! and the execute() method contains only pure business logic.

pub mod file_read_aspectized;
