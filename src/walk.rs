//! Recursive discovery of rule files on disk.

use crate::RuleFormat;
use crate::discover::ToolRuleDir;
use std::path::{Path, PathBuf};

/// A rule file found during a directory walk.
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// Path relative to the walk root.
    pub relative: PathBuf,
    /// Format associated with the tool directory.
    pub format: RuleFormat,
}

/// Walk a tool rule directory using extension/suffix rules from `dir`.
pub fn walk_tool_dir(
    root: &Path,
    dir: &ToolRuleDir,
) -> Result<Vec<DiscoveredFile>, std::io::Error> {
    let mut files = Vec::new();
    walk_recursive(root, root, dir, &mut files)?;
    files.sort_by(|a, b| a.relative.cmp(&b.relative));
    Ok(files)
}

fn walk_recursive(
    root: &Path,
    current: &Path,
    dir: &ToolRuleDir,
    files: &mut Vec<DiscoveredFile>,
) -> Result<(), std::io::Error> {
    if !current.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_recursive(root, &path, dir, files)?;
        } else if matches_rule_file(&path, dir) {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            files.push(DiscoveredFile {
                path: path.clone(),
                relative,
                format: dir.format,
            });
        }
    }
    Ok(())
}

fn matches_rule_file(path: &Path, dir: &ToolRuleDir) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    if let Some(suffix) = dir.suffix {
        return file_name.ends_with(suffix);
    }

    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| dir.extensions.contains(&ext))
}

/// Recursively collect all `*.md` files under `root`.
pub fn walk_md_files(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    walk_md_recursive(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_md_recursive(current: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_md_recursive(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}
