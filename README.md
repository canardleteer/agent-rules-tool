# agent-rules-tool

> [!IMPORTANT]
> Initial tool scaffolding.

> [!WARNING]
> Clanker generated code.

Lint and migrate AI agent rule files using the
[agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec)
structured rules format.

* **RFC:** [RFC.md](https://github.com/rameshsunkara/agent-rules-spec/blob/main/RFC.md)
* **Schema:** [agent-rule.schema.json](https://github.com/rameshsunkara/agent-rules-spec/blob/main/schema/agent-rule.schema.json)
* **Mappings:** [compatibility/mapping.md](https://github.com/rameshsunkara/agent-rules-spec/blob/main/compatibility/mapping.md)

## Install

```bash
cargo install --path .
```

Or build locally:

```bash
cargo build --release
```

## CLI

### Lint

By default, lints all `*.md` files under `.agents/rules/`:

```bash
agent-rules-tool lint
agent-rules-tool lint -d path/to/rules
agent-rules-tool lint -i path/to/rule.md
agent-rules-tool lint --severity warn --report report.yaml
```

* `--severity error` (default): exit `1` if any error-level violation is found
* `--severity warn`: exit `1` on warnings or errors
* `--report`: write a YAML report (aggregate for directory mode,
  single-file for `-i`)

### Migrate

By default, scans known tool-native rule directories and writes canonical
rules to `.agents/rules/`:

```bash
agent-rules-tool migrate auto
agent-rules-tool migrate cursor -d .cursor/rules
agent-rules-tool migrate auto -i .cursor/rules/api.mdc -o .agents/rules/api.md
agent-rules-tool migrate agents --to cursor -d .agents/rules
agent-rules-tool migrate auto --force
```

Unknown frontmatter keys error with a link to file an issue on
[agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec/issues).
Lossy or ambiguous field mappings emit warnings.

* `<type>`: source format (`agents`, `cursor`, `windsurf`, `copilot`,
  `cline`, `claude`, `jetbrains`, `amazonq`, or `auto`)
* `--to`: target format (default: `agents`)
* `-o`: output directory (dir mode) or file (`-i` mode)
* `--force`: overwrite existing output files

Output directories are created automatically. Existing files are not
overwritten unless `--force` is set.

## Third-party specifications

Native rule formats and documentation links for each supported agent. See
[docs/third-party-specs.md](docs/third-party-specs.md).

## Releases

Published to [crates.io/crates/agent-rules-tool](https://crates.io/crates/agent-rules-tool)
via [release-plz](https://release-plz.dev/) on merge to `main`. See
[docs/releases.md](docs/releases.md) for the release workflow.

## Maintaining vendored spec

See [docs/maintenance.md](docs/maintenance.md).

## Library

```rust
use agent_rules_tool::{lint_string, migrate_string, LintOptions, MigrateOptions};
use agent_rules_tool::format::RuleFormat;

let content = std::fs::read_to_string("rule.md")?;
let report = lint_string(&content, &LintOptions::default())?;

let migrated = migrate_string(
    &content,
    &MigrateOptions {
        from: RuleFormat::Cursor,
        to: RuleFormat::Agents,
        ..Default::default()
    },
)?;
for warning in &migrated.warnings {
    eprintln!("{}: {}", warning.field.as_deref().unwrap_or("-"), warning.message);
}
```

## Credits

This tool implements the [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec)
structured rules format. The specification was authored by
[Ram](https://github.com/rameshsunkara).
