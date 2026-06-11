//! Lint a vendored agents rule and migrate a Cursor-native rule to agents format.
//!
//! Run from the repository root:
//!
//! ```bash
//! cargo run --example lint_and_migrate
//! ```

use agent_rules_tool::format::RuleFormat;
use agent_rules_tool::{LintOptions, MigrateOptions, lint_string, migrate_string};
use std::path::PathBuf;

const CURSOR_NATIVE: &str = r#"---
description: API design patterns
globs:
  - "src/api/**/*.ts"
alwaysApply: false
---

# API Conventions

Use RESTful naming.
"#;

fn main() -> Result<(), agent_rules_tool::Error> {
    let canonical = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec/examples/code-style.md");
    let content = std::fs::read_to_string(&canonical)?;

    let report = lint_string(
        &content,
        &LintOptions {
            filename_hint: Some("code-style".to_string()),
            ..Default::default()
        },
    )?;

    println!("Lint {}: valid={}", canonical.display(), report.valid);
    for violation in &report.violations {
        println!(
            "  {:?} {}: {}",
            violation.severity, violation.field, violation.message
        );
    }

    let migrated = migrate_string(
        CURSOR_NATIVE,
        &MigrateOptions {
            from: RuleFormat::Cursor,
            to: RuleFormat::Agents,
            filename_hint: Some("api-conventions".to_string()),
            ..Default::default()
        },
    )?;

    for warning in &migrated.warnings {
        eprintln!(
            "migrate warning: {} — {}",
            warning.field.as_deref().unwrap_or("-"),
            warning.message
        );
    }

    let migrated_report = lint_string(
        &migrated.content,
        &LintOptions {
            filename_hint: Some("api-conventions".to_string()),
            ..Default::default()
        },
    )?;

    println!("Lint migrated Cursor rule: valid={}", migrated_report.valid);
    println!("--- migrated output ---\n{}", migrated.content);

    Ok(())
}
