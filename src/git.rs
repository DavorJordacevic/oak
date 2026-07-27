use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use rayon::prelude::*;

use crate::tree::TreeNode;

pub fn annotate(root: &Path, tree: &mut TreeNode) {
    let statuses = status_map(root);
    if statuses.is_empty() {
        return;
    }
    annotate_node(root, tree, &statuses);
}

fn status_map(root: &Path) -> HashMap<PathBuf, String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
        .output();
    let Ok(output) = output else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }

    let mut statuses = HashMap::new();
    for entry in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        if entry.len() < 4 {
            continue;
        }
        let status = String::from_utf8_lossy(&entry[..2]).trim().to_string();
        let path = String::from_utf8_lossy(&entry[3..]).to_string();
        if !path.is_empty() && !status.is_empty() {
            statuses.insert(PathBuf::from(path), status);
        }
    }
    statuses
}

fn annotate_node(root: &Path, node: &mut TreeNode, statuses: &HashMap<PathBuf, String>) {
    if let Ok(relative) = node.path.strip_prefix(root)
        && let Some(status) = statuses.get(relative)
    {
        node.git_status = Some(status.clone());
    } else if node.is_dir
        && let Ok(relative) = node.path.strip_prefix(root)
        && !relative.as_os_str().is_empty()
        && let Some(status) = statuses
            .iter()
            .find_map(|(path, status)| path.starts_with(relative).then_some(status))
    {
        node.git_status = Some(status.clone());
    }

    for child in &mut node.children {
        annotate_node(root, child, statuses);
    }
}

pub fn blame(root: &Path, tree: &mut TreeNode) {
    let map = blame_map(root);
    if map.is_empty() {
        return;
    }
    blame_node(root, tree, &map);
}

fn blame_map(root: &Path) -> HashMap<PathBuf, String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["ls-files", "-z"])
        .output();
    let Ok(output) = output else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }

    let files: Vec<PathBuf> = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| PathBuf::from(String::from_utf8_lossy(entry).as_ref()))
        .collect();

    files
        .par_iter()
        .filter_map(|file| {
            let output = Command::new("git")
                .args(["-C"])
                .arg(root)
                .args(["log", "-1", "--format=%an"])
                .arg("--")
                .arg(file)
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let author = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if author.is_empty() {
                return None;
            }
            Some((file.clone(), author))
        })
        .collect()
}

fn blame_node(root: &Path, node: &mut TreeNode, map: &HashMap<PathBuf, String>) {
    if !node.is_dir
        && let Ok(relative) = node.path.strip_prefix(root)
        && let Some(author) = map.get(relative)
    {
        node.git_blame = Some(author.clone());
    }

    for child in &mut node.children {
        blame_node(root, child, map);
    }
}
