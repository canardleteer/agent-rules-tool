---
name: workflow-sync
description: Keep GitHub Actions workflows aligned with quality rules and config
trigger: auto
paths:
  - ".github/workflows/**"
  - ".rumdl.toml"
  - ".agents/rules/rust-quality.md"
  - ".agents/rules/markdown-quality.md"
  - ".agents/rules/agent-rules-lint.md"
  - "xtask/**"
  - "docs/maintenance.md"
  - "**/Cargo.toml"
  - "rust-toolchain.toml"
  - "release-plz.toml"
---

# Workflow sync

When changing files that affect CI, keep workflows and agent rules aligned.

## PR workflow (`ci.yml`)

Pull requests and pushes to `main` run [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml).
A `changes` job uses [`dorny/paths-filter`](https://github.com/dorny/paths-filter) so
downstream jobs **skip** (and still satisfy required checks) when irrelevant paths
did not change. Do **not** add workflow-level `paths:` filters to required checks.

| Job (required check name) | Runs when paths match… | Agent rule / command |
|---------------------------|------------------------|----------------------|
| `CI (ubuntu-latest)` / `CI (macos-latest)` / `CI (windows-latest)` | `**/*.rs`, `**/Cargo.toml`, `examples/**`, `rust-toolchain.toml`, `.github/workflows/ci.yml`, `.agents/rules/**` | [`rust-quality.md`](rust-quality.md), [`agent-rules-lint.md`](agent-rules-lint.md). Separate jobs (not a matrix) so skips match required check names. `cargo fmt` / `clippy --all-targets` cover `examples/`. |
| `Markdown Hygiene` | `**/*.md`, `.rumdl.toml` | [`markdown-quality.md`](markdown-quality.md) |
| `Check Spec` | `spec/**`, `xtask/**`, `docs/maintenance.md` | [`docs/maintenance.md`](../../docs/maintenance.md) |

## Other workflows (not MR gates)

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| [`check-spec.yml`](../../.github/workflows/check-spec.yml) | Weekly cron, `workflow_dispatch` | Upstream drift alarm (same command as the PR job) |
| [`release-plz.yml`](../../.github/workflows/release-plz.yml) | Push to `main` | Release PRs and crates.io publishing |
| [`cd.yml`](../../.github/workflows/cd.yml) | `release: published` | Build and upload release binaries (see [releases.md](../../docs/releases.md)) |

## Sync checklist

| When you change… | Also verify/update… |
|------------------|---------------------|
| [`.rumdl.toml`](../../.rumdl.toml) `include` / `exclude` | `ci.yml` `changes` job `md` filter and rumdl job config |
| [`markdown-quality.md`](markdown-quality.md) rumdl command/config | `ci.yml` rumdl job |
| [`rust-quality.md`](rust-quality.md) cargo commands or `paths` | `ci.yml` `rust` filter and rust job steps (fast-fail order; publish dry-run on ubuntu) |
| [`agent-rules-lint.md`](agent-rules-lint.md) paths or lint command | `ci.yml` `rules` filter (runs the rust job for rule-only edits) |
| [`release-plz.toml`](../../release-plz.toml) | [`release-plz.yml`](../../.github/workflows/release-plz.yml); `RELEASE_PLZ_TOKEN` secret (see [`docs/releases.md`](../../docs/releases.md)) |
| [`release-plz/action`](../../.github/workflows/release-plz.yml) version bump | Transitive allowlist pins below (read `action.yml` for SHAs) |
| [`xtask`](../../xtask/) spec-update or [`docs/maintenance.md`](../../docs/maintenance.md) | `ci.yml` `spec` filter and [`check-spec.yml`](../../.github/workflows/check-spec.yml) |
| Any workflow file | Current action majors (`checkout@v6`, `paths-filter@v4`, `rumdl@v0`, etc.); commands aligned with agent rules |
| New workflow file, new `uses:` action, or allowlist change | [AGENTS.md § New workflows](../../AGENTS.md#new-workflows-and-third-party-actions): explicit yes required — **mid-debug / "fix CI" is not approval** |

Allowed action patterns (repo policy: select actions only; **do not extend
without explicit user approval** — debugging a failure does not count; see
[AGENTS.md](../../AGENTS.md#new-workflows-and-third-party-actions)):

* `actions/checkout@v6`
* `dtolnay/rust-toolchain@stable`
* `Swatinem/rust-cache@v2`
* `dorny/paths-filter@v4`
* `rustsec/audit-check@858dc40f52ca2b8570b7a997c1c4e35c6fc9a432`
* `release-plz/action@v0.5.128`
* `release-plz/git-config@59144859caf016f8b817a2ac9b051578729173c4`
  (transitive via release-plz)
* `taiki-e/install-action@f23382d582832e41d5eb4fff2bddb06bc5adf8d3`
  (transitive via release-plz)
* `taiki-e/install-action@56545b37b57562edd73171cb6c62cc509db4c34e`
  (cd.yml `cross` for musl targets)
* `taiki-e/setup-cross-toolchain-action@3d9770ce98eb7dbcf378563182a5e8031165f75b`
  (cd.yml cross gnu targets)
* `taiki-e/upload-rust-binary-action@f0d45ae91ee7b8ee928de7a9d04d893a08bcbec6`
  (cd.yml release assets)
* `cargo-bins/cargo-binstall@1800853f2578f8c34492ec76154caef8e163fbca`
  (transitive via release-plz)
* `rvben/rumdl@v0`

After editing rule or workflow files, run `rumdl check .` and
`cargo run --bin agent-rules-tool -- lint -d .agents/rules`.
