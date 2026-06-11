# Maintaining vendored spec

Refresh `spec/` from [agent-rules-spec](https://github.com/rameshsunkara/agent-rules-spec).

All commands accept optional `--ref <branch|tag|commit>`. Without it, they
use the upstream repository's default branch.

```bash
# Re-discover schema + examples/*.md and rebuild spec/index.yaml
cargo xtask spec-update fresh

# Re-pull files listed in spec/index.yaml
cargo xtask spec-update update

# Check for upstream updates (exit 1 when an update is available)
cargo xtask spec-update check

# Pin to a specific upstream ref (branch, tag, or commit)
cargo xtask spec-update update --ref f23a86b4df8f2455e121cea88ce0d48a73a8f990
cargo xtask spec-update fresh --ref main
cargo xtask spec-update check --ref main
```

`fresh` and `update` update `spec/`, `spec/index.yaml`, and `src/spec.rs`
(`SPEC_COMMIT`). See `spec/index.yaml` for the pinned upstream commit.

On pull requests that touch `spec/`, `xtask/`, or `docs/maintenance.md`, the
`Check Spec` job in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)
runs the same `check` command. [`.github/workflows/check-spec.yml`](../.github/workflows/check-spec.yml)
also runs it weekly and on manual dispatch for upstream drift.
