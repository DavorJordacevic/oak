mod icon;
mod tree;
mod walk;
mod sort;
mod render;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::render::RenderOpts;
use crate::sort::sort_nodes;
use crate::tree::TreeNode;
use crate::walk::walk;

#[derive(Parser)]
#[command(name = "oak")]
#[command(
    version,
    about = "A modern, fast, gitignore-aware directory listing",
    long_about = None
)]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'L', long, help = "Maximum display depth")]
    level: Option<usize>,

    #[arg(short = 'a', long, help = "Show hidden files")]
    all: bool,

    #[arg(short = 's', long, help = "Show file sizes")]
    sizes: bool,

    #[arg(short = 't', long, help = "Show modification times")]
    times: bool,

    #[arg(short = 'P', long, help = "Only show files matching pattern (regex)")]
    pattern: Option<String>,

    #[arg(short = 'I', long, help = "Exclude files matching pattern (regex)")]
    exclude: Option<String>,

    #[arg(long, help = "Do not respect .gitignore / .ignore files")]
    no_ignore: bool,

    #[arg(long, help = "Disable NerdFont icons")]
    no_icons: bool,

    #[arg(long, help = "Output without color")]
    no_color: bool,

    #[arg(long, help = "Show directories only")]
    dirs_only: bool,

    #[arg(long, help = "Show files only")]
    files_only: bool,

    #[arg(short = 'S', long, value_enum, default_value_t, help = "Sort order")]
    sort: sort::SortBy,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let root_path = if cli.path.is_relative() {
        std::env::current_dir()?.join(&cli.path)
    } else {
        cli.path.clone()
    };

    let root_path = root_path.canonicalize().unwrap_or(root_path);

    if !root_path.is_dir() {
        anyhow::bail!("Not a directory: {}", root_path.display());
    }

    let entries_by_parent = walk(
        &root_path,
        cli.all,
        cli.level,
        cli.no_ignore,
    )?;

    let mut tree = TreeNode::build(&root_path, &entries_by_parent)
        .ok_or_else(|| anyhow::anyhow!("Failed to build tree"))?;

    if let Ok(meta) = std::fs::metadata(&root_path) {
        tree.modified = meta.modified().unwrap_or(tree.modified);
    }

    apply_pattern_filter(&mut tree, cli.pattern.as_deref(), cli.exclude.as_deref());

    sort_nodes(&mut tree, cli.sort);

    let opts = RenderOpts {
        show_sizes: cli.sizes,
        show_times: cli.times,
        show_icons: !cli.no_icons,
        show_colors: !cli.no_color,
        dirs_only: cli.dirs_only,
        files_only: cli.files_only,
    };

    let (dirs, files, total_size) = render::render(&tree, &opts)?;

    let size_str = render::human_size(total_size);

    if !cli.dirs_only && !cli.files_only {
        let dir_label = if dirs == 1 { "directory" } else { "directories" };
        let file_label = if files == 1 { "file" } else { "files" };
        println!();
        println!("{} {}, {} {}, {}", dirs, dir_label, files, file_label, size_str);
    } else if cli.dirs_only {
        let dir_label = if dirs == 1 { "directory" } else { "directories" };
        println!();
        println!("{} {}", dirs, dir_label);
    } else if cli.files_only {
        let file_label = if files == 1 { "file" } else { "files" };
        println!();
        println!("{} {}, {}", files, file_label, size_str);
    }

    Ok(())
}

fn apply_pattern_filter(
    node: &mut TreeNode,
    include: Option<&str>,
    exclude: Option<&str>,
) {
    let include_re = include.map(|p| regex::Regex::new(p).unwrap());
    let exclude_re = exclude.map(|p| regex::Regex::new(p).unwrap());

    filter_children(node, &include_re, &exclude_re);
}

fn filter_children(
    node: &mut TreeNode,
    include: &Option<regex::Regex>,
    exclude: &Option<regex::Regex>,
) {
    node.children.retain(|child| {
        if child.is_dir {
            return true;
        }
        let keep = include
            .as_ref()
            .map_or(true, |re| re.is_match(&child.name));
        let skip = exclude
            .as_ref()
            .map_or(false, |re| re.is_match(&child.name));
        keep && !skip
    });

    for child in &mut node.children {
        if child.is_dir {
            filter_children(child, include, exclude);
        }
    }
}
