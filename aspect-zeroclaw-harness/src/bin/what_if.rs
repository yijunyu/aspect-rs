//! What-if analysis CLI: shows how aspects would have prevented concern scattering.
//!
//! Usage:
//!   what_if --repo <ZEROCLAW_REPO_PATH> [--format json|table]
//!
//! Example:
//!   what_if --repo /home/user/re2026/zeroclaw-analysis --format table

use aspect_zeroclaw_harness::versions::what_if::WhatIfAnalysis;
use aspect_zeroclaw_harness::loc::reporter::OutputFormat;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "what_if", about = "What-if analysis: how aspects would have prevented ZeroClaw scattering")]
struct Args {
    /// Path to the ZeroClaw repository.
    #[arg(long, short)]
    repo: PathBuf,

    /// Output format: table (default) or json.
    #[arg(long, default_value = "table")]
    format: String,

    /// Use synthetic data from the analysis report instead of git checkout.
    /// Useful when the repo is not available or git checkout is not permitted.
    #[arg(long, default_value = "false")]
    synthetic: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let format = OutputFormat::from_str(&args.format);

    let analysis = WhatIfAnalysis::new(&args.repo);

    eprintln!("Running what-if analysis on ZeroClaw versions...");
    eprintln!("Repo: {}", args.repo.display());

    let report = if args.synthetic {
        eprintln!("Using synthetic data from zeroclaw-analysis-report.md");
        // Force fallback to synthetic data by using a non-existent commit
        let mut synthetic_analysis = WhatIfAnalysis::new(&args.repo)
            .with_versions(aspect_zeroclaw_harness::versions::what_if::known_versions());
        // The analysis will fall back to synthetic data if git checkout fails
        synthetic_analysis.generate_counterfactual_report()?
    } else {
        analysis.generate_counterfactual_report()?
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        _ => {
            println!("=== ZeroClaw What-If Analysis ===\n");
            println!("Versions analyzed: {}", report.summary.versions_analyzed);
            println!("LOC at v1:   {} files, {} LOC",
                report.versions.first().map(|v| v.actual_files).unwrap_or(0),
                report.summary.loc_v1);
            println!("LOC at v{}: {} files, {} LOC",
                report.versions.len(),
                report.versions.last().map(|v| v.actual_files).unwrap_or(0),
                report.summary.loc_v4);
            println!("LOC growth:  {} lines ({:.1}%)",
                report.summary.loc_growth,
                if report.summary.loc_v1 > 0 {
                    report.summary.loc_growth as f64 / report.summary.loc_v1 as f64 * 100.0
                } else { 0.0 });
            println!();
            println!("Crosscutting LOC at v1: {}", report.summary.crosscutting_loc_v1);
            println!("Crosscutting LOC at v{}: {}", report.versions.len(), report.summary.crosscutting_loc_v4);
            println!("Crosscutting growth:    {}", report.summary.crosscutting_growth);
            println!();
            println!("IF aspects had been used from v1:");
            println!("  LOC that would NOT have been scattered: {}", report.total_loc_prevented);
            println!("  As % of total LOC growth:               {:.1}%", report.summary.saving_percentage);
            println!();
            println!("Per-concern LOC prevented:");
            let mut by_concern: Vec<(&String, &usize)> = report.by_concern.iter().collect();
            by_concern.sort_by(|a, b| b.1.cmp(a.1));
            for (concern, saved) in &by_concern {
                if **saved > 0 {
                    println!("  {:<25} {:>6} LOC", concern, saved);
                }
            }
            println!();
            println!("Version trajectory:");
            for vr in &report.versions {
                let xc: usize = vr.concerns.values().map(|c| c.matching_lines).sum();
                println!("  {} ({}): {} files, {} LOC, {} crosscutting LOC",
                    vr.version, &vr.commit[..8], vr.actual_files, vr.actual_loc, xc);
            }
        }
    }

    Ok(())
}
