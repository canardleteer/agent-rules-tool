//! Known tool rule directories and format inference from paths.

use crate::RuleFormat;
use std::path::{Path, PathBuf};

/// Metadata for a known agent tool rule directory.
#[derive(Debug, Clone)]
pub struct ToolRuleDir {
    /// Relative path segment (e.g. `.cursor/rules`).
    pub path: &'static str,
    /// Rule format stored in this directory.
    pub format: RuleFormat,
    /// Allowed file extensions when no suffix is set.
    pub extensions: &'static [&'static str],
    /// Optional filename suffix (e.g. Copilot `.instructions.md`).
    pub suffix: Option<&'static str>,
}

/// Known tool rule directories from [mapping.md](https://github.com/rameshsunkara/agent-rules-spec/blob/main/compatibility/mapping.md).
pub const TOOL_RULE_DIRS: &[ToolRuleDir] = &[
    ToolRuleDir {
        path: ".agents/rules",
        format: RuleFormat::Agents,
        extensions: &["md"],
        suffix: None,
    },
    ToolRuleDir {
        path: ".cursor/rules",
        format: RuleFormat::Cursor,
        extensions: &["mdc", "md"],
        suffix: None,
    },
    ToolRuleDir {
        path: ".windsurf/rules",
        format: RuleFormat::Windsurf,
        extensions: &["md"],
        suffix: None,
    },
    ToolRuleDir {
        path: ".github/instructions",
        format: RuleFormat::Copilot,
        extensions: &["md"],
        suffix: Some(".instructions.md"),
    },
    ToolRuleDir {
        path: ".clinerules",
        format: RuleFormat::Cline,
        extensions: &["md"],
        suffix: None,
    },
    ToolRuleDir {
        path: ".claude/rules",
        format: RuleFormat::Claude,
        extensions: &["md"],
        suffix: None,
    },
    ToolRuleDir {
        path: ".aiassistant/rules",
        format: RuleFormat::Jetbrains,
        extensions: &["md"],
        suffix: None,
    },
    ToolRuleDir {
        path: ".amazonq/rules",
        format: RuleFormat::AmazonQ,
        extensions: &["md"],
        suffix: None,
    },
];

/// Infer [`RuleFormat`] when `path` contains a known tool directory segment.
pub fn infer_format_from_path(path: &Path) -> Option<RuleFormat> {
    let path_str = path.to_string_lossy().replace('\\', "/");
    for dir in TOOL_RULE_DIRS {
        if path_str.contains(dir.path) {
            return Some(dir.format);
        }
    }
    None
}

/// Return the [`ToolRuleDir`] entry matching `path`, if any.
pub fn find_tool_dir(path: &Path) -> Option<&'static ToolRuleDir> {
    let path_str = path.to_string_lossy().replace('\\', "/");
    TOOL_RULE_DIRS
        .iter()
        .find(|dir| path_str.contains(dir.path))
}

/// List tool rule directories that exist under `project_root`.
pub fn existing_tool_dirs(project_root: &Path) -> Vec<(PathBuf, &'static ToolRuleDir)> {
    TOOL_RULE_DIRS
        .iter()
        .filter_map(|dir| {
            let full = project_root.join(dir.path);
            if full.exists() {
                Some((full, dir))
            } else {
                None
            }
        })
        .collect()
}
