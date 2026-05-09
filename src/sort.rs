use std::cmp::Ordering;

use crate::tree::TreeNode;

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum SortBy {
    #[default]
    /// Sort by last modified time (most recent first)
    Mtime,
    /// Sort alphabetically by name
    Name,
    /// Sort by file size (largest first)
    Size,
    /// Sort by file extension
    Ext,
}

pub fn sort_nodes(node: &mut TreeNode, sort_by: SortBy) {
    node.children.sort_by(|a, b| {
        // Always group directories before files
        match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => match sort_by {
                SortBy::Mtime => {
                    match b.modified.cmp(&a.modified) {
                        Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                        other => other,
                    }
                }
                SortBy::Name => {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
                SortBy::Size => {
                    match b.size.cmp(&a.size) {
                        Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                        other => other,
                    }
                }
                SortBy::Ext => {
                    let a_ext = a.name.rsplit('.').next().unwrap_or("").to_lowercase();
                    let b_ext = b.name.rsplit('.').next().unwrap_or("").to_lowercase();
                    match a_ext.cmp(&b_ext) {
                        Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                        other => other,
                    }
                }
            }
        }
    });

    for child in &mut node.children {
        if child.is_dir {
            sort_nodes(child, sort_by);
        }
    }
}
