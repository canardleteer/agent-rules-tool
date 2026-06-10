use agent_rules_tool::format::RuleFormat;
use agent_rules_tool::migrate::{InputRule, MigrateOptions, migrate_paths, migrate_string};
use std::path::PathBuf;
use tempfile::TempDir;

const CURSOR_NATIVE: &str = r#"---
description: API design patterns
globs:
  - "src/api/**/*.ts"
alwaysApply: false
---

# API Conventions
Use RESTful naming.
"#;

#[test]
fn migrate_cursor_to_agents() {
    let options = MigrateOptions {
        from: RuleFormat::Cursor,
        to: RuleFormat::Agents,
        force: false,
        filename_hint: Some("api-conventions".to_string()),
    };
    let result = migrate_string(CURSOR_NATIVE, &options).expect("migrate");
    assert!(result.content.contains("trigger: auto"));
    assert!(result.content.contains("paths:"));
    assert!(result.content.contains("src/api/**/*.ts"));
    assert!(!result.content.contains("globs:"));
    assert!(!result.content.contains("alwaysApply"));
}

#[test]
fn migrate_agents_to_cursor_round_trip() {
    let agents = r#"---
name: api-conventions
description: API design patterns
trigger: auto
paths:
  - "src/api/**/*.ts"
---

# API Conventions
Use RESTful naming.
"#;

    let to_cursor = MigrateOptions {
        from: RuleFormat::Agents,
        to: RuleFormat::Cursor,
        force: false,
        filename_hint: Some("api-conventions".to_string()),
    };
    let cursor = migrate_string(agents, &to_cursor).expect("to cursor");
    assert!(cursor.content.contains("globs:"));
    assert!(cursor.content.contains("alwaysApply: false"));

    let back = MigrateOptions {
        from: RuleFormat::Cursor,
        to: RuleFormat::Agents,
        force: false,
        filename_hint: Some("api-conventions".to_string()),
    };
    let agents_again = migrate_string(&cursor.content, &back).expect("back to agents");
    assert!(agents_again.content.contains("trigger: auto"));
    assert!(agents_again.content.contains("paths:"));
}

#[test]
fn migrate_directory_writes_agents_rules() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    let cursor_dir = root.join(".cursor/rules");
    std::fs::create_dir_all(&cursor_dir).expect("mkdir");
    let source = cursor_dir.join("api-conventions.mdc");
    std::fs::write(&source, CURSOR_NATIVE).expect("write source");

    let output_root = root.join(".agents/rules");
    let inputs = vec![InputRule {
        source_path: source.clone(),
        relative_path: PathBuf::from("api-conventions.mdc"),
        format: RuleFormat::Cursor,
        content: CURSOR_NATIVE.to_string(),
    }];

    let options = MigrateOptions {
        from: RuleFormat::Cursor,
        to: RuleFormat::Agents,
        force: false,
        filename_hint: None,
    };

    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let summary = rt
        .block_on(migrate_paths(&inputs, &output_root, &options))
        .expect("migrate paths");

    assert_eq!(summary.written.len(), 1);
    let out_file = output_root.join("api-conventions.md");
    assert!(out_file.exists());
    let content = std::fs::read_to_string(&out_file).expect("read output");
    assert!(content.contains("trigger: auto"));
}

#[test]
fn migrate_skips_existing_without_force() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    let output_root = root.join(".agents/rules");
    std::fs::create_dir_all(&output_root).expect("mkdir");
    let out_file = output_root.join("api-conventions.md");
    std::fs::write(&out_file, "existing").expect("write existing");

    let inputs = vec![InputRule {
        source_path: PathBuf::from("source.mdc"),
        relative_path: PathBuf::from("api-conventions.mdc"),
        format: RuleFormat::Cursor,
        content: CURSOR_NATIVE.to_string(),
    }];

    let options = MigrateOptions {
        from: RuleFormat::Cursor,
        to: RuleFormat::Agents,
        force: false,
        filename_hint: None,
    };

    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let summary = rt
        .block_on(migrate_paths(&inputs, &output_root, &options))
        .expect("migrate");

    assert_eq!(summary.skipped.len(), 1);
    assert_eq!(
        std::fs::read_to_string(&out_file).expect("read"),
        "existing"
    );
}

#[test]
fn migrate_overwrites_with_force() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    let output_root = root.join(".agents/rules");
    std::fs::create_dir_all(&output_root).expect("mkdir");
    let out_file = output_root.join("api-conventions.md");
    std::fs::write(&out_file, "existing").expect("write existing");

    let inputs = vec![InputRule {
        source_path: PathBuf::from("source.mdc"),
        relative_path: PathBuf::from("api-conventions.mdc"),
        format: RuleFormat::Cursor,
        content: CURSOR_NATIVE.to_string(),
    }];

    let options = MigrateOptions {
        from: RuleFormat::Cursor,
        to: RuleFormat::Agents,
        force: true,
        filename_hint: None,
    };

    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let summary = rt
        .block_on(migrate_paths(&inputs, &output_root, &options))
        .expect("migrate");

    assert_eq!(summary.written.len(), 1);
    let content = std::fs::read_to_string(&out_file).expect("read");
    assert!(content.contains("trigger: auto"));
}

#[test]
fn migrate_unknown_cursor_key_errors() {
    let content = r#"---
description: test
globs: "**/*.rs"
alwaysApply: false
activation: paths
---
"#;
    let options = MigrateOptions {
        from: RuleFormat::Cursor,
        to: RuleFormat::Agents,
        ..Default::default()
    };
    let err = migrate_string(content, &options).unwrap_err();
    assert!(
        err.to_string()
            .contains("unknown frontmatter field 'activation'")
    );
    assert!(err.to_string().contains("agent-rules-spec/issues"));
}

#[test]
fn migrate_unknown_agents_key_errors() {
    let content = r#"---
name: test
trigger: always
activation: paths
---
"#;
    let options = MigrateOptions {
        from: RuleFormat::Agents,
        to: RuleFormat::Cursor,
        ..Default::default()
    };
    let err = migrate_string(content, &options).unwrap_err();
    assert!(
        err.to_string()
            .contains("unknown frontmatter field 'activation'")
    );
}

#[test]
fn migrate_windsurf_model_decision_warns() {
    let content = r#"---
description: test
trigger: model_decision
globs:
  - "**/*.rs"
---
"#;
    let options = MigrateOptions {
        from: RuleFormat::Windsurf,
        to: RuleFormat::Agents,
        ..Default::default()
    };
    let result = migrate_string(content, &options).expect("migrate");
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("model_decision"))
    );
}

#[test]
fn migrate_agents_to_cursor_keywords_warns() {
    let content = r#"---
name: test
description: test
trigger: auto
paths:
  - "**/*.rs"
keywords:
  - "api"
---
"#;
    let options = MigrateOptions {
        from: RuleFormat::Agents,
        to: RuleFormat::Cursor,
        ..Default::default()
    };
    let result = migrate_string(content, &options).expect("migrate");
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.field.as_deref() == Some("keywords"))
    );
}
