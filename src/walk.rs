use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use ignore::WalkBuilder;

use crate::tree::RawEntry;

const ALWAYS_SKIP: &[&str] = &[".git", ".svn", ".hg", ".DS_Store"];

pub fn walk(
    root: &Path,
    show_hidden: bool,
    max_depth: Option<usize>,
    no_ignore: bool,
) -> Result<HashMap<PathBuf, Vec<RawEntry>>> {
    let mut builder = WalkBuilder::new(root);

    if no_ignore {
        builder.standard_filters(false);
        builder.hidden(!show_hidden);
    } else {
        builder.standard_filters(true);
        builder.hidden(!show_hidden);
    }

    builder.max_depth(max_depth);

    builder.filter_entry(move |entry| {
        let name = entry.file_name().to_string_lossy();
        if ALWAYS_SKIP.contains(&name.as_ref()) {
            return false;
        }
        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            return true;
        }
        if !show_hidden && name.starts_with('.') {
            return false;
        }
        true
    });

    let mut entries_by_parent: HashMap<PathBuf, Vec<RawEntry>> = HashMap::new();
    let root_buf = root.to_path_buf();
    entries_by_parent.entry(root_buf.clone()).or_default();

    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path().to_path_buf();
        if path == root_buf {
            continue;
        }

        let name = entry
            .path()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let is_dir = entry.file_type().is_some_and(|ft| ft.is_dir());
        let is_symlink = entry.file_type().is_some_and(|ft| ft.is_symlink());
        let link_target = if is_symlink {
            std::fs::read_link(&path).ok()
        } else {
            None
        };
        let link_broken = link_target.as_ref().is_some_and(|target| {
            let resolved = if target.is_absolute() {
                target.clone()
            } else {
                path.parent().unwrap_or(root).join(target)
            };
            !resolved.exists()
        });
        let (size, modified) = match entry.metadata() {
            Ok(m) => (m.len(), m.modified().unwrap_or(SystemTime::UNIX_EPOCH)),
            Err(_) => (0, SystemTime::UNIX_EPOCH),
        };

        let raw = RawEntry {
            name,
            path: path.clone(),
            is_dir,
            is_symlink,
            link_target,
            link_broken,
            size,
            modified,
        };

        if let Some(parent) = path.parent() {
            entries_by_parent
                .entry(parent.to_path_buf())
                .or_default()
                .push(raw);
        }
    }

    Ok(entries_by_parent)
}
