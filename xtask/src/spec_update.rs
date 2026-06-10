use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SPEC_REPO: &str = "https://github.com/rameshsunkara/agent-rules-spec";
const SCHEMA_UPSTREAM: &str = "schema/agent-rule.schema.json";
const SCHEMA_VENDORED: &str = "agent-rule.schema.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpecIndex {
    repository: String,
    commit: String,
    files: Vec<SpecIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpecIndexEntry {
    vendored: String,
    upstream: String,
}

pub fn run_fresh(git_ref: Option<&str>) -> Result<()> {
    let root = workspace_root()?;
    let spec_dir = root.join("spec");
    let (clone_dir, commit) = shallow_clone(git_ref)?;
    let entries = discover_upstream_files(clone_dir.path())?;
    sync_vendored_files(clone_dir.path(), &spec_dir, &entries)?;
    prune_stale_vendored(&spec_dir, &entries)?;
    write_index(&spec_dir, &commit, &entries)?;
    update_pinned_commit(&root, &commit)?;
    print_summary("refreshed", git_ref, &commit, entries.len());
    Ok(())
}

pub fn run_update(git_ref: Option<&str>) -> Result<()> {
    let root = workspace_root()?;
    let spec_dir = root.join("spec");
    let index_path = spec_dir.join("index.yaml");
    let existing = read_index(&index_path).context("read spec/index.yaml")?;
    let (clone_dir, commit) = shallow_clone(git_ref)?;
    sync_vendored_files(clone_dir.path(), &spec_dir, &existing.files)?;
    write_index(&spec_dir, &commit, &existing.files)?;
    update_pinned_commit(&root, &commit)?;
    print_summary("updated", git_ref, &commit, existing.files.len());
    Ok(())
}

/// Compare `spec/index.yaml` against upstream. Exit successfully when current.
pub fn run_check(git_ref: Option<&str>) -> Result<()> {
    let root = workspace_root()?;
    let index_path = root.join("spec/index.yaml");
    let index = read_index(&index_path).context("read spec/index.yaml")?;
    let (clone_dir, upstream_commit) = shallow_clone(git_ref)?;
    let upstream_files = discover_upstream_files(clone_dir.path())?;

    let mut issues = 0usize;

    if index.commit != upstream_commit {
        issues += 1;
        println!("update available:");
        println!("  vendored: spec/index.yaml commit {}", index.commit);
        match git_ref {
            Some(reference) => println!("  upstream: {reference} @ {upstream_commit}"),
            None => println!("  upstream: default branch @ {upstream_commit}"),
        }
        println!("  run: cargo xtask spec-update update{}", ref_hint(git_ref));
    } else {
        println!(
            "spec/ is current with upstream @ {} ({})",
            upstream_commit,
            git_ref
                .map(|reference| format!("ref: {reference}"))
                .unwrap_or_else(|| "default branch".to_string())
        );
    }

    let manifest_diff = manifest_diff(&index.files, &upstream_files);
    if !manifest_diff.is_empty() {
        issues += 1;
        println!("upstream file manifest differs from spec/index.yaml:");
        for line in manifest_diff {
            println!("  {line}");
        }
        println!("  run: cargo xtask spec-update fresh{}", ref_hint(git_ref));
    }

    if let Some(spec_rs_commit) = read_spec_rs_commit(&root.join("src/spec.rs"))?
        && spec_rs_commit != index.commit
    {
        issues += 1;
        println!(
            "drift: src/spec.rs SPEC_COMMIT ({spec_rs_commit}) != spec/index.yaml commit ({})",
            index.commit
        );
        println!("  run: cargo xtask spec-update update{}", ref_hint(git_ref));
    }

    if issues > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn ref_hint(git_ref: Option<&str>) -> String {
    git_ref
        .map(|reference| format!(" --ref {reference}"))
        .unwrap_or_default()
}

fn manifest_diff(local: &[SpecIndexEntry], upstream: &[SpecIndexEntry]) -> Vec<String> {
    use std::collections::{HashMap, HashSet};

    let local_map: HashMap<_, _> = local
        .iter()
        .map(|entry| (entry.vendored.as_str(), entry.upstream.as_str()))
        .collect();
    let upstream_map: HashMap<_, _> = upstream
        .iter()
        .map(|entry| (entry.vendored.as_str(), entry.upstream.as_str()))
        .collect();

    let local_keys: HashSet<_> = local_map.keys().copied().collect();
    let upstream_keys: HashSet<_> = upstream_map.keys().copied().collect();

    let mut lines = Vec::new();
    for key in upstream_keys.difference(&local_keys) {
        lines.push(format!("missing locally: {key}"));
    }
    for key in local_keys.difference(&upstream_keys) {
        lines.push(format!("stale locally: {key}"));
    }
    for key in local_keys.intersection(&upstream_keys) {
        if local_map[key] != upstream_map[key] {
            lines.push(format!(
                "upstream path changed for {key}: {} -> {}",
                local_map[key], upstream_map[key]
            ));
        }
    }
    lines.sort();
    lines
}

fn read_spec_rs_commit(path: &Path) -> Result<Option<String>> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let needle = "pub const SPEC_COMMIT: &str = \"";
    let Some(start) = content.find(needle) else {
        return Ok(None);
    };
    let value_start = start + needle.len();
    let Some(end) = content[value_start..].find('"') else {
        bail!("could not parse SPEC_COMMIT in {}", path.display());
    };
    Ok(Some(content[value_start..value_start + end].to_string()))
}

fn print_summary(action: &str, git_ref: Option<&str>, commit: &str, file_count: usize) {
    match git_ref {
        Some(reference) => println!(
            "spec/ {action} from {SPEC_REPO} @ {commit} (ref: {reference}, {file_count} files)"
        ),
        None => println!("spec/ {action} from {SPEC_REPO} @ {commit} ({file_count} files)"),
    }
}

fn workspace_root() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("xtask manifest has no parent directory")?
        .to_path_buf())
}

fn shallow_clone(git_ref: Option<&str>) -> Result<(tempfile::TempDir, String)> {
    let tmp = tempfile::tempdir().context("create temp dir for spec clone")?;
    let path = tmp.path();
    let path_str = path.to_str().context("temp path is not valid UTF-8")?;

    let success = match git_ref {
        Some(reference) => clone_at_ref(path, path_str, reference)?,
        None => clone_default_branch(path_str)?,
    };
    if !success {
        bail!("git clone/fetch failed for {SPEC_REPO}");
    }

    let commit = rev_parse_head(path)?;
    Ok((tmp, commit))
}

fn clone_default_branch(path_str: &str) -> Result<bool> {
    Ok(Command::new("git")
        .args(["clone", "--depth", "1", SPEC_REPO, path_str])
        .status()
        .context("spawn git clone")?
        .success())
}

fn clone_at_ref(path: &Path, path_str: &str, reference: &str) -> Result<bool> {
    if Command::new("git")
        .args([
            "clone", "--depth", "1", "--branch", reference, SPEC_REPO, path_str,
        ])
        .status()
        .context("spawn git clone --branch")?
        .success()
    {
        return Ok(true);
    }

    run_git(path, &["init"], "git init")?;
    run_git(
        path,
        &["remote", "add", "origin", SPEC_REPO],
        "git remote add",
    )?;
    run_git(
        path,
        &["fetch", "--depth", "1", "origin", reference],
        "git fetch",
    )?;
    run_git(path, &["checkout", "FETCH_HEAD"], "git checkout")?;
    Ok(true)
}

fn run_git(path: &Path, args: &[&str], label: &str) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(path)
        .status()
        .with_context(|| format!("spawn {label}"))?;
    if !status.success() {
        bail!("{label} failed with status {status}");
    }
    Ok(())
}

fn rev_parse_head(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .context("spawn git rev-parse")?;
    if !output.status.success() {
        bail!(
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn discover_upstream_files(clone_dir: &Path) -> Result<Vec<SpecIndexEntry>> {
    let mut entries = vec![SpecIndexEntry {
        vendored: SCHEMA_VENDORED.to_string(),
        upstream: SCHEMA_UPSTREAM.to_string(),
    }];

    let examples_dir = clone_dir.join("examples");
    if examples_dir.is_dir() {
        let mut example_files: Vec<_> = fs::read_dir(&examples_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect();
        example_files.sort();

        for path in example_files {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .context("invalid example filename")?;
            entries.push(SpecIndexEntry {
                vendored: format!("examples/{file_name}"),
                upstream: format!("examples/{file_name}"),
            });
        }
    }

    Ok(entries)
}

fn sync_vendored_files(
    clone_dir: &Path,
    spec_dir: &Path,
    entries: &[SpecIndexEntry],
) -> Result<()> {
    fs::create_dir_all(spec_dir.join("examples"))?;
    for entry in entries {
        let source = clone_dir.join(&entry.upstream);
        let dest = spec_dir.join(&entry.vendored);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        if !source.exists() {
            bail!("upstream file missing in clone: {}", entry.upstream);
        }
        fs::copy(&source, &dest)
            .with_context(|| format!("copy {} -> {}", source.display(), dest.display()))?;
    }
    Ok(())
}

fn prune_stale_vendored(spec_dir: &Path, entries: &[SpecIndexEntry]) -> Result<()> {
    let keep: std::collections::HashSet<_> = entries
        .iter()
        .map(|e| e.vendored.as_str())
        .chain(std::iter::once("index.yaml"))
        .collect();

    prune_dir(spec_dir, spec_dir, &keep)
}

fn prune_dir(root: &Path, current: &Path, keep: &std::collections::HashSet<&str>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            prune_dir(root, &path, keep)?;
            if fs::read_dir(&path)?.next().is_none() {
                fs::remove_dir(&path).ok();
            }
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if !keep.contains(rel.as_str()) {
            fs::remove_file(&path)
                .with_context(|| format!("remove stale vendored file {}", path.display()))?;
        }
    }
    Ok(())
}

fn read_index(path: &Path) -> Result<SpecIndex> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_saphyr::from_str(&content).context("parse spec/index.yaml")
}

fn write_index(spec_dir: &Path, commit: &str, entries: &[SpecIndexEntry]) -> Result<()> {
    let index = SpecIndex {
        repository: SPEC_REPO.to_string(),
        commit: commit.to_string(),
        files: entries.to_vec(),
    };
    let yaml = serde_saphyr::to_string(&index).context("serialize spec/index.yaml")?;
    fs::write(spec_dir.join("index.yaml"), yaml).context("write spec/index.yaml")?;
    Ok(())
}

fn update_pinned_commit(root: &Path, commit: &str) -> Result<()> {
    update_spec_rs(&root.join("src/spec.rs"), commit)
}

fn update_spec_rs(path: &Path, commit: &str) -> Result<()> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let updated = replace_const_string(&content, "SPEC_COMMIT", commit)?;
    fs::write(path, updated).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn replace_const_string(content: &str, name: &str, value: &str) -> Result<String> {
    let needle = format!("pub const {name}: &str = \"");
    let Some(start) = content.find(&needle) else {
        bail!("could not find constant {name} in src/spec.rs");
    };
    let value_start = start + needle.len();
    let Some(end) = content[value_start..].find('"') else {
        bail!("could not find end of constant {name} in src/spec.rs");
    };
    let mut updated = String::with_capacity(content.len());
    updated.push_str(&content[..value_start]);
    updated.push_str(value);
    updated.push_str(&content[value_start + end..]);
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_spec_commit_constant() {
        let input = r#"pub const SPEC_COMMIT: &str = "oldhash";
"#;
        let out = replace_const_string(input, "SPEC_COMMIT", "newhash").unwrap();
        assert!(out.contains(r#"pub const SPEC_COMMIT: &str = "newhash";"#));
    }

    #[test]
    fn manifest_diff_reports_missing_and_stale_files() {
        let local = vec![SpecIndexEntry {
            vendored: "examples/old.md".to_string(),
            upstream: "examples/old.md".to_string(),
        }];
        let upstream = vec![SpecIndexEntry {
            vendored: "examples/new.md".to_string(),
            upstream: "examples/new.md".to_string(),
        }];
        let diff = manifest_diff(&local, &upstream);
        assert!(
            diff.iter()
                .any(|line| line.contains("missing locally: examples/new.md"))
        );
        assert!(
            diff.iter()
                .any(|line| line.contains("stale locally: examples/old.md"))
        );
    }
}
