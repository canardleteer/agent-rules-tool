use agent_rules_tool::lint::lint_string;
use agent_rules_tool::spec::{SCHEMA_URL, SPEC_COMMIT};
use agent_rules_tool::{LintOptions, Severity, load_spec_index};
use std::path::PathBuf;

fn spec_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
}

#[test]
fn vendored_index_matches_spec_commit_and_files_exist() {
    let index = load_spec_index().expect("index.yaml should parse");
    assert_eq!(index.commit, SPEC_COMMIT);
    assert_eq!(
        index.repository,
        "https://github.com/rameshsunkara/agent-rules-spec"
    );

    for entry in &index.files {
        let path = spec_root().join(&entry.vendored);
        assert!(path.exists(), "missing vendored file: {}", entry.vendored);
    }
}

#[test]
fn lint_vendored_examples_are_valid() {
    let index = load_spec_index().expect("index should load");
    let examples: Vec<_> = index
        .files
        .iter()
        .filter(|e| e.vendored.starts_with("examples/"))
        .collect();
    assert_eq!(examples.len(), 3);

    for entry in examples {
        let path = spec_root().join(&entry.vendored);
        let content = std::fs::read_to_string(&path).expect("read example");
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(str::to_string);
        let report = lint_string(
            &content,
            &LintOptions {
                severity_threshold: Severity::Error,
                filename_hint: stem,
            },
        )
        .expect("lint should succeed");
        assert!(report.valid, "expected valid: {}", entry.vendored);
    }
}

#[test]
fn lint_invalid_frontmatter_reports_schema_violations() {
    let content = r#"---
trigger: bogus
priority: 9999
---
body
"#;
    let report = lint_string(content, &LintOptions::default()).expect("lint should run");
    assert!(!report.valid);
    assert!(
        report.violations.iter().any(|v| v.spec_ref == SCHEMA_URL),
        "expected schema citation in violations"
    );
}
