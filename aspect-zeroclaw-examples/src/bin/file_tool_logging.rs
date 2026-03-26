//! Demo: LoggingAspect applied to a ZeroClaw-style file tool.
//!
//! Shows how LoggingAspect from aspect-std provides structured entry/exit logging
//! without any logging code in the business logic.

use aspect_agent::tool_call_audit::{ToolCallAuditAspect, InMemoryAuditStorage, AuditStorage};
use aspect_std::{LoggingAspect, TimingAspect};
use aspect_core::prelude::*;
use std::sync::Arc;

fn main() {
    let storage = Arc::new(InMemoryAuditStorage::default());
    let audit = ToolCallAuditAspect::new(storage.clone());
    let logger = LoggingAspect::new();
    let timer = TimingAspect::new();

    let jp = JoinPoint::new(
        "file_read",
        "tools::file_read",
        Location { file: "file_read.rs", line: 42 },
    );

    println!("=== LoggingAspect + TimingAspect + AuditAspect on file_read ===\n");

    // Simulate: before advice
    logger.before(&jp);
    timer.before(&jp);
    audit.before(&jp);

    println!("  [BUSINESS LOGIC] Reading file: workspace/data.txt");
    let result = "file contents: Hello ZeroClaw";

    // Simulate: after advice
    audit.after(&jp, &result);
    timer.after(&jp, &result);
    logger.after(&jp, &result);

    println!("\nAudit log entries: {}", storage.len());
    let entries = storage.entries();
    for entry in &entries {
        println!("  {:?} — {} ({}ms)", entry.outcome, entry.function_name, entry.duration_ms);
    }

    println!("\nKey insight: {}", "0 lines of logging/timing/audit code in file_read business logic");
    println!("Aspects provide: structured logging, performance timing, audit trail");
    println!("Compare to original: ~25 LOC of crosscutting code removed");
}
