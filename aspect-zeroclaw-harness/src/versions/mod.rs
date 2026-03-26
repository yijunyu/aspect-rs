//! (C) Version analysis: v1–v4 what-if analysis.
//!
//! Checks out each pinned ZeroClaw commit, runs the LOC counter, and produces
//! a trajectory report showing how crosscutting concerns grew over time and
//! what LOC savings aspects would have provided at each version.

pub mod what_if;
pub use what_if::{VersionReport, WhatIfAnalysis, CounterfactualReport};
