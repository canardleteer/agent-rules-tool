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
| `CI (ubuntu-latest)` / `CI (macos-latest)` | `**/*.rs`, `**/Cargo.toml`, `rust-toolchain.toml`, `.github/workflows/ci.yml`, `.agents/rules/**` | [`rust-quality.md`](rust-quality.md), [`agent-rules-lint.md`](agent-rules-lint.md) |
| `Markdown Hygiene` | `**/*.md`, `.rumdl.toml` | [`markdown-quality.md`](markdown-quality.md) |
| `Check Spec` | `spec/**`, `xtask/**`, `docs/maintenance.md` | [`docs/maintenance.md`](../../docs/maintenance.md) |

## Other workflows (not MR gates)

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| [`check-spec.yml`](../../.github/workflows/check-spec.yml) | Weekly cron, `workflow_dispatch` | Upstream drift alarm (same command as the PR job) |
| [`release-plz.yml`](../../.github/workflows/release-plz.yml) | Push to `main` | Release PRs and crates.io publishing |

## Sync checklist

| When you change… | Also verify/update… |
|------------------|---------------------|
| [`.rumdl.toml`](../../.rumdl.toml) `include` / `exclude` | `ci.yml` `changes` job `md` filter and rumdl job config |
| [`markdown-quality.md`](markdown-quality.md) rumdl command/config | `ci.yml` rumdl job |
| [`rust-quality.md`](rust-quality.md) cargo commands or `paths` | `ci.yml` `rust` filter and rust job steps (fast-fail order; publish dry-run on ubuntu) |
| [`agent-rules-lint.md`](agent-rules-lint.md) paths or lint command | `ci.yml` `rules` filter (runs the rust job for rule-only edits) |
| [`release-plz.toml`](../../release-plz.toml) | [`release-plz.yml`](../../.github/workflows/release-plz.yml) |
| [`xtask`](../../xtask/) spec-update or [`docs/maintenance.md`](../../docs/maintenance.md) | `ci.yml` `spec` filter and [`check-spec.yml`](../../.github/workflows/check-spec.yml) |
| Any workflow file | Current action majors (`checkout@v6`, `paths-filter@v4`, `rumdl@v0`, etc.); commands aligned with agent rules |
| New third-party action in a workflow | Repository **Settings → Actions → Allow select actions** (or `gh api …/selected-actions`); keep pins in sync with `ci.yml` / `release-plz.yml` |

Allowed action patterns (repo policy: select actions only):

* `actions/checkout@v6`
* `dtolnay/rust-toolchain@stable`
* `Swatinem/rust-cache@v2`
* `dorny/paths-filter@v4`
* `rustsec/audit-check@858dc40f52ca2b8570b7a997c1c4e35c6fc9a432`
* `release-plz/action@v0.5.128`
* `rvben/rumdl@v0`

After editing rule or workflow files, run `rumdl check .` and
`cargo run --bin agent-rules-tool -- lint -d .agents/rules`.
