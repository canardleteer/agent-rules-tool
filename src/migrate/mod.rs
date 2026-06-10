//! Migrate agent rule files between tool-native and canonical formats.

mod mapping;

use crate::discover::{find_tool_dir, infer_format_from_path};
use crate::error::Error;
use crate::format::RuleFormat;
use crate::io::{WriteOutcome, write_file_atomic};
use crate::migrate::mapping::{collect_migration_warnings, validate_source_keys};
use crate::parse::{is_empty_frontmatter, parse_rule};
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::warn;

pub(crate) const ISSUE_URL: &str = "https://github.com/rameshsunkara/agent-rules-spec/issues";

/// A non-fatal migration finding (lossy or ambiguous field mapping).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrateWarning {
    /// Frontmatter field related to the warning, if any.
    pub field: Option<String>,
    /// Human-readable explanation.
    pub message: String,
}

/// Options for [`migrate_string`] and [`migrate_paths`].
#[derive(Debug, Clone)]
pub struct MigrateOptions {
    /// Source format, or [`RuleFormat::Auto`] to detect from frontmatter.
    pub from: RuleFormat,
    /// Target format to emit.
    pub to: RuleFormat,
    /// When writing to disk, overwrite existing output files.
    pub force: bool,
    /// Filename stem used when inferring `name` for empty frontmatter.
    pub filename_hint: Option<String>,
}

impl Default for MigrateOptions {
    fn default() -> Self {
        Self {
            from: RuleFormat::Auto,
            to: RuleFormat::Agents,
            force: false,
            filename_hint: None,
        }
    }
}

/// One rule file discovered for batch migration.
#[derive(Debug, Clone)]
pub struct InputRule {
    /// Absolute path to the source file on disk.
    pub source_path: PathBuf,
    /// Path relative to the scan root (used for output layout).
    pub relative_path: PathBuf,
    /// Detected or declared format of the source file.
    pub format: RuleFormat,
    /// Raw file contents.
    pub content: String,
}

/// Result of migrating one rule file's content in memory.
#[derive(Debug, Clone)]
pub struct MigrateResult {
    /// Migrated markdown (frontmatter + body).
    pub content: String,
    /// Lossy or ambiguous mapping warnings.
    pub warnings: Vec<MigrateWarning>,
}

/// Summary of files written or skipped by [`migrate_paths`].
#[derive(Debug, Clone, Default)]
pub struct MigrateSummary {
    /// Output paths that were written.
    pub written: Vec<PathBuf>,
    /// Output paths that were skipped (duplicate stem or existing file without `force`).
    pub skipped: Vec<PathBuf>,
    /// Warnings aggregated across all migrated files.
    pub warnings: Vec<MigrateWarning>,
}

/// Migrate one rule file's content between formats in memory.
///
/// Parses markdown frontmatter, normalizes to canonical agents frontmatter, then
/// maps fields to the target format.
///
/// # Examples
///
/// ```
/// use agent_rules_tool::{migrate_string, MigrateOptions};
/// use agent_rules_tool::format::RuleFormat;
///
/// let content = "---\nglobs: \"**/*.rs\"\nalwaysApply: false\n---\n\n# Rule\n";
/// let result = migrate_string(
///     content,
///     &MigrateOptions {
///         from: RuleFormat::Cursor,
///         to: RuleFormat::Agents,
///         ..Default::default()
///     },
/// )?;
/// assert!(result.content.contains("trigger:"));
/// # Ok::<(), agent_rules_tool::Error>(())
/// ```
pub fn migrate_string(content: &str, options: &MigrateOptions) -> Result<MigrateResult, Error> {
    let parsed = parse_rule(content)?;
    let source = options.from.resolve(&parsed.frontmatter);

    let src_obj = if is_empty_frontmatter(&parsed.frontmatter) {
        None
    } else {
        Some(
            parsed
                .frontmatter
                .as_object()
                .ok_or_else(|| Error::Migrate("frontmatter must be an object".to_string()))?
                .clone(),
        )
    };

    if let Some(ref obj) = src_obj {
        validate_source_keys(source, obj)?;
    }

    let agents = to_agents(
        &parsed.frontmatter,
        source,
        options.filename_hint.as_deref(),
    )?;

    let empty = Map::new();
    let src_ref = src_obj.as_ref().unwrap_or(&empty);
    let warnings = collect_migration_warnings(source, options.to, src_ref, &agents);

    let output_fm = from_agents(&agents, options.to)?;
    let content = assemble_markdown(&output_fm, &parsed.body)?;

    Ok(MigrateResult { content, warnings })
}

/// Migrate discovered inputs and write results under `output_root`.
///
/// Uses atomic writes via [`crate::io::write_file_atomic`]. Skips outputs that
/// already exist unless [`MigrateOptions::force`] is set.
pub async fn migrate_paths(
    inputs: &[InputRule],
    output_root: &Path,
    options: &MigrateOptions,
) -> Result<MigrateSummary, Error> {
    let mut summary = MigrateSummary::default();
    let mut seen_stems: HashSet<String> = HashSet::new();

    for input in inputs {
        let stem = normalized_stem(&input.relative_path, options.to)?;
        if !seen_stems.insert(stem.clone()) && !options.force {
            warn!(
                path = %input.source_path.display(),
                stem = %stem,
                "duplicate rule stem; skipping (use --force to overwrite)"
            );
            summary
                .skipped
                .push(output_root.join(output_relative(&input.relative_path, options.to)));
            continue;
        }

        let migrated = migrate_string(
            &input.content,
            &MigrateOptions {
                from: if input.format == RuleFormat::Auto {
                    options.from
                } else {
                    input.format
                },
                to: options.to,
                force: options.force,
                filename_hint: Some(stem),
            },
        )?;

        summary.warnings.extend(migrated.warnings);

        let out_rel = output_relative(&input.relative_path, options.to);
        let out_path = output_root.join(&out_rel);

        match write_file_atomic(&out_path, &migrated.content, options.force).await? {
            WriteOutcome::Written => summary.written.push(out_path),
            WriteOutcome::Skipped => {
                warn!(path = %out_path.display(), "output exists; skipping (use --force to overwrite)");
                summary.skipped.push(out_path);
            }
        }
    }

    Ok(summary)
}

fn to_agents(
    frontmatter: &Value,
    source: RuleFormat,
    filename_hint: Option<&str>,
) -> Result<Map<String, Value>, Error> {
    let mut agents = Map::new();

    if is_empty_frontmatter(frontmatter) {
        if let Some(stem) = filename_hint {
            agents.insert("name".to_string(), json!(stem));
        }
        agents.insert("trigger".to_string(), json!("always"));
        return Ok(agents);
    }

    let obj = frontmatter
        .as_object()
        .ok_or_else(|| Error::Migrate("frontmatter must be an object".to_string()))?;

    match source {
        RuleFormat::Agents | RuleFormat::Auto => {
            for (k, v) in obj {
                agents.insert(k.clone(), v.clone());
            }
        }
        RuleFormat::Cursor => convert_cursor_to_agents(obj, &mut agents)?,
        RuleFormat::Windsurf => convert_windsurf_to_agents(obj, &mut agents)?,
        RuleFormat::Copilot => convert_copilot_to_agents(obj, &mut agents)?,
        RuleFormat::Claude | RuleFormat::Cline => convert_claude_to_agents(obj, &mut agents)?,
        RuleFormat::Jetbrains => convert_jetbrains_to_agents(obj, &mut agents)?,
        RuleFormat::AmazonQ => convert_amazonq_to_agents(obj, &mut agents)?,
    }

    if !agents.contains_key("name")
        && let Some(stem) = filename_hint
    {
        agents.insert("name".to_string(), json!(stem));
    }

    Ok(agents)
}

fn convert_cursor_to_agents(
    src: &Map<String, Value>,
    dst: &mut Map<String, Value>,
) -> Result<(), Error> {
    copy_field(src, dst, "description");
    let globs = src.get("globs").cloned();
    let always_apply = src.get("alwaysApply").and_then(|v| v.as_bool());

    if let Some(globs) = globs {
        dst.insert("paths".to_string(), globs);
    }

    let trigger = if always_apply == Some(true) {
        "always"
    } else if src.contains_key("globs") {
        "auto"
    } else {
        "manual"
    };
    dst.insert("trigger".to_string(), json!(trigger));
    copy_optional_fields(src, dst, &["name"]);
    Ok(())
}

fn convert_windsurf_to_agents(
    src: &Map<String, Value>,
    dst: &mut Map<String, Value>,
) -> Result<(), Error> {
    copy_field(src, dst, "description");
    if let Some(globs) = src.get("globs") {
        dst.insert("paths".to_string(), globs.clone());
    }
    let trigger = match src.get("trigger").and_then(|v| v.as_str()) {
        Some("always_on") => "always",
        Some("glob") => "auto",
        Some("manual") => "manual",
        Some("model_decision") => "auto",
        _ if src.contains_key("globs") => "auto",
        _ => "always",
    };
    dst.insert("trigger".to_string(), json!(trigger));
    copy_optional_fields(src, dst, &["name"]);
    Ok(())
}

fn convert_copilot_to_agents(
    src: &Map<String, Value>,
    dst: &mut Map<String, Value>,
) -> Result<(), Error> {
    copy_field(src, dst, "description");
    if let Some(apply_to) = src.get("applyTo") {
        let paths = match apply_to {
            Value::String(s) => json!(
                s.split(',')
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .collect::<Vec<_>>()
            ),
            other => other.clone(),
        };
        dst.insert("paths".to_string(), paths);
        dst.insert("trigger".to_string(), json!("auto"));
    } else {
        dst.insert("trigger".to_string(), json!("always"));
    }
    copy_optional_fields(src, dst, &["name"]);
    Ok(())
}

fn convert_claude_to_agents(
    src: &Map<String, Value>,
    dst: &mut Map<String, Value>,
) -> Result<(), Error> {
    copy_field(src, dst, "description");
    if let Some(paths) = src.get("paths") {
        dst.insert("paths".to_string(), paths.clone());
        dst.insert("trigger".to_string(), json!("auto"));
    } else {
        dst.insert("trigger".to_string(), json!("always"));
    }
    copy_optional_fields(src, dst, &["name"]);
    Ok(())
}

fn convert_jetbrains_to_agents(
    src: &Map<String, Value>,
    dst: &mut Map<String, Value>,
) -> Result<(), Error> {
    copy_field(src, dst, "description");
    copy_field(src, dst, "name");
    if let Some(paths) = src.get("paths") {
        dst.insert("paths".to_string(), paths.clone());
        dst.insert("trigger".to_string(), json!("auto"));
    } else if let Some(trigger) = src.get("trigger").and_then(|v| v.as_str()) {
        let mapped = match trigger {
            "always" | "always_on" => "always",
            "auto" | "glob" | "model_decision" => "auto",
            "manual" => "manual",
            other => other,
        };
        dst.insert("trigger".to_string(), json!(mapped));
    } else {
        dst.insert("trigger".to_string(), json!("always"));
    }
    if let Some(keywords) = src.get("keywords") {
        dst.insert("keywords".to_string(), keywords.clone());
    }
    Ok(())
}

fn convert_amazonq_to_agents(
    src: &Map<String, Value>,
    dst: &mut Map<String, Value>,
) -> Result<(), Error> {
    copy_field(src, dst, "description");
    copy_field(src, dst, "name");
    dst.insert("trigger".to_string(), json!("always"));
    Ok(())
}

fn from_agents(
    agents: &Map<String, Value>,
    target: RuleFormat,
) -> Result<Map<String, Value>, Error> {
    let mut out = Map::new();
    match target {
        RuleFormat::Agents | RuleFormat::Auto => {
            for (k, v) in agents {
                out.insert(k.clone(), v.clone());
            }
        }
        RuleFormat::Cursor => {
            copy_field(agents, &mut out, "description");
            if let Some(paths) = agents.get("paths") {
                out.insert("globs".to_string(), paths.clone());
            }
            let trigger = agents
                .get("trigger")
                .and_then(|v| v.as_str())
                .unwrap_or("always");
            match trigger {
                "always" => {
                    out.insert("alwaysApply".to_string(), json!(true));
                }
                "auto" => {
                    out.insert("alwaysApply".to_string(), json!(false));
                }
                "manual" => {}
                _ => {}
            }
        }
        RuleFormat::Windsurf => {
            copy_field(agents, &mut out, "description");
            if let Some(paths) = agents.get("paths") {
                out.insert("globs".to_string(), paths.clone());
            }
            let trigger = agents
                .get("trigger")
                .and_then(|v| v.as_str())
                .unwrap_or("always");
            let ws_trigger = match trigger {
                "always" => "always_on",
                "auto" => "glob",
                "manual" => "manual",
                _ => "always_on",
            };
            out.insert("trigger".to_string(), json!(ws_trigger));
        }
        RuleFormat::Copilot => {
            copy_field(agents, &mut out, "description");
            if let Some(paths) = agents.get("paths").and_then(|v| v.as_array()) {
                let joined: Vec<&str> = paths.iter().filter_map(|p| p.as_str()).collect();
                if !joined.is_empty() {
                    out.insert("applyTo".to_string(), json!(joined.join(",")));
                }
            }
        }
        RuleFormat::Claude | RuleFormat::Cline => {
            if let Some(paths) = agents.get("paths") {
                out.insert("paths".to_string(), paths.clone());
            }
        }
        RuleFormat::Jetbrains | RuleFormat::AmazonQ => {
            copy_field(agents, &mut out, "description");
            copy_field(agents, &mut out, "name");
            if let Some(paths) = agents.get("paths") {
                out.insert("paths".to_string(), paths.clone());
            }
        }
    }

    Ok(out)
}

fn copy_field(src: &Map<String, Value>, dst: &mut Map<String, Value>, field: &str) {
    if let Some(v) = src.get(field) {
        dst.insert(field.to_string(), v.clone());
    }
}

fn copy_optional_fields(src: &Map<String, Value>, dst: &mut Map<String, Value>, fields: &[&str]) {
    for field in fields {
        copy_field(src, dst, field);
    }
}

fn assemble_markdown(frontmatter: &Map<String, Value>, body: &str) -> Result<String, Error> {
    if frontmatter.is_empty() {
        return Ok(body.to_string());
    }
    let yaml = serde_saphyr::to_string(frontmatter).map_err(|e| Error::Yaml(e.to_string()))?;
    let trimmed_body = body.trim_start_matches('\n');
    Ok(format!("---\n{yaml}---\n\n{trimmed_body}"))
}

fn normalized_stem(relative: &Path, target: RuleFormat) -> Result<String, Error> {
    let file_name = relative
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Migrate("invalid file name".to_string()))?;

    let stem = if file_name.ends_with(".instructions.md") {
        file_name.trim_end_matches(".instructions.md")
    } else {
        relative
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name)
    };

    let normalized = if target == RuleFormat::Agents {
        stem.to_ascii_lowercase().replace('_', "-")
    } else {
        stem.to_string()
    };

    Ok(normalized)
}

fn output_relative(relative: &Path, target: RuleFormat) -> PathBuf {
    let stem = relative
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("rule");
    let parent = relative.parent().unwrap_or(Path::new(""));
    let normalized_stem = stem.to_ascii_lowercase().replace('_', "-");

    let file_name = match target {
        RuleFormat::Cursor => format!("{normalized_stem}.mdc"),
        RuleFormat::Copilot => format!("{normalized_stem}.instructions.md"),
        _ => format!("{normalized_stem}.md"),
    };

    parent.join(file_name)
}

/// Build [`InputRule`] entries by scanning tool directories or an explicit path.
///
/// When `explicit_dir` is `None`, scans [`crate::discover::existing_tool_dirs`] under
/// `project_root` (skipping canonical `.agents/rules` as a source).
pub fn build_inputs_from_dirs(
    project_root: &Path,
    explicit_dir: Option<&Path>,
    from_hint: RuleFormat,
) -> Result<Vec<InputRule>, Error> {
    let mut inputs = Vec::new();

    if let Some(dir) = explicit_dir {
        let dir = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            project_root.join(dir)
        };
        let tool_dir = find_tool_dir(&dir);
        let format = tool_dir
            .map(|d| d.format)
            .or_else(|| infer_format_from_path(&dir))
            .unwrap_or(from_hint);

        if let Some(td) = tool_dir {
            for file in crate::walk::walk_tool_dir(&dir, td)? {
                let content = std::fs::read_to_string(&file.path)?;
                inputs.push(InputRule {
                    source_path: file.path,
                    relative_path: file.relative,
                    format,
                    content,
                });
            }
        } else {
            for file in crate::walk::walk_md_files(&dir)? {
                let relative = file.strip_prefix(&dir).unwrap_or(&file).to_path_buf();
                let content = std::fs::read_to_string(&file)?;
                inputs.push(InputRule {
                    source_path: file,
                    relative_path: relative,
                    format,
                    content,
                });
            }
        }
    } else {
        for (dir_path, tool_dir) in crate::discover::existing_tool_dirs(project_root) {
            if tool_dir.format == RuleFormat::Agents {
                continue;
            }
            for file in crate::walk::walk_tool_dir(&dir_path, tool_dir)? {
                let content = std::fs::read_to_string(&file.path)?;
                inputs.push(InputRule {
                    source_path: file.path,
                    relative_path: file.relative,
                    format: tool_dir.format,
                    content,
                });
            }
        }
    }

    Ok(inputs)
}
