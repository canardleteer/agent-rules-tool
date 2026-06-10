//! CLI definitions for the `agent-rules-tool` binary.
//!
//! Library users typically call [`lint_string`](crate::lint_string) and
//! [`migrate_string`](crate::migrate_string) instead.
#![allow(missing_docs)]

use crate::Severity;
use crate::format::{RuleFormat, RuleFormatArg};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "agent-rules-tool",
    about = "Lint and migrate agent rules per agent-rules-spec"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Lint agent rule files against the agent-rules-spec schema
    Lint {
        /// Directory to lint (default: .agents/rules)
        #[arg(short = 'd', long, conflicts_with = "input_file")]
        directory: Option<PathBuf>,

        /// Single file to lint
        #[arg(short = 'i', long, conflicts_with = "directory")]
        input_file: Option<PathBuf>,

        /// Failure threshold: exit 1 if any violation at/above this level (default: error)
        #[arg(long, default_value = "error")]
        severity: SeverityArg,

        /// Write YAML lint report to this path
        #[arg(long)]
        report: Option<PathBuf>,
    },

    /// Migrate agent rule files between tool-native and canonical formats
    Migrate {
        /// Source format (or auto to detect from frontmatter / directory)
        #[arg(value_enum)]
        from: RuleFormatArg,

        /// Source directory (default: scan all known tool dirs)
        #[arg(short = 'd', long, conflicts_with = "input_file")]
        directory: Option<PathBuf>,

        /// Single file to migrate
        #[arg(short = 'i', long, conflicts_with = "directory")]
        input_file: Option<PathBuf>,

        /// Target format (default: agents)
        #[arg(long, default_value = "agents")]
        to: RuleFormatArg,

        /// Output directory (dir mode) or file (file mode)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Overwrite existing output files
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SeverityArg {
    Warn,
    Error,
}

impl From<SeverityArg> for Severity {
    fn from(arg: SeverityArg) -> Self {
        match arg {
            SeverityArg::Warn => Severity::Warn,
            SeverityArg::Error => Severity::Error,
        }
    }
}

impl Commands {
    pub fn lint_severity(&self) -> Option<Severity> {
        match self {
            Commands::Lint { severity, .. } => Some((*severity).into()),
            _ => None,
        }
    }

    pub fn migrate_to(&self) -> Option<RuleFormat> {
        match self {
            Commands::Migrate { to, .. } => Some(RuleFormat::from(*to)),
            _ => None,
        }
    }
}
