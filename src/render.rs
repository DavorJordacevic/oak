use std::io::{self};
use std::time::SystemTime;

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

    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == child_count - 1;
        let should_show = match (opts.dirs_only, opts.files_only) {
            (true, _) => child.is_dir,
            (_, true) => !child.is_dir,
            _ => true,
        };

        if should_show {
            render_child(child, "", is_last, opts, &mut dir_count, &mut file_count, &mut total_size)?;
        }
    }

    Ok((dir_count, file_count, total_size))
}

fn print_root_name(node: &TreeNode, opts: &RenderOpts) {
    let icon_str = if opts.show_icons {
        format!("{} ", icon::get_icon(&node.name, node.is_dir, node.is_symlink))
    } else {
        String::new()
    };
    let name = format!("{}/", node.name);
    let line = format!("{}{}", icon_str, name);

    if opts.show_colors {
        print!(
            "{}",
            line.if_supports_color(Stdout, |t| t.bright_blue())
        );
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
        format!("{} ", icon::get_icon(&node.name, node.is_dir, node.is_symlink))
    } else {
        String::new()
    };

    let displayed_name = format!("{}{}", icon_str, name);

    let mut meta_parts: Vec<String> = Vec::new();
    if opts.show_sizes && !node.is_dir {
        meta_parts.push(human_size(node.size));
    }
    if opts.show_times && !node.is_dir {
        meta_parts.push(relative_time(node.modified));
    }
    let meta = if meta_parts.is_empty() {
        String::new()
    } else {
        format!("  {}", meta_parts.join("  "))
    };

    let line = format!("{}{}{}", prefix, connector, displayed_name);

    if opts.show_colors {
        print_colored(&line, node);
    } else {
        print!("{}", line);
    }

    if !meta.is_empty() {
        if opts.show_colors {
            print!(
                "{}",
                meta.if_supports_color(Stdout, |t| t.dimmed())
            );
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

        let child_count = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            let child_is_last = i == child_count - 1;
            let should_show = match (opts.dirs_only, opts.files_only) {
                (true, _) => child.is_dir,
                (_, true) => !child.is_dir,
                _ => true,
            };

            if should_show {
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
        }
    } else {
        *file_count += 1;
        *total_size += node.size;
    }

    Ok(())
}

fn print_colored(line: &str, node: &TreeNode) {
    if node.is_dir {
        print!(
            "{}",
            String::from(line).if_supports_color(Stdout, |t| t.bright_blue())
        );
    } else if node.is_symlink {
        print!(
            "{}",
            String::from(line).if_supports_color(Stdout, |t| t.cyan())
        );
    } else {
        let ext = node.name.rsplit('.').next().unwrap_or("");
        match ext {
            "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" => {
                print!(
                    "{}",
                    String::from(line).if_supports_color(Stdout, |t| t.green())
                );
            }
            "rs" | "go" | "py" | "js" | "ts" | "jsx" | "tsx" | "rb" | "java" | "c" | "cpp"
            | "cs" | "swift" | "kt" | "scala" | "dart" | "lua" | "hs" | "elm" | "ex" | "erl"
            | "clj" | "r" | "sql" | "php" | "nim" | "ml" => {
                print!(
                    "{}",
                    String::from(line).if_supports_color(Stdout, |t| t.yellow())
                );
            }
            "md" | "markdown" | "txt" | "rst" => {
                print!(
                    "{}",
                    String::from(line).if_supports_color(Stdout, |t| t.white())
                );
            }
            "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" | "conf" | "lock" | "env" => {
                print!(
                    "{}",
                    String::from(line).if_supports_color(Stdout, |t| t.magenta())
                );
            }
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => {
                print!(
                    "{}",
                    String::from(line).if_supports_color(Stdout, |t| t.red())
                );
            }
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "webp" | "bmp" | "mp4" | "mkv"
            | "webm" | "avi" | "mov" | "mp3" | "wav" | "flac" | "ogg" => {
                print!(
                    "{}",
                    String::from(line).if_supports_color(Stdout, |t| t.magenta())
                );
            }
            _ => {
                print!("{}", String::from(line).if_supports_color(Stdout, |t| t));
            }
        }
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
