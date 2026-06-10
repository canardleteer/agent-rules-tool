//! Parse markdown rule files into YAML frontmatter and body text.

use crate::error::Error;
use serde_json::Value;

/// Parsed rule file: JSON frontmatter value and markdown body.
#[derive(Debug, Clone)]
pub struct ParsedRule {
    /// Frontmatter converted to JSON for schema validation.
    pub frontmatter: Value,
    /// Markdown content after the closing `---`.
    pub body: String,
}

/// Parse a rule file string into frontmatter and body.
///
/// Empty or missing frontmatter becomes JSON `null`.
pub fn parse_rule(content: &str) -> Result<ParsedRule, Error> {
    let (yaml_value, body) = markdown_frontmatter::parse::<serde_yaml::Value>(content)?;
    let frontmatter = yaml_to_json(yaml_value)?;
    Ok(ParsedRule {
        frontmatter,
        body: body.to_string(),
    })
}

fn yaml_to_json(yaml: serde_yaml::Value) -> Result<Value, Error> {
    match yaml {
        serde_yaml::Value::Null => Ok(Value::Null),
        other => serde_json::to_value(other).map_err(|e| Error::Yaml(e.to_string())),
    }
}

/// Returns `true` when frontmatter is null or an empty object.
pub fn is_empty_frontmatter(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Object(map) => map.is_empty(),
        _ => false,
    }
}
