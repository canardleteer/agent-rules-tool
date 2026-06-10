//! YAML serialization of lint reports for CLI `--report` output.

use crate::error::Error;
use crate::spec::SPEC_COMMIT;
use crate::{FileLintResult, LintReport, Severity, Violation};
use serde::Serialize;

/// One violation in a YAML lint report.
#[derive(Debug, Serialize)]
pub struct ReportViolation {
    /// `"warn"` or `"error"`.
    pub severity: String,
    /// Frontmatter field or JSON pointer path.
    pub field: String,
    /// Human-readable message.
    pub message: String,
    /// Spec or schema URL citation.
    pub spec_ref: String,
}

/// Per-file section of an aggregate YAML report.
#[derive(Debug, Serialize)]
pub struct FileReport {
    /// File path relative to the lint root.
    pub path: String,
    /// Whether this file had no violations.
    pub valid: bool,
    /// Violations found in this file.
    pub violations: Vec<ReportViolation>,
}

/// Directory lint report written by `--report` in directory mode.
#[derive(Debug, Serialize)]
pub struct AggregateReport {
    /// Pinned spec commit at time of lint ([`SPEC_COMMIT`]).
    pub spec_commit: String,
    /// `true` when every file is valid.
    pub valid: bool,
    /// Per-file results.
    pub files: Vec<FileReport>,
}

/// Single-file lint report written by `--report` in file mode.
#[derive(Debug, Serialize)]
pub struct SingleFileReport {
    /// Pinned spec commit at time of lint ([`SPEC_COMMIT`]).
    pub spec_commit: String,
    /// Whether the file had no violations.
    pub valid: bool,
    /// Violations found.
    pub violations: Vec<ReportViolation>,
}

impl From<&Violation> for ReportViolation {
    fn from(v: &Violation) -> Self {
        Self {
            severity: match v.severity {
                Severity::Warn => "warn".to_string(),
                Severity::Error => "error".to_string(),
            },
            field: v.field.clone(),
            message: v.message.clone(),
            spec_ref: v.spec_ref.to_string(),
        }
    }
}

/// Serialize one [`LintReport`] to YAML for `--report` output.
pub fn report_to_yaml_single(report: &LintReport) -> Result<String, Error> {
    let doc = SingleFileReport {
        spec_commit: SPEC_COMMIT.to_string(),
        valid: report.valid,
        violations: report.violations.iter().map(Into::into).collect(),
    };
    serde_saphyr::to_string(&doc).map_err(|e| Error::Yaml(e.to_string()))
}

/// Serialize directory lint results to YAML for `--report` output.
pub fn report_to_yaml_aggregate(results: &[FileLintResult]) -> Result<String, Error> {
    let files = results
        .iter()
        .map(|r| FileReport {
            path: r.path.display().to_string(),
            valid: r.report.valid,
            violations: r.report.violations.iter().map(Into::into).collect(),
        })
        .collect();
    let doc = AggregateReport {
        spec_commit: SPEC_COMMIT.to_string(),
        valid: aggregate_valid(results),
        files,
    };
    serde_saphyr::to_string(&doc).map_err(|e| Error::Yaml(e.to_string()))
}

fn aggregate_valid(results: &[FileLintResult]) -> bool {
    results.iter().all(|r| r.report.valid)
}
