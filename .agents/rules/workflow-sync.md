---
name: workflow-sync
description: Keep GitHub Actions workflows aligned with quality rules and config
trigger: auto
paths:
  - ".github/workflows/**"
  - ".rumdl.toml"
  - ".agents/rules/rust-quality.md"
  - ".agents/rules/markdown-quality.md"
  - "xtask/**"
  - "docs/maintenance.md"
  - "Cargo.toml"
  - "rust-toolchain.toml"
---

# Workflow sync

When changing files that affect CI, keep workflows and agent rules aligned.

| When you change… | Also verify/update… |
|------------------|---------------------|
| [`.rumdl.toml`](../../.rumdl.toml) `include` | [`.github/workflows/rumdl.yml`](../../.github/workflows/rumdl.yml) `paths` filters |
| [`.rumdl.toml`](../../.rumdl.toml) `exclude` | Keep vendored `spec/` out of rumdl (upstream examples, not repo-owned docs) |
| [`markdown-quality.md`](markdown-quality.md) rumdl command/config | `rumdl.yml` action config |
| [`rust-quality.md`](rust-quality.md) cargo commands | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) steps (same fast-fail order) |
| [`xtask`](../../xtask/) spec-update behavior or [`docs/maintenance.md`](../../docs/maintenance.md) | [`.github/workflows/check-spec.yml`](../../.github/workflows/check-spec.yml) |
| Any workflow file | Use current action majors (`checkout@v6`, `rumdl@v0`, etc.); keep commands aligned with agent rules |

After editing rule or workflow files, run `rumdl check .` and
`cargo run --bin agent-rules-tool -- lint -d .agents/rules`.
