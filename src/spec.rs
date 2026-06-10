//! Spec citations, URLs, and embedded vendored artifacts from
//! [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec).

/// Upstream specification repository URL.
pub const SPEC_REPO: &str = "https://github.com/rameshsunkara/agent-rules-spec";
/// Compile-time mirror of `commit` in [`spec/index.yaml`](../spec/index.yaml).
pub const SPEC_COMMIT: &str = "f23a86b4df8f2455e121cea88ce0d48a73a8f990";
/// Relative path to the JSON Schema within the upstream repository.
pub const SCHEMA_PATH: &str = "schema/agent-rule.schema.json";
/// Canonical URL for the agent rule JSON Schema document.
pub const SCHEMA_URL: &str =
    "https://github.com/rameshsunkara/agent-rules-spec/blob/main/schema/agent-rule.schema.json";
/// RFC section on frontmatter fields.
pub const RFC_FRONTMATTER_FIELDS: &str =
    "https://github.com/rameshsunkara/agent-rules-spec/blob/main/RFC.md#frontmatter-fields";
/// RFC section on trigger modes (`always`, `auto`, `manual`).
pub const RFC_TRIGGER_MODES: &str =
    "https://github.com/rameshsunkara/agent-rules-spec/blob/main/RFC.md#trigger-modes";
/// RFC section on file naming and layout conventions.
pub const RFC_FILE_CONVENTIONS: &str =
    "https://github.com/rameshsunkara/agent-rules-spec/blob/main/RFC.md#file-conventions";
/// URL for the tool compatibility field mapping document.
pub const MAPPING_URL: &str =
    "https://github.com/rameshsunkara/agent-rules-spec/blob/main/compatibility/mapping.md";

/// Embedded JSON Schema text from `spec/agent-rule.schema.json`.
pub const EMBEDDED_SCHEMA: &str = include_str!("../spec/agent-rule.schema.json");
/// Embedded `spec/index.yaml` manifest text.
pub const EMBEDDED_INDEX: &str = include_str!("../spec/index.yaml");

/// Default directory scanned by the CLI `lint` subcommand.
pub const DEFAULT_LINT_DIR: &str = ".agents/rules";
/// Default directory written by the CLI `migrate` subcommand.
pub const DEFAULT_OUTPUT_DIR: &str = ".agents/rules";
