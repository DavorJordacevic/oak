use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub link_target: Option<PathBuf>,
    pub link_broken: bool,
    pub git_status: Option<String>,
    pub permissions: String,
    pub size: u64,
    pub modified: SystemTime,
    pub children: Vec<TreeNode>,
}

pub struct RawEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub link_target: Option<PathBuf>,
    pub link_broken: bool,
    pub permissions: String,
    pub size: u64,
    pub modified: SystemTime,
}

impl TreeNode {
    pub fn build(root: &Path, entries_by_parent: &HashMap<PathBuf, Vec<RawEntry>>) -> Option<Self> {
        let root_path = root.to_path_buf();
        let root_name = root
            .file_name()
            .unwrap_or(root.as_os_str())
            .to_string_lossy()
            .to_string();

        let children: Vec<TreeNode> =
            entries_by_parent
                .get(&root_path)
                .map_or(Vec::new(), |entries| {
                    entries
                        .iter()
                        .filter_map(|entry| {
                            if entry.is_dir {
                                Self::build(&entry.path, entries_by_parent).map(|mut node| {
                                    node.is_symlink = entry.is_symlink;
                                    node.link_target = entry.link_target.clone();
                                    node.link_broken = entry.link_broken;
                                    node.permissions = entry.permissions.clone();
                                    node.modified = entry.modified;
                                    node
                                })
                            } else {
                                Some(TreeNode {
                                    name: entry.name.clone(),
                                    path: entry.path.clone(),
                                    is_dir: false,
                                    is_symlink: entry.is_symlink,
                                    link_target: entry.link_target.clone(),
                                    link_broken: entry.link_broken,
                                    git_status: None,
                                    permissions: entry.permissions.clone(),
                                    size: entry.size,
                                    modified: entry.modified,
                                    children: Vec::new(),
                                })
                            }
                        })
                        .collect()
                });
        let size = children.iter().map(|child| child.size).sum();

        Some(TreeNode {
            name: root_name,
            path: root_path,
            is_dir: true,
            is_symlink: false,
            link_target: None,
            link_broken: false,
            git_status: None,
            permissions: String::new(),
            size,
            modified: SystemTime::UNIX_EPOCH,
            children,
        })
    }
}
