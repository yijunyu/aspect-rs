//! # aspect-zeroclaw-harness
//!
//! Test harness measuring 4 dimensions of aspect-rs applied to ZeroClaw:
//!
//! - **(A) Correctness** (`correctness`): behavioral equivalence tests
//! - **(B) LOC savings** (`loc`): AST-based crosscutting concern counter
//! - **(C) What-if analysis** (`versions`): v1–v4 scattering trajectory
//! - **(D) v5 integration** (`v5`): live tests against ZeroClaw current HEAD

pub mod correctness;
pub mod loc;
pub mod versions;
pub mod v5;
