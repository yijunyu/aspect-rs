//! (B) LOC measurement: crosscutting concern counter.
//!
//! Uses AST-based analysis (via `syn`) to count lines of code attributable to each
//! identified crosscutting concern in ZeroClaw. More accurate than grep because it
//! counts entire code blocks rather than just keyword matches.

pub mod counter;
pub mod crosscutting;
pub mod reporter;

pub use counter::LocCounter;
pub use crosscutting::{ConcernStats, ConcernType};
pub use reporter::LocReport;
