//! Error types returned by parse, lint, migrate, and I/O operations.

use thiserror::Error;

/// Error returned by library and CLI operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Markdown frontmatter could not be parsed.
    #[error("parse error: {0}")]
    Parse(#[from] markdown_frontmatter::Error),

    /// JSON Schema compilation or validation failed unexpectedly.
    #[error("schema error: {0}")]
    Schema(String),

    /// File or directory I/O failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML serialization or deserialization failed.
    #[error("yaml error: {0}")]
    Yaml(String),

    /// JSON parsing failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Migration between formats failed.
    #[error("migrate error: {0}")]
    Migrate(String),

    /// Caller input was invalid (missing fields, unsupported combination, etc.).
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
