//! Rule format identifiers and auto-detection from frontmatter.

use clap::ValueEnum;
use serde_json::Value;

/// CLI-facing rule format selector (mirrors [`RuleFormat`] for `clap`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RuleFormatArg {
    /// Canonical [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec) format.
    Agents,
    /// Cursor `.cursor/rules` format.
    Cursor,
    /// Windsurf `.windsurf/rules` format.
    Windsurf,
    /// GitHub Copilot `.github/instructions` format.
    Copilot,
    /// Cline `.clinerules` format.
    Cline,
    /// Claude `.claude/rules` format.
    Claude,
    /// JetBrains AI Assistant format.
    Jetbrains,
    /// Amazon Q format.
    Amazonq,
    /// Detect format from frontmatter or path.
    Auto,
}

/// Supported agent rule file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleFormat {
    /// Canonical [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec) format.
    Agents,
    /// Cursor `.cursor/rules` format.
    Cursor,
    /// Windsurf `.windsurf/rules` format.
    Windsurf,
    /// GitHub Copilot `.github/instructions` format.
    Copilot,
    /// Cline `.clinerules` format.
    Cline,
    /// Claude `.claude/rules` format.
    Claude,
    /// JetBrains AI Assistant format.
    Jetbrains,
    /// Amazon Q format.
    AmazonQ,
    /// Detect format from frontmatter or path.
    Auto,
}

impl From<RuleFormatArg> for RuleFormat {
    fn from(arg: RuleFormatArg) -> Self {
        match arg {
            RuleFormatArg::Agents => RuleFormat::Agents,
            RuleFormatArg::Cursor => RuleFormat::Cursor,
            RuleFormatArg::Windsurf => RuleFormat::Windsurf,
            RuleFormatArg::Copilot => RuleFormat::Copilot,
            RuleFormatArg::Cline => RuleFormat::Cline,
            RuleFormatArg::Claude => RuleFormat::Claude,
            RuleFormatArg::Jetbrains => RuleFormat::Jetbrains,
            RuleFormatArg::Amazonq => RuleFormat::AmazonQ,
            RuleFormatArg::Auto => RuleFormat::Auto,
        }
    }
}

impl RuleFormat {
    /// Default output directory for this format when translating to disk.
    pub fn default_output_dir(self) -> &'static str {
        match self {
            RuleFormat::Agents => ".agents/rules",
            RuleFormat::Cursor => ".cursor/rules",
            RuleFormat::Windsurf => ".windsurf/rules",
            RuleFormat::Copilot => ".github/instructions",
            RuleFormat::Cline => ".clinerules",
            RuleFormat::Claude => ".claude/rules",
            RuleFormat::Jetbrains => ".aiassistant/rules",
            RuleFormat::AmazonQ => ".amazonq/rules",
            RuleFormat::Auto => ".agents/rules",
        }
    }

    /// Infer format from frontmatter shape (heuristic field matching).
    pub fn detect_from_frontmatter(frontmatter: &Value) -> RuleFormat {
        if frontmatter.is_null() {
            return RuleFormat::Agents;
        }
        let obj = match frontmatter.as_object() {
            Some(o) if !o.is_empty() => o,
            _ => return RuleFormat::Agents,
        };

        if obj.contains_key("applyTo") {
            return RuleFormat::Copilot;
        }

        if obj.contains_key("globs") {
            if obj.contains_key("alwaysApply") {
                return RuleFormat::Cursor;
            }
            if let Some(trigger) = obj.get("trigger").and_then(|v| v.as_str())
                && matches!(trigger, "glob" | "always_on" | "manual" | "model_decision")
            {
                return RuleFormat::Windsurf;
            }
            return RuleFormat::Cursor;
        }

        if obj.contains_key("paths") {
            if let Some(trigger) = obj.get("trigger").and_then(|v| v.as_str())
                && matches!(trigger, "always" | "auto" | "manual")
            {
                return RuleFormat::Agents;
            }
            return RuleFormat::Claude;
        }

        if obj.contains_key("trigger") {
            return RuleFormat::Agents;
        }

        RuleFormat::Agents
    }

    /// Resolve [`RuleFormat::Auto`] via [`Self::detect_from_frontmatter`]; otherwise return `self`.
    pub fn resolve(self, frontmatter: &Value) -> RuleFormat {
        if self == RuleFormat::Auto {
            Self::detect_from_frontmatter(frontmatter)
        } else {
            self
        }
    }
}
