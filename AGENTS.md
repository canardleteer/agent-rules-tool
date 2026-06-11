# AGENTS.md

This repository implements **agent-rules-tool**, a CLI and library for
linting and migrating AI agent rule files per
[agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec).

## Vendored spec

Vendored [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec) artifacts
live under `spec/`. Provenance is recorded in [`spec/index.yaml`](spec/index.yaml)
— use its `commit` and `files` fields as the source of truth for the pinned
upstream revision and vendored file manifest.

## Module map

| Module | Purpose |
|--------|---------|
| `spec.rs` | Spec URLs, embedded schema, constants |
| `parse.rs` | Markdown + YAML frontmatter parsing |
| `schema.rs` | JSON Schema validation via `jsonschema` |
| `lint.rs` | Lint orchestration and RFC semantic checks |
| `report.rs` | YAML report serialization (`serde-saphyr`) |
| `format.rs` | `RuleFormat` enum and auto-detection |
| `migrate.rs` | Per-tool field mapping and migration validation |
| `discover.rs` | Known tool directory table |
| `walk.rs` | Recursive rule file discovery |
| `io.rs` | Safe atomic file writes |
| `cli.rs` | `clap` CLI definitions |
| `main.rs` | Async entrypoint (`tokio`) with `tracing` |

## Conventions

* Every lint violation carries a `spec_ref` citing the RFC or schema.
* Use `tracing` (`error!`, `warn!`, `info!`) for diagnostics in the CLI.
* Library APIs accept strings; file I/O is CLI-only.
* Default lint target: `.agents/rules/`. Default migrate output: `.agents/rules/`.
* Do not overwrite output files without `--force`.
* Refresh vendored spec: see [docs/maintenance.md](docs/maintenance.md).

## Documentation (`docs/`)

Long-form guides live under [`docs/`](docs/). This list will grow — after any
change, **check whether an existing doc covers the topic** and update it when
behavior, commands, or maintainer workflow changed.

| Doc | Topics covered — review when you change… |
|-----|------------------------------------------|
| [`maintenance.md`](docs/maintenance.md) | Vendored `spec/`, `cargo xtask spec-update`, `check-spec` workflow |
| [`releases.md`](docs/releases.md) | release-plz, crates.io publishing, release workflow on `main` |
| [`third-party-specs.md`](docs/third-party-specs.md) | External agent rule formats, migration sources, doc links |

When adding a new concern that maintainers or contributors need to read later,
prefer a new `docs/<topic>.md` (and link it here) over duplicating detail in
`AGENTS.md` or the README.

## Agent rules

Structured rules for this repo live in [`.agents/rules/`](.agents/rules/).
Agents **must follow these rules** when making changes, **even if the tool
does not natively load [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec)**
(Cursor rules, plain AGENTS.md-only agents, etc.).

| Rule | Applies when | Action |
|------|--------------|--------|
| `rust-quality` | `**/*.rs` changes | fmt → clippy → lint rules → test → rustdoc |
| `markdown-quality` | `**/*.md` changes | `rumdl check .` |
| `agent-rules-lint` | `.agents/rules/**` changes | rumdl → `cargo run … lint` (see rule for fallback) |
| `workflow-sync` | CI/config paths (see rule) | keep `.github/workflows/` aligned with quality rules and `.rumdl.toml` |

## CI

Pull-request gates live in [`.github/workflows/ci.yml`](.github/workflows/ci.yml):
path-aware jobs (via `dorny/paths-filter`) share one workflow so skipped jobs
still satisfy required checks. Required status check names on `main`:

* `CI (ubuntu-latest)` — Rust fmt, clippy, rule lint, tests, rustdoc, publish
  dry-run, audit
* `CI (macos-latest)` — same Rust pipeline on macOS
* `Markdown Hygiene` — rumdl when `**/*.md` or `.rumdl.toml` changes
* `Check Spec` — `cargo xtask spec-update check` when vendored spec paths change

Post-merge / maintainer workflows (not MR gates): [`check-spec.yml`](.github/workflows/check-spec.yml)
(weekly drift), [`release-plz.yml`](.github/workflows/release-plz.yml) (releases).
See [`.agents/rules/workflow-sync.md`](.agents/rules/workflow-sync.md) when editing CI.

For rule linting: if `cargo run` fails, alert the user that the dev path is
broken; if the installed binary is also unavailable, alert the user that
neither path works.
