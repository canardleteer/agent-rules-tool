//! Per-format frontmatter key tables and migration validation.

use super::{ISSUE_URL, MigrateWarning};
use crate::error::Error;
use crate::format::RuleFormat;
use serde_json::{Map, Value};

const AGENTS_KEYS: &[&str] = &[
    "name",
    "description",
    "trigger",
    "paths",
    "keywords",
    "discovery",
    "priority",
    "tags",
];
const CURSOR_KEYS: &[&str] = &["description", "globs", "alwaysApply", "name"];
const WINDSURF_KEYS: &[&str] = &["description", "trigger", "globs", "name"];
const COPILOT_KEYS: &[&str] = &["description", "applyTo", "name"];
const CLAUDE_CLINE_KEYS: &[&str] = &["description", "paths", "name"];
const JETBRAINS_KEYS: &[&str] = &["name", "description", "trigger", "paths", "keywords"];
const AMAZONQ_KEYS: &[&str] = &["name", "description"];

fn known_keys(format: RuleFormat) -> &'static [&'static str] {
    match format {
        RuleFormat::Agents | RuleFormat::Auto => AGENTS_KEYS,
        RuleFormat::Cursor => CURSOR_KEYS,
        RuleFormat::Windsurf => WINDSURF_KEYS,
        RuleFormat::Copilot => COPILOT_KEYS,
        RuleFormat::Claude | RuleFormat::Cline => CLAUDE_CLINE_KEYS,
        RuleFormat::Jetbrains => JETBRAINS_KEYS,
        RuleFormat::AmazonQ => AMAZONQ_KEYS,
    }
}

fn format_label(format: RuleFormat) -> &'static str {
    match format {
        RuleFormat::Agents => "agents",
        RuleFormat::Auto => "auto",
        RuleFormat::Cursor => "cursor",
        RuleFormat::Windsurf => "windsurf",
        RuleFormat::Copilot => "copilot",
        RuleFormat::Claude => "claude",
        RuleFormat::Cline => "cline",
        RuleFormat::Jetbrains => "jetbrains",
        RuleFormat::AmazonQ => "amazonq",
    }
}

/// Reject frontmatter keys that are not recognized for `format`.
pub fn validate_source_keys(format: RuleFormat, obj: &Map<String, Value>) -> Result<(), Error> {
    if format == RuleFormat::Auto {
        return Ok(());
    }
    let allowed = known_keys(format);
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(Error::Migrate(format!(
                "unknown frontmatter field '{key}' for {} format; open an issue at {ISSUE_URL} and include your rule file",
                format_label(format)
            )));
        }
    }
    Ok(())
}

/// Collect lossy or ambiguous migration warnings for a conversion.
pub fn collect_migration_warnings(
    source: RuleFormat,
    target: RuleFormat,
    src: &Map<String, Value>,
    agents: &Map<String, Value>,
) -> Vec<MigrateWarning> {
    let mut warnings = Vec::new();
    let source = if source == RuleFormat::Auto {
        RuleFormat::Agents
    } else {
        source
    };
    let target = if target == RuleFormat::Auto {
        RuleFormat::Agents
    } else {
        target
    };

    if target == RuleFormat::Agents {
        warnings.extend(inbound_to_agents_warnings(source, src, agents));
    }
    if source == RuleFormat::Agents || !agents.is_empty() {
        warnings.extend(outbound_from_agents_warnings(target, agents));
    }

    warnings
}

fn inbound_to_agents_warnings(
    source: RuleFormat,
    src: &Map<String, Value>,
    agents: &Map<String, Value>,
) -> Vec<MigrateWarning> {
    let mut warnings = Vec::new();

    match source {
        RuleFormat::Cursor => {
            let has_globs = src.contains_key("globs");
            let always_apply = src.get("alwaysApply").and_then(|v| v.as_bool());
            if !has_globs && always_apply != Some(true) && src.contains_key("description") {
                warnings.push(MigrateWarning {
                    field: Some("description".to_string()),
                    message:
                        "Cursor Apply Intelligently maps to trigger: auto with discovery: true"
                            .to_string(),
                });
            }
        }
        RuleFormat::Windsurf
            if src.get("trigger").and_then(|v| v.as_str()) == Some("model_decision") =>
        {
            warnings.push(MigrateWarning {
                field: Some("trigger".to_string()),
                message: "Windsurf model_decision maps to trigger: auto with discovery: true"
                    .to_string(),
            });
        }
        RuleFormat::Copilot => {
            if let Some(Value::String(s)) = src.get("applyTo")
                && s.contains(',')
            {
                warnings.push(MigrateWarning {
                    field: Some("applyTo".to_string()),
                    message: "Copilot applyTo comma-separated globs were split into paths array; verify patterns".to_string(),
                });
            }
            if !src.contains_key("applyTo") && src.contains_key("description") {
                warnings.push(MigrateWarning {
                    field: Some("description".to_string()),
                    message:
                        "Copilot on-demand instruction maps to trigger: auto with discovery: true"
                            .to_string(),
                });
            }
        }
        RuleFormat::Jetbrains
            if src.get("trigger").and_then(|v| v.as_str()) == Some("model_decision") =>
        {
            warnings.push(MigrateWarning {
                field: Some("trigger".to_string()),
                message: "JetBrains By model decision maps to trigger: auto with discovery: true"
                    .to_string(),
            });
        }
        RuleFormat::Claude | RuleFormat::Cline
            if agents.get("trigger").and_then(|v| v.as_str()) == Some("manual") =>
        {
            warnings.push(MigrateWarning {
                field: Some("trigger".to_string()),
                message: format!(
                    "{} native format does not support trigger: manual; inferred always/auto from paths",
                    format_label(source)
                ),
            });
        }
        _ => {}
    }

    warnings
}

fn outbound_from_agents_warnings(
    target: RuleFormat,
    agents: &Map<String, Value>,
) -> Vec<MigrateWarning> {
    let mut warnings = Vec::new();
    let trigger = agents
        .get("trigger")
        .and_then(|v| v.as_str())
        .unwrap_or("always");

    let (allowed, strip_message_prefix) = match target {
        RuleFormat::Agents | RuleFormat::Auto => return warnings,
        RuleFormat::Cursor => (
            &["description", "globs", "alwaysApply"][..],
            "Cursor format does not support",
        ),
        RuleFormat::Windsurf => (
            &["description", "trigger", "globs"][..],
            "Windsurf format does not support",
        ),
        RuleFormat::Copilot => (
            &["description", "applyTo"][..],
            "Copilot format does not support",
        ),
        RuleFormat::Claude | RuleFormat::Cline => {
            (&["paths"][..], "Claude/Cline format does not support")
        }
        RuleFormat::Jetbrains => (
            &["description", "name", "paths"][..],
            "JetBrains format file frontmatter does not support",
        ),
        RuleFormat::AmazonQ => (
            &["description", "name"][..],
            "Amazon Q format does not support",
        ),
    };

    for key in agents.keys() {
        if key == "discovery"
            && matches!(
                target,
                RuleFormat::Cursor
                    | RuleFormat::Windsurf
                    | RuleFormat::Copilot
                    | RuleFormat::Jetbrains
            )
        {
            continue;
        }
        if !allowed.contains(&key.as_str()) {
            warnings.push(MigrateWarning {
                field: Some(key.clone()),
                message: format!("{strip_message_prefix} field '{key}'; it will be omitted"),
            });
        }
    }

    if trigger == "manual" {
        match target {
            RuleFormat::Copilot | RuleFormat::Claude | RuleFormat::Cline => {
                warnings.push(MigrateWarning {
                    field: Some("trigger".to_string()),
                    message: format!(
                        "agents trigger: manual has no equivalent in {} format",
                        format_label(target)
                    ),
                });
            }
            _ => {}
        }
    }

    if agents.get("discovery").and_then(|v| v.as_bool()) == Some(true) {
        let native = match target {
            RuleFormat::Cursor => Some("Cursor Apply Intelligently"),
            RuleFormat::Windsurf => Some("Windsurf trigger: model_decision"),
            RuleFormat::Copilot => Some("Copilot on-demand instruction"),
            RuleFormat::Jetbrains => Some("JetBrains By model decision"),
            _ => None,
        };
        if let Some(native) = native {
            warnings.push(MigrateWarning {
                field: Some("discovery".to_string()),
                message: format!(
                    "discovery: true exported as {native}; advisory paths/keywords are omitted"
                ),
            });
        }
    } else if target == RuleFormat::Cursor
        && trigger == "auto"
        && !agents.contains_key("paths")
        && !agents.contains_key("keywords")
    {
        warnings.push(MigrateWarning {
            field: Some("trigger".to_string()),
            message: "agents trigger: auto without paths or keywords maps ambiguously to Cursor activation".to_string(),
        });
    }

    warnings
}
