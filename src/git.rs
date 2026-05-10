use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    }

    for child in &mut node.children {
        annotate_node(root, child, statuses);
    }
}
