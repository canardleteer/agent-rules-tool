---
name: agent-rules-lint
description: Lint agent rule files with agent-rules-tool after rule changes
trigger: auto
paths:
  - ".agents/rules/**"
---

# Agent rules lint

After any change under `.agents/rules/`, run checks in order (fast fail):

1. `rumdl check .` — markdown/style (see [`markdown-quality.md`](markdown-quality.md))
2. Lint with this repo's tool (primary):

```bash
cargo run --bin agent-rules-tool -- lint -d .agents/rules
```

For a single edited file, `-i path/to/rule.md` is also valid; directory mode is
fine for any change in the tree.

Fallback if `cargo run` fails (build error, missing toolchain, etc.):

```bash
agent-rules-tool lint -d .agents/rules
```

Use `command -v agent-rules-tool` to confirm an install exists before
falling back.

User alerts (required):

* If `cargo run` fails, tell the user the dev/build path is broken and show
  the error before trying the installed binary.
* If the installed binary is also missing or fails, tell the user neither path
  works and they cannot validate rules until one is fixed (`cargo build`,
  `cargo install --path .`, etc.).

Do not skip linting or ignore spec violations to get green.

When you also changed `**/*.rs`, step 2 is already covered by
[`rust-quality.md`](rust-quality.md) (after fmt and clippy).

CI runs the rust jobs when `.agents/rules/**` changes even without `.rs` edits
(see [`workflow-sync.md`](workflow-sync.md)).
