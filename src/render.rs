use std::io::{self};
use std::time::{Duration, SystemTime};

use owo_colors::{OwoColorize, Stream::Stdout};

use crate::icon;
use crate::tree::TreeNode;

pub struct RenderOpts {
    pub show_sizes: bool,
    pub show_times: bool,
    pub show_icons: bool,
    pub show_colors: bool,
    pub dirs_only: bool,
    pub files_only: bool,
    pub show_links: bool,
    pub show_stats: bool,
    pub show_du: bool,
    pub show_git: bool,
    pub show_perms: bool,
}

pub fn render(node: &TreeNode, opts: &RenderOpts) -> io::Result<(usize, usize, u64)> {
    let mut dir_count = 0;
    let mut file_count = 0;
    let mut total_size = 0u64;

    if node.is_dir {
        dir_count += 1;
    } else {
        file_count += 1;
        total_size += node.size;
    }

    print_root_name(node, opts);

    let children = visible_children(node, opts);
    let child_count = children.len();
    for (i, child) in children.into_iter().enumerate() {
        let is_last = i == child_count - 1;
        render_child(
            child,
            "",
            is_last,
            opts,
            &mut dir_count,
            &mut file_count,
            &mut total_size,
        )?;
    }

    Ok((dir_count, file_count, total_size))
}

fn print_root_name(node: &TreeNode, opts: &RenderOpts) {
    let icon_str = if opts.show_icons {
        format!(
            "{} ",
            icon::get_icon(&node.name, node.is_dir, node.is_symlink)
        )
    } else {
        String::new()
    };
    let name = format!("{}/", node.name);
    let line = format!("{}{}", icon_str, name);

    if opts.show_colors {
        print!("{}", line.if_supports_color(Stdout, |t| t.bright_blue()));
    } else {
        print!("{}", line);
    }
    println!();
}

fn render_child(
    node: &TreeNode,
    prefix: &str,
    is_last: bool,
    opts: &RenderOpts,
    dir_count: &mut usize,
    file_count: &mut usize,
    total_size: &mut u64,
) -> io::Result<()> {
    let connector = if is_last {
        "\u{2514}\u{2500}\u{2500} "
    } else {
        "\u{251c}\u{2500}\u{2500} "
    };

    let name = if node.is_dir {
        format!("{}/", node.name)
    } else {
        node.name.clone()
    };

    let icon_str = if opts.show_icons {
        format!(
            "{} ",
            icon::get_icon(&node.name, node.is_dir, node.is_symlink)
        )
    } else {
        String::new()
    };

    let displayed_name = format!("{}{}", icon_str, name);
    let leading = if opts.show_perms && !node.permissions.is_empty() {
        format!("{}  ", node.permissions)
    } else {
        String::new()
    };

    let mut meta_parts: Vec<String> = Vec::new();
    if opts.show_sizes && !node.is_dir {
        meta_parts.push(human_size(node.size));
    }
    if opts.show_du && node.is_dir {
        meta_parts.push(human_size(node.size));
    }
    if opts.show_times && !node.is_dir {
        meta_parts.push(relative_time(node.modified));
    }
    if opts.show_links
        && node.is_symlink
        && let Some(target) = &node.link_target
    {
        let mut link = format!("-> {}", target.display());
        if node.link_broken {
            link.push_str(" [broken]");
        }
        meta_parts.push(link);
    }
    if opts.show_git
        && let Some(status) = &node.git_status
    {
        meta_parts.push(status.clone());
    }
    let meta = if meta_parts.is_empty() {
        String::new()
    } else {
        format!("  {}", meta_parts.join("  "))
    };

    let line = format!("{}{}{}{}", prefix, connector, leading, displayed_name);

    if opts.show_colors {
        print_colored(&line, node);
    } else {
        print!("{}", line);
    }

    if !meta.is_empty() {
        if opts.show_colors {
            print!("{}", meta.if_supports_color(Stdout, |t| t.dimmed()));
        } else {
            print!("{}", meta);
        }
    }
    println!();

    if node.is_dir {
        *dir_count += 1;
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}\u{2502}   ", prefix)
        };

        let children = visible_children(node, opts);
        let child_count = children.len();
        for (i, child) in children.into_iter().enumerate() {
            let child_is_last = i == child_count - 1;
            render_child(
                child,
                &child_prefix,
                child_is_last,
                opts,
                dir_count,
                file_count,
                total_size,
            )?;
        }
    } else {
        *file_count += 1;
        *total_size += node.size;
    }

    Ok(())
}

fn visible_children<'a>(node: &'a TreeNode, opts: &RenderOpts) -> Vec<&'a TreeNode> {
    let mut children = Vec::new();
    collect_visible_children(node, opts, &mut children);
    children
}

fn collect_visible_children<'a>(
    node: &'a TreeNode,
    opts: &RenderOpts,
    visible: &mut Vec<&'a TreeNode>,
) {
    for child in &node.children {
        match (opts.dirs_only, opts.files_only, child.is_dir) {
            (true, _, true) => visible.push(child),
            (_, true, true) => collect_visible_children(child, opts, visible),
            (_, true, false) => visible.push(child),
            (false, false, _) => visible.push(child),
            _ => {}
        }
    }
}

fn print_colored(line: &str, node: &TreeNode) {
    if node.is_dir {
        print!("{}", color_dir(line));
    } else if node.is_symlink {
        print!("{}", color_symlink(line));
    } else {
        let ext = node.name.rsplit('.').next().unwrap_or("");
        print!("{}", color_file(line, ext));
    }
}

fn color_dir(text: &str) -> String {
    format!(
        "{}",
        String::from(text).if_supports_color(Stdout, |t| t.bright_blue())
    )
}

fn color_symlink(text: &str) -> String {
    format!(
        "{}",
        String::from(text).if_supports_color(Stdout, |t| t.cyan())
    )
}

fn color_meta(text: &str) -> String {
    format!(
        "{}",
        String::from(text).if_supports_color(Stdout, |t| t.dimmed())
    )
}

fn color_file(text: &str, ext: &str) -> String {
    match ext {
        "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" => {
            format!(
                "{}",
                String::from(text).if_supports_color(Stdout, |t| t.green())
            )
        }
        "rs" | "go" | "py" | "js" | "ts" | "jsx" | "tsx" | "rb" | "java" | "c" | "cpp" | "cs"
        | "swift" | "kt" | "scala" | "dart" | "lua" | "hs" | "elm" | "ex" | "erl" | "clj" | "r"
        | "sql" | "php" | "nim" | "ml" => {
            format!(
                "{}",
                String::from(text).if_supports_color(Stdout, |t| t.yellow())
            )
        }
        "md" | "markdown" | "txt" | "rst" => {
            format!(
                "{}",
                String::from(text).if_supports_color(Stdout, |t| t.white())
            )
        }
        "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" | "conf" | "lock" | "env" => {
            format!(
                "{}",
                String::from(text).if_supports_color(Stdout, |t| t.magenta())
            )
        }
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => {
            format!(
                "{}",
                String::from(text).if_supports_color(Stdout, |t| t.red())
            )
        }
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "webp" | "bmp" | "mp4" | "mkv"
        | "webm" | "avi" | "mov" | "mp3" | "wav" | "flac" | "ogg" => {
            format!(
                "{}",
                String::from(text).if_supports_color(Stdout, |t| t.magenta())
            )
        }
        _ => format!("{}", String::from(text).if_supports_color(Stdout, |t| t)),
    }
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

pub fn print_stats(node: &TreeNode, opts: &RenderOpts) {
    if !opts.show_stats {
        return;
    }

    let mut files = Vec::new();
    collect_files(node, &mut files);
    if files.is_empty() {
        return;
    }

    let mut by_type: Vec<(String, usize, u64)> = type_stats(&files).into_values().collect();
    by_type.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let mut largest = files;
    largest.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));

    println!();
    println!("by type:");
    for (label, count, size) in by_type.into_iter().take(5) {
        let file_label = if count == 1 { "file" } else { "files" };
        println!(
            "  {}  {} {}  {}",
            label,
            count,
            file_label,
            human_size(size)
        );
    }

    println!();
    println!("largest:");
    for file in largest.into_iter().take(5) {
        println!("  {}  {}", file.path.display(), human_size(file.size));
    }
}

pub fn render_timeline(node: &TreeNode, opts: &RenderOpts) -> io::Result<()> {
    let mut entries = Vec::new();
    collect_timeline_entries(node, node, opts, &mut entries);
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut current_group = "";
    for (path, modified, node) in entries {
        let group = timeline_group(modified);
        if group != current_group {
            if !current_group.is_empty() {
                println!();
            }
            if opts.show_colors {
                println!("{}", color_dir(&format!("{group}:")));
            } else {
                println!("{group}:");
            }
            current_group = group;
        }
        let leading = if opts.show_perms && !node.permissions.is_empty() {
            format!("{}  ", node.permissions)
        } else {
            String::new()
        };
        let mut meta_parts = Vec::new();
        if (opts.show_du && node.is_dir) || (opts.show_sizes && !node.is_dir) {
            meta_parts.push(human_size(node.size));
        }
        if opts.show_git
            && let Some(status) = &node.git_status
        {
            meta_parts.push(status.clone());
        }
        let meta = if meta_parts.is_empty() {
            String::new()
        } else {
            format!("  {}", meta_parts.join("  "))
        };
        let entry = format!("  {leading}{path}");
        if opts.show_colors {
            if node.is_dir {
                print!("{}", color_dir(&entry));
            } else if node.is_symlink {
                print!("{}", color_symlink(&entry));
            } else {
                let ext = node.name.rsplit('.').next().unwrap_or("");
                print!("{}", color_file(&entry, ext));
            }
            if !meta.is_empty() {
                print!("{}", color_meta(&meta));
            }
            println!();
        } else {
            println!("{entry}{meta}");
        }
    }
    Ok(())
}

fn collect_timeline_entries<'a>(
    root: &TreeNode,
    node: &'a TreeNode,
    opts: &RenderOpts,
    entries: &mut Vec<(String, SystemTime, &'a TreeNode)>,
) {
    for child in visible_children(node, opts) {
        if let Ok(relative) = child.path.strip_prefix(&root.path)
            && !relative.as_os_str().is_empty()
        {
            let mut path = relative.to_string_lossy().to_string();
            if child.is_dir {
                path.push('/');
            }
            entries.push((path, child.modified, child));
        }
        if child.is_dir {
            collect_timeline_entries(root, child, opts, entries);
        }
    }
}

fn timeline_group(modified: SystemTime) -> &'static str {
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::ZERO);
    let days = age.as_secs() / 86_400;
    if days == 0 {
        "Today"
    } else if days <= 7 {
        "Last week"
    } else if days <= 31 {
        "Last month"
    } else if days <= 92 {
        "3 months ago"
    } else if days <= 366 {
        "This year"
    } else {
        "Older"
    }
}

fn collect_files<'a>(node: &'a TreeNode, files: &mut Vec<&'a TreeNode>) {
    if !node.is_dir {
        files.push(node);
    }
    for child in &node.children {
        collect_files(child, files);
    }
}

fn type_stats(files: &[&TreeNode]) -> std::collections::HashMap<String, (String, usize, u64)> {
    let mut stats = std::collections::HashMap::new();
    for file in files {
        let label = type_label(&file.name).to_string();
        let entry = stats.entry(label.clone()).or_insert((label, 0, 0));
        entry.1 += 1;
        entry.2 += file.size;
    }
    stats
}

fn type_label(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("") {
        "rs" => "Rust",
        "md" | "markdown" | "mdx" => "Markdown",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" => "Images",
        "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" | "conf" => "Config",
        "txt" | "rst" => "Text",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "tgz" => "Archives",
        "mp4" | "mkv" | "webm" | "avi" | "mov" => "Video",
        "mp3" | "wav" | "flac" | "ogg" | "aac" => "Audio",
        "" => "Other",
        _ => "Other",
    }
}

fn relative_time(modified: SystemTime) -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(modified).unwrap_or_default();
    let secs = duration.as_secs();

    if secs < 5 {
        "just now".to_string()
    } else if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 604800 {
        format!("{}d ago", secs / 86400)
    } else if secs < 2592000 {
        format!("{}w ago", secs / 604800)
    } else if secs < 31536000 {
        format!("{}mo ago", secs / 2592000)
    } else {
        format!("{}y ago", secs / 31536000)
    }
}
