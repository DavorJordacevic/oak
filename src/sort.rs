use std::cmp::Ordering;

use oak::cli::SortBy;

use crate::tree::TreeNode;

// Keeping tests next to the small fixture helpers makes the ordering behavior
// easier to read; production code follows below.
#[cfg_attr(test, allow(clippy::items_after_test_module))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    fn leaf(name: &str) -> TreeNode {
        TreeNode {
            name: name.to_string(),
            path: PathBuf::from(name),
            is_dir: false,
            is_symlink: false,
            link_target: None,
            link_broken: false,
            git_status: None,
            git_blame: None,
            permissions: String::new(),
            size: if name.len() > 5 { 100 } else { 10 },
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(name.len() as u64 * 100),
            children: Vec::new(),
        }
    }

    fn dir(name: &str, children: Vec<TreeNode>) -> TreeNode {
        TreeNode {
            name: name.to_string(),
            path: PathBuf::from(name),
            is_dir: true,
            is_symlink: false,
            link_target: None,
            link_broken: false,
            git_status: None,
            git_blame: None,
            permissions: String::new(),
            size: children.iter().map(|c| c.size).sum(),
            modified: SystemTime::UNIX_EPOCH,
            children,
        }
    }

    #[test]
    fn sort_by_name_alphabetical() {
        let mut root = dir(
            "root",
            vec![leaf("delta"), leaf("beta"), leaf("alpha"), leaf("gamma")],
        );
        sort_nodes(&mut root, SortBy::Name);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta", "delta", "gamma"]);
    }

    #[test]
    fn sort_dirs_before_files_always() {
        let mut root = dir(
            "root",
            vec![
                leaf("z-file"),
                dir("a-dir", Vec::new()),
                leaf("b-file"),
                dir("m-dir", Vec::new()),
            ],
        );
        sort_nodes(&mut root, SortBy::Name);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["a-dir", "m-dir", "b-file", "z-file"]);
    }

    #[test]
    fn sort_dirs_before_files_with_size_sort() {
        let mut root = dir(
            "root",
            vec![
                leaf("big-file"), // size 100 (len > 5)
                dir("tiny-dir", Vec::new()),
                leaf("a"), // size 10
            ],
        );
        sort_nodes(&mut root, SortBy::Size);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["tiny-dir", "big-file", "a"]);
    }

    #[test]
    fn sort_by_size_largest_first() {
        let mut root = dir(
            "root",
            vec![
                leaf("a"),         // size 10
                leaf("longname"),  // size 100
                leaf("ab"),        // size 10
                leaf("extralong"), // size 100
            ],
        );
        sort_nodes(&mut root, SortBy::Size);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["extralong", "longname", "a", "ab"]);
    }

    #[test]
    fn sort_by_size_falls_back_to_name() {
        let mut root = dir("root", vec![leaf("longname"), leaf("extralong")]);
        root.children[0].size = 50;
        root.children[1].size = 50;
        sort_nodes(&mut root, SortBy::Size);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["extralong", "longname"]);
    }

    #[test]
    fn sort_by_mtime_most_recent_first() {
        let mut root = dir("root", vec![leaf("old"), leaf("new"), leaf("mid")]);
        // Set specific mtimes: newer = higher seconds
        root.children[0].modified = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        root.children[1].modified = SystemTime::UNIX_EPOCH + Duration::from_secs(300);
        root.children[2].modified = SystemTime::UNIX_EPOCH + Duration::from_secs(200);
        sort_nodes(&mut root, SortBy::Mtime);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["new", "mid", "old"]);
    }

    #[test]
    fn sort_by_ext() {
        let mut root = dir(
            "root",
            vec![
                leaf("main.rs"),
                leaf("lib.py"),
                leaf("app.rs"),
                leaf("util.go"),
            ],
        );
        sort_nodes(&mut root, SortBy::Ext);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        // ext "go" < "py" < "rs", and within rs alphabetical
        assert_eq!(names, vec!["util.go", "lib.py", "app.rs", "main.rs"]);
    }

    #[test]
    fn sort_by_ext_no_extension() {
        let mut root = dir(
            "root",
            vec![leaf("Makefile"), leaf("dockerfile"), leaf("LICENSE")],
        );
        sort_nodes(&mut root, SortBy::Ext);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["dockerfile", "LICENSE", "Makefile"]);
    }

    #[test]
    fn sort_nested_dirs_recursively() {
        let mut root = dir("root", vec![dir("sub", vec![leaf("z.txt"), leaf("a.txt")])]);
        sort_nodes(&mut root, SortBy::Name);
        let sub = &root.children[0];
        let sub_names: Vec<&str> = sub.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(sub_names, vec!["a.txt", "z.txt"]);
    }
}

pub fn sort_nodes(node: &mut TreeNode, sort_by: SortBy) {
    node.children.sort_by(|a, b| {
        // Always group directories before files
        match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => match sort_by {
                SortBy::Mtime => match b.modified.cmp(&a.modified) {
                    Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    other => other,
                },
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Size => match b.size.cmp(&a.size) {
                    Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    other => other,
                },
                SortBy::Ext => {
                    let a_ext = a.name.rsplit('.').next().unwrap_or("").to_lowercase();
                    let b_ext = b.name.rsplit('.').next().unwrap_or("").to_lowercase();
                    match a_ext.cmp(&b_ext) {
                        Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                        other => other,
                    }
                }
            },
        }
    });

    for child in &mut node.children {
        if child.is_dir {
            sort_nodes(child, sort_by);
        }
    }
}
