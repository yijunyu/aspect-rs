//! LOC report generation: outputs analysis results in JSON, table, or CSV format.

use crate::loc::crosscutting::{ConcernStats, ConcernType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output format for the LOC report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "csv" => Self::Csv,
            _ => Self::Table,
        }
    }
}

/// Complete LOC analysis report for a codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocReport {
    /// Path analyzed.
    pub path: String,
    /// Total files analyzed.
    pub total_files: usize,
    /// Total lines of code.
    pub total_loc: usize,
    /// Statistics per concern type.
    pub concerns: HashMap<String, ConcernMetrics>,
    /// Total LOC attributable to crosscutting concerns.
    pub crosscutting_loc_total: usize,
    /// Estimated LOC savings if all concerns were centralized as aspects.
    pub estimated_loc_savings: usize,
    /// Proportion of crosscutting LOC vs. total.
    pub crosscutting_fraction: f64,
}

/// Metrics for a single concern in the report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcernMetrics {
    pub files_affected: usize,
    pub total_files: usize,
    pub scattering_score: f64,
    pub matching_lines: usize,
    pub tangling_degree: f64,
    pub estimated_loc_savings: usize,
    /// ZeroClaw v3 baseline scattering (for comparison).
    pub baseline_scattering: f64,
    /// How this analysis compares to baseline (positive = more scattered).
    pub scattering_delta: f64,
}

impl LocReport {
    pub fn from_stats(path: impl Into<String>, stats: HashMap<ConcernType, ConcernStats>) -> Self {
        let total_files = stats.values().next().map(|s| s.total_files).unwrap_or(0);
        let total_loc = stats.values().next().map(|s| s.total_lines).unwrap_or(0);

        let crosscutting_loc_total: usize = stats.values().map(|s| s.matching_lines).sum();
        let estimated_loc_savings: usize = stats.values().map(|s| s.estimated_loc_savings()).sum();
        let crosscutting_fraction = if total_loc > 0 {
            crosscutting_loc_total as f64 / total_loc as f64
        } else {
            0.0
        };

        let concerns = stats
            .iter()
            .map(|(ct, s)| {
                let scattering_score = s.scattering_score();
                let baseline = ct.baseline_scattering();
                let metrics = ConcernMetrics {
                    files_affected: s.files_affected,
                    total_files: s.total_files,
                    scattering_score,
                    matching_lines: s.matching_lines,
                    tangling_degree: s.tangling_degree(),
                    estimated_loc_savings: s.estimated_loc_savings(),
                    baseline_scattering: baseline,
                    scattering_delta: scattering_score - baseline,
                };
                (ct.name().to_string(), metrics)
            })
            .collect();

        Self {
            path: path.into(),
            total_files,
            total_loc,
            concerns,
            crosscutting_loc_total,
            estimated_loc_savings,
            crosscutting_fraction,
        }
    }

    /// Render the report in the specified format.
    pub fn render(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.render_json(),
            OutputFormat::Table => self.render_table(),
            OutputFormat::Csv => self.render_csv(),
        }
    }

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }

    fn render_table(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("LOC Analysis: {}\n", self.path));
        out.push_str(&format!("Total files: {}  Total LOC: {}\n", self.total_files, self.total_loc));
        out.push_str(&format!("Crosscutting LOC: {} ({:.1}% of total)\n", self.crosscutting_loc_total, self.crosscutting_fraction * 100.0));
        out.push_str(&format!("Estimated LOC savings with aspects: {}\n\n", self.estimated_loc_savings));
        out.push_str(&format!(
            "{:<22} {:>8} {:>8} {:>10} {:>8} {:>8}\n",
            "Concern", "Files", "Scatter%", "Lines", "Baseline", "Delta"
        ));
        out.push_str(&"-".repeat(74));
        out.push('\n');

        let mut sorted: Vec<(&String, &ConcernMetrics)> = self.concerns.iter().collect();
        sorted.sort_by(|a, b| b.1.scattering_score.partial_cmp(&a.1.scattering_score).unwrap());

        for (name, m) in sorted {
            let delta_str = if m.scattering_delta >= 0.0 {
                format!("+{:.1}%", m.scattering_delta * 100.0)
            } else {
                format!("{:.1}%", m.scattering_delta * 100.0)
            };
            out.push_str(&format!(
                "{:<22} {:>8} {:>7.1}% {:>10} {:>7.1}% {:>8}\n",
                name,
                m.files_affected,
                m.scattering_score * 100.0,
                m.matching_lines,
                m.baseline_scattering * 100.0,
                delta_str,
            ));
        }
        out
    }

    fn render_csv(&self) -> String {
        let mut out = String::from("concern,files_affected,total_files,scattering_score,matching_lines,estimated_savings,baseline_scattering,delta\n");
        let mut sorted: Vec<(&String, &ConcernMetrics)> = self.concerns.iter().collect();
        sorted.sort_by_key(|(name, _)| name.as_str());
        for (name, m) in sorted {
            out.push_str(&format!(
                "{},{},{},{:.4},{},{},{:.4},{:.4}\n",
                name,
                m.files_affected,
                m.total_files,
                m.scattering_score,
                m.matching_lines,
                m.estimated_loc_savings,
                m.baseline_scattering,
                m.scattering_delta,
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loc::crosscutting::ConcernStats;

    fn make_stats() -> HashMap<ConcernType, ConcernStats> {
        let mut m = HashMap::new();
        m.insert(ConcernType::RateLimiting, ConcernStats {
            concern: ConcernType::RateLimiting,
            files_affected: 28,
            total_files: 192,
            matching_lines: 120,
            total_lines: 129040,
        });
        m
    }

    #[test]
    fn test_report_json_is_valid() {
        let stats = make_stats();
        let report = LocReport::from_stats("test_path", stats);
        let json = report.render(OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("total_files").is_some());
    }

    #[test]
    fn test_report_table_contains_concern_name() {
        let stats = make_stats();
        let report = LocReport::from_stats("test_path", stats);
        let table = report.render(OutputFormat::Table);
        assert!(table.contains("rate_limiting"));
    }

    #[test]
    fn test_report_csv_has_header() {
        let stats = make_stats();
        let report = LocReport::from_stats("test_path", stats);
        let csv = report.render(OutputFormat::Csv);
        assert!(csv.starts_with("concern,"));
    }
}
