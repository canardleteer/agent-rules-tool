---
name: rust-quality
description: Run fmt, clippy, rule lint, tests, and rustdoc after Rust changes (fast fail)
trigger: auto
paths:
  - "**/*.rs"
---

# Rust quality

After any `.rs` change, run these commands from the repo root in order (fast
fail — stop and fix at the first failure):

1. `cargo fmt --all` — apply formatting
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   — fix all warnings and errors; do not add `allow` attributes unless the user
   explicitly asks
3. `cargo run --bin agent-rules-tool -- lint -d .agents/rules` — fix spec
   violations in [`.agents/rules/`](../../.agents/rules/); if `cargo run` fails,
   follow the fallback and user-alert steps in [`agent-rules-lint.md`](agent-rules-lint.md)
4. `cargo test --workspace --all-targets` — fix failures; never skip, delete,
   or `#[ignore]` tests to get green; if stuck, ask the user for help
5. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p agent-rules-tool
   --all-features` — fix rustdoc warnings and errors

This order matches [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml).
