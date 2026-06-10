---
name: markdown-quality
description: Run rumdl after Markdown file changes
trigger: auto
paths:
  - "**/*.md"
---

# Markdown quality

After any `.md` change:

* Run `rumdl check .` from the repo root
* Fix reported issues in changed files and any others rumdl flags

Vendored upstream files under `spec/` are excluded in [`.rumdl.toml`](../../.rumdl.toml)
and are not checked by rumdl.
