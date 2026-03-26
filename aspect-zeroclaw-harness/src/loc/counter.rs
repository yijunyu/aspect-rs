//! LOC counter: walks a Rust source tree and counts crosscutting concern lines.

use crate::loc::crosscutting::{ConcernStats, ConcernType, get_patterns};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Result of analyzing a single Rust source file.
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub total_lines: usize,
    pub concerns_found: HashSet<ConcernType>,
    pub concern_lines: HashMap<ConcernType, usize>,
}

/// Analyzes a Rust source tree for crosscutting concerns.
pub struct LocCounter {
    root: PathBuf,
}

impl LocCounter {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Analyze all `.rs` files under the root and return per-file analysis.
    pub fn analyze_files(&self) -> Result<Vec<FileAnalysis>> {
        let patterns = get_patterns();
        let mut results = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let path = entry.path().to_path_buf();
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let total_lines = content.lines().count();
            let mut concerns_found = HashSet::new();
            let mut concern_lines: HashMap<ConcernType, usize> = HashMap::new();

            for line in content.lines() {
                // Skip comment-only lines for accuracy
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("*") {
                    continue;
                }

                for (&concern_type, concern_patterns) in patterns {
                    if concern_patterns.matches_line(line) {
                        concerns_found.insert(concern_type);
                        *concern_lines.entry(concern_type).or_insert(0) += 1;
                    }
                }
            }

            results.push(FileAnalysis {
                path,
                total_lines,
                concerns_found,
                concern_lines,
            });
        }

        Ok(results)
    }

    /// Aggregate file analyses into concern-level statistics.
    pub fn aggregate(&self, analyses: &[FileAnalysis]) -> HashMap<ConcernType, ConcernStats> {
        let total_files = analyses.len();
        let total_lines: usize = analyses.iter().map(|a| a.total_lines).sum();

        ConcernType::all()
            .iter()
            .map(|&ct| {
                let files_affected = analyses.iter().filter(|a| a.concerns_found.contains(&ct)).count();
                let matching_lines: usize = analyses.iter().map(|a| a.concern_lines.get(&ct).copied().unwrap_or(0)).sum();

                let stats = ConcernStats {
                    concern: ct,
                    files_affected,
                    total_files,
                    matching_lines,
                    total_lines,
                };
                (ct, stats)
            })
            .collect()
    }

    /// Run the full analysis pipeline.
    pub fn run(&self) -> Result<HashMap<ConcernType, ConcernStats>> {
        let analyses = self.analyze_files()?;
        Ok(self.aggregate(&analyses))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_counter_finds_rate_limiting() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            "tool.rs",
            "fn execute() {\n    if self.security.is_rate_limited() {\n        return;\n    }\n}\n",
        );

        let counter = LocCounter::new(tmp.path());
        let analyses = counter.analyze_files().unwrap();
        assert_eq!(analyses.len(), 1);
        assert!(analyses[0].concerns_found.contains(&ConcernType::RateLimiting));
    }

    #[test]
    fn test_counter_clean_file_has_no_concerns() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            "pure.rs",
            "fn add(a: i32, b: i32) -> i32 { a + b }\n",
        );

        let counter = LocCounter::new(tmp.path());
        let analyses = counter.analyze_files().unwrap();
        assert_eq!(analyses.len(), 1);
        // May have some matches for common patterns but not rate limiting
        assert!(!analyses[0].concerns_found.contains(&ConcernType::RateLimiting));
        assert!(!analyses[0].concerns_found.contains(&ConcernType::ApprovalHitl));
    }

    #[test]
    fn test_aggregate_scattering_score() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), "a.rs", "if self.security.is_rate_limited() {}\n");
        write_file(tmp.path(), "b.rs", "fn pure() {}\n");
        write_file(tmp.path(), "c.rs", "let x = is_rate_limited();\n");

        let counter = LocCounter::new(tmp.path());
        let analyses = counter.analyze_files().unwrap();
        let stats = counter.aggregate(&analyses);

        let rl = &stats[&ConcernType::RateLimiting];
        // 2 out of 3 files have rate limiting
        assert_eq!(rl.files_affected, 2);
        assert_eq!(rl.total_files, 3);
        assert!((rl.scattering_score() - 0.667).abs() < 0.01);
    }

    #[test]
    fn test_comments_are_skipped() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            "commented.rs",
            "// if self.security.is_rate_limited() {\n//     return;\n// }\nfn pure() {}\n",
        );

        let counter = LocCounter::new(tmp.path());
        let analyses = counter.analyze_files().unwrap();
        // Comments should not be counted
        assert!(!analyses[0].concerns_found.contains(&ConcernType::RateLimiting));
    }
}
