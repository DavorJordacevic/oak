use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, PermissionsExt};

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
        let permissions = format_permissions(&path, is_dir, is_symlink);

        let raw = RawEntry {
            name,
            path: path.clone(),
            is_dir,
            is_symlink,
            link_target,
            link_broken,
            permissions,
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

#[cfg(unix)]
fn format_permissions(path: &Path, is_dir: bool, is_symlink: bool) -> String {
    let file_type = if is_symlink {
        'l'
    } else if is_dir {
        'd'
    } else {
        '-'
    };
    let mode = std::fs::symlink_metadata(path)
        .map(|meta| meta.permissions().mode())
        .unwrap_or(0);
    let mut output = String::with_capacity(10);
    output.push(file_type);
    for bit in [
        0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001,
    ] {
        let ch = match bit {
            0o400 | 0o040 | 0o004 => 'r',
            0o200 | 0o020 | 0o002 => 'w',
            _ => 'x',
        };
        output.push(if mode & bit != 0 { ch } else { '-' });
    }

    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return output;
    };
    if meta.file_type().is_socket() {
        output.replace_range(0..1, "s");
    } else if meta.file_type().is_fifo() {
        output.replace_range(0..1, "p");
    }
    output
}

#[cfg(not(unix))]
fn format_permissions(_path: &Path, is_dir: bool, is_symlink: bool) -> String {
    let file_type = if is_symlink {
        'l'
    } else if is_dir {
        'd'
    } else {
        '-'
    };
    format!("{file_type}---------")
}
