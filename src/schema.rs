//! JSON Schema validation of rule frontmatter against the embedded schema.

use crate::error::Error;
use crate::spec::{EMBEDDED_SCHEMA, SCHEMA_URL};
use crate::{Severity, Violation};
use jsonschema::Validator;
use serde_json::Value;
use std::sync::LazyLock;

static VALIDATOR: LazyLock<Validator> = LazyLock::new(|| {
    let schema: Value =
        serde_json::from_str(EMBEDDED_SCHEMA).expect("embedded schema is valid JSON");
    Validator::new(&schema).expect("embedded schema compiles")
});

/// Validate `frontmatter` against the embedded agent-rules-spec JSON Schema.
///
/// Returns an empty vector for null or empty-object frontmatter.
pub fn validate_frontmatter(frontmatter: &Value) -> Result<Vec<Violation>, Error> {
    if matches!(frontmatter, Value::Null) {
        return Ok(Vec::new());
    }
    if let Value::Object(map) = frontmatter
        && map.is_empty()
    {
        return Ok(Vec::new());
    }

    let violations: Vec<Violation> = VALIDATOR
        .iter_errors(frontmatter)
        .map(|err| {
            let instance_path = err.instance_path().to_string();
            let field = if instance_path.is_empty() {
                "frontmatter".to_string()
            } else {
                instance_path.trim_start_matches('/').to_string()
            };
            Violation {
                severity: Severity::Error,
                field,
                message: err.to_string(),
                spec_ref: SCHEMA_URL,
            }
        })
        .collect();
    Ok(violations)
}
