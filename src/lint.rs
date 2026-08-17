//! Lint rule file content against the embedded JSON Schema and RFC semantics.

use crate::error::Error;
use crate::parse::{is_empty_frontmatter, parse_rule};
use crate::schema::validate_frontmatter;
use crate::spec::{RFC_DISCOVERY, RFC_FILE_CONVENTIONS, RFC_TRIGGER_MODES};
use crate::{FileLintResult, LintOptions, LintReport, Severity, Violation};
use serde_json::Value;
use std::path::Path;

/// Lint one rule file's markdown content (frontmatter + body).
///
/// Validates frontmatter against the embedded schema and applies semantic checks
/// such as `name` vs filename stem, `discovery: true` draft warnings, and
/// `trigger: auto` requiring `paths` or `keywords` unless the rule is in
/// discovery mode.
///
/// # Examples
///
/// ```
/// use agent_rules_tool::{lint_string, LintOptions};
///
/// let content = "---\nname: my-rule\ntrigger: always\n---\n\n# My rule\n";
/// let report = lint_string(content, &LintOptions::default())?;
/// assert!(report.valid);
/// # Ok::<(), agent_rules_tool::Error>(())
/// ```
pub fn lint_string(content: &str, options: &LintOptions) -> Result<LintReport, Error> {
    let parsed = parse_rule(content)?;
    let mut violations = validate_frontmatter(&parsed.frontmatter)?;
    violations.extend(semantic_checks(
        &parsed.frontmatter,
        options.filename_hint.as_deref(),
    ));

    let valid = violations.is_empty();
    Ok(LintReport { violations, valid })
}

/// Recursively lint all `*.md` files under `root`.
///
/// Returns an empty vector when `root` does not exist. Paths in each
/// [`FileLintResult`] are relative to `root`.
pub fn lint_directory(root: &Path, options: &LintOptions) -> Result<Vec<FileLintResult>, Error> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    collect_md_files(root, root, &mut results, options)?;
    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

fn collect_md_files(
    root: &Path,
    current: &Path,
    results: &mut Vec<FileLintResult>,
    options: &LintOptions,
) -> Result<(), Error> {
    if !current.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(root, &path, results, options)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let content = std::fs::read_to_string(&path)?;
            let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            let filename_hint = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(str::to_string);
            let mut file_options = options.clone();
            file_options.filename_hint = filename_hint;
            let report = lint_string(&content, &file_options)?;
            results.push(FileLintResult { path: rel, report });
        }
    }
    Ok(())
}

fn semantic_checks(frontmatter: &Value, filename_hint: Option<&str>) -> Vec<Violation> {
    if is_empty_frontmatter(frontmatter) {
        return Vec::new();
    }

    let mut violations = Vec::new();

    if let Some(stem) = filename_hint
        && let Some(name) = frontmatter.get("name").and_then(|v| v.as_str())
        && name != stem
    {
        violations.push(Violation {
            severity: Severity::Warn,
            field: "name".to_string(),
            message: format!("name '{name}' does not match filename stem '{stem}'"),
            spec_ref: RFC_FILE_CONVENTIONS,
        });
    }

    let discovery = frontmatter
        .get("discovery")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if discovery {
        violations.push(Violation {
            severity: Severity::Warn,
            field: "discovery".to_string(),
            message: "discovery: true marks this rule as a draft seeking agentic guidance"
                .to_string(),
            spec_ref: RFC_DISCOVERY,
        });
    }

    if let Some(trigger) = frontmatter.get("trigger").and_then(|v| v.as_str())
        && trigger == "auto"
    {
        let has_paths = frontmatter
            .get("paths")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty());
        let has_keywords = frontmatter
            .get("keywords")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty());
        if !has_paths && !has_keywords {
            if discovery {
                violations.push(Violation {
                    severity: Severity::Warn,
                    field: "trigger".to_string(),
                    message: "discovery draft with trigger: auto has no binding paths or keywords"
                        .to_string(),
                    spec_ref: RFC_DISCOVERY,
                });
            } else {
                violations.push(Violation {
                    severity: Severity::Error,
                    field: "trigger".to_string(),
                    message: "when trigger is 'auto', paths or keywords is required".to_string(),
                    spec_ref: RFC_TRIGGER_MODES,
                });
            }
        }
    }

    violations
}

/// Returns `true` when any violation meets or exceeds `threshold`.
pub fn exceeds_threshold(violations: &[Violation], threshold: Severity) -> bool {
    violations.iter().any(|v| v.severity >= threshold)
}

/// Returns `true` when every file result in `results` is valid.
pub fn aggregate_valid(results: &[FileLintResult]) -> bool {
    results.iter().all(|r| r.report.valid)
}
