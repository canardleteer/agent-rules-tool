#![warn(missing_docs)]

//! Lint and migrate AI agent rule files per
//! [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec).
//!
//! # Quick start
//!
//! ```
//! use agent_rules_tool::{lint_string, migrate_string, LintOptions, MigrateOptions};
//! use agent_rules_tool::format::RuleFormat;
//!
//! # fn example(content: &str) -> Result<(), agent_rules_tool::Error> {
//! let report = lint_string(content, &LintOptions::default())?;
//! assert!(report.valid);
//!
//! let migrated = migrate_string(
//!     content,
//!     &MigrateOptions {
//!         from: RuleFormat::Cursor,
//!         to: RuleFormat::Agents,
//!         ..Default::default()
//!     },
//! )?;
//! # let _ = migrated;
//! # Ok(())
//! # }
//! ```
//!
//! # Modules
//!
//! - [`lint`] — validate rule content against schema and RFC semantics
//! - [`migrate`] — convert between tool-native and canonical formats
//! - [`mod@format`] — format detection and tool directory defaults
//! - [`spec`] — vendored spec URLs, embedded schema, and constants
//! - [`report`] — YAML lint report serialization

pub mod cli;
pub mod discover;
pub mod error;
pub mod format;
pub mod io;
pub mod lint;
pub mod migrate;
pub mod parse;
pub mod report;
pub mod schema;
pub mod spec;
pub mod walk;

/// Error type returned by library operations.
pub use error::Error;
/// Rule format identifiers for migration and detection.
pub use format::{RuleFormat, RuleFormatArg};
/// Lint a single rule string or directory of rule files.
pub use lint::{exceeds_threshold, lint_directory, lint_string};
/// Migrate rule files between formats.
pub use migrate::{
    InputRule, MigrateOptions, MigrateResult, MigrateSummary, MigrateWarning,
    build_inputs_from_dirs, migrate_paths, migrate_string,
};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Violation severity for lint results and exit-code thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Non-fatal issue; fails only when threshold is [`Severity::Warn`].
    Warn,
    /// Schema or RFC violation.
    Error,
}

impl Severity {
    /// Returns `true` when `self` is at or above `threshold`.
    pub fn meets_threshold(self, threshold: Self) -> bool {
        self >= threshold
    }
}

/// A single lint finding with a citation into the spec or schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// How severe this finding is.
    pub severity: Severity,
    /// Frontmatter field or path associated with the violation.
    pub field: String,
    /// Human-readable description of the problem.
    pub message: String,
    /// URL to the relevant RFC section or schema document.
    pub spec_ref: &'static str,
}

/// Options controlling [`lint_string`] and [`lint_directory`].
#[derive(Debug, Clone)]
pub struct LintOptions {
    /// Minimum severity that should fail a lint run (CLI exit code).
    pub severity_threshold: Severity,
    /// Filename stem used for semantic checks such as `name` vs filename.
    pub filename_hint: Option<String>,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            severity_threshold: Severity::Error,
            filename_hint: None,
        }
    }
}

/// Result of linting one rule file's content.
#[derive(Debug, Clone)]
pub struct LintReport {
    /// All violations found in the content.
    pub violations: Vec<Violation>,
    /// `true` when `violations` is empty.
    pub valid: bool,
}

/// Lint result for one file within a directory scan.
#[derive(Debug, Clone)]
pub struct FileLintResult {
    /// Path relative to the directory passed to [`lint_directory`].
    pub path: PathBuf,
    /// Lint outcome for this file.
    pub report: LintReport,
}

/// Vendored [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec) index manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct SpecIndex {
    /// Upstream repository URL.
    pub repository: String,
    /// Pinned upstream git commit.
    pub commit: String,
    /// Vendored file manifest entries.
    pub files: Vec<SpecIndexEntry>,
}

/// One entry in [`SpecIndex::files`].
#[derive(Debug, Clone, Deserialize)]
pub struct SpecIndexEntry {
    /// Path under `spec/` in this repository.
    pub vendored: String,
    /// Corresponding path in the upstream repository.
    pub upstream: String,
}

/// Load the embedded spec index from [`spec::EMBEDDED_INDEX`].
pub fn load_spec_index() -> Result<SpecIndex, Error> {
    serde_saphyr::from_str(spec::EMBEDDED_INDEX).map_err(|e| Error::Yaml(e.to_string()))
}
