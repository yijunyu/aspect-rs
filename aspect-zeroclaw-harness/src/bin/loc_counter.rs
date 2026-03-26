//! LOC counter CLI: measures crosscutting concern scattering in a Rust source tree.
//!
//! Usage:
//!   loc_counter --path <PATH> [--format json|table|csv] [--compare-with <PATH>]
//!
//! Example:
//!   loc_counter --path /home/user/re2026/zeroclaw-analysis/src --format table
//!   loc_counter --path ./original --compare-with ./aspectized --format json

use aspect_zeroclaw_harness::loc::counter::LocCounter;
use aspect_zeroclaw_harness::loc::reporter::{LocReport, OutputFormat};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "loc_counter", about = "Measure crosscutting concern LOC in a Rust source tree")]
struct Args {
    /// Path to the Rust source directory to analyze.
    #[arg(long, short)]
    path: PathBuf,

    /// Optional second path to compare against (e.g., aspectized version).
    #[arg(long)]
    compare_with: Option<PathBuf>,

    /// Output format: table (default), json, or csv.
    #[arg(long, default_value = "table")]
    format: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let format = OutputFormat::from_str(&args.format);

    eprintln!("Analyzing: {}", args.path.display());
    let counter = LocCounter::new(&args.path);
    let stats = counter.run()?;
    let report = LocReport::from_stats(args.path.to_string_lossy().as_ref(), stats);

    println!("{}", report.render(format));

    if let Some(compare_path) = args.compare_with {
        eprintln!("\nComparing with: {}", compare_path.display());
        let compare_counter = LocCounter::new(&compare_path);
        let compare_stats = compare_counter.run()?;
        let compare_report = LocReport::from_stats(compare_path.to_string_lossy().as_ref(), compare_stats);

        println!("\n=== COMPARISON: Original vs. Aspectized ===");
        println!("Original crosscutting LOC:    {}", report.crosscutting_loc_total);
        println!("Aspectized crosscutting LOC:  {}", compare_report.crosscutting_loc_total);
        let savings = report.crosscutting_loc_total.saturating_sub(compare_report.crosscutting_loc_total);
        let pct = if report.crosscutting_loc_total > 0 {
            savings as f64 / report.crosscutting_loc_total as f64 * 100.0
        } else {
            0.0
        };
        println!("LOC savings:                  {} ({:.1}%)", savings, pct);
        println!();
        println!("{}", compare_report.render(format));
    }

    Ok(())
}
