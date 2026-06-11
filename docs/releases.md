# Releases

Releases are automated with [release-plz](https://release-plz.dev/) on every push
to `main`.

## Repository secrets

| Secret | Purpose |
|--------|---------|
| `RELEASE_PLZ_TOKEN` | Classic PAT with `repo` scope. Passed to release-plz as `GITHUB_TOKEN` so [Release PRs trigger CI](https://release-plz.dev/docs/github/token). The default Actions token cannot start check workflows on bot-opened PRs. |
| `CARGO_REGISTRY_TOKEN` | crates.io API token for `cargo publish`. |

Create a classic PAT under GitHub → Settings → Developer settings, then:

```bash
gh secret set RELEASE_PLZ_TOKEN --repo canardleteer/agent-rules-tool
```

## How it works

1. **`release-plz-pr`** opens or updates a Release PR (version bump in
   `Cargo.toml`, `CHANGELOG.md`).
2. Merge that PR when you are ready to ship.
3. **`release-plz-release`** publishes to [crates.io](https://crates.io/crates/agent-rules-tool)
   and creates a GitHub Release/tag. With `release_always = false` in
   [`release-plz.toml`](../release-plz.toml), publish runs only when that
   Release PR merges—not on every push to `main`.
4. **`cd.yml`** runs on `release: published`, builds pre-built CLI binaries per
   [release-plz binary-release docs](https://release-plz.dev/docs/extra/releasing-binaries),
   and uploads them as GitHub Release assets.

Use [conventional commits](https://www.conventionalcommits.org/) on `main` so
release-plz picks the right semver bump (`feat:` minor, `fix:` patch,
`feat!:` / `BREAKING CHANGE:` major).

Release PRs also run **`cargo-semver-checks`** (`semver_check = true` in
[`release-plz.toml`](../release-plz.toml)): public API breaks are flagged on the
PR and can drive a major bump. That check needs a prior crates.io version as a
baseline; it does not catch every semver violation.

## Local package check

CI and [`rust-quality.md`](../.agents/rules/rust-quality.md) run:

```bash
cargo publish -p agent-rules-tool --dry-run
```

This validates manifest metadata and packaging without uploading.
