mod config;
mod icon;
mod render;
mod sort;
mod tree;
mod walk;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::config::{Config, merge_config};
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

    #[arg(long, help = "Save these options as future defaults and exit")]
    save_config: bool,

    #[arg(long, help = "Do not read saved config")]
    no_config: bool,

    #[arg(long, help = "Do not respect .gitignore / .ignore files")]
    no_ignore: bool,

    #[arg(long, help = "Disable icons")]
    no_icons: bool,

    #[arg(long, help = "Output without color")]
    no_color: bool,

    #[arg(long, help = "Show directories only")]
    dirs_only: bool,

    #[arg(long, help = "Show files only")]
    files_only: bool,

    #[arg(short = 'S', long, value_enum, help = "Sort order")]
    sort: Option<sort::SortBy>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = if cli.no_config {
        Config::default()
    } else {
        Config::load()?
    };
    let cli_config = Config {
        level: cli.level,
        all: cli.all.then_some(true),
        sizes: cli.sizes.then_some(true),
        times: cli.times.then_some(true),
        pattern: cli.pattern,
        exclude: cli.exclude,
        no_ignore: cli.no_ignore.then_some(true),
        no_icons: cli.no_icons.then_some(true),
        no_color: cli.no_color.then_some(true),
        dirs_only: cli.dirs_only.then_some(true),
        files_only: cli.files_only.then_some(true),
        sort: cli.sort,
    };
    let opts = merge_config(cli_config, config);

    if opts.dirs_only && opts.files_only {
        anyhow::bail!("--dirs-only and --files-only cannot be used together");
    }

    if cli.save_config {
        let path = opts.save()?;
        println!("Saved config to {}", path.display());
        return Ok(());
    }

    let root_path = if cli.path.is_relative() {
        std::env::current_dir()?.join(&cli.path)
    } else {
        cli.path.clone()
    };

    let root_path = root_path.canonicalize().unwrap_or(root_path);

    if !root_path.is_dir() {
        anyhow::bail!("Not a directory: {}", root_path.display());
    }

    let entries_by_parent = walk(&root_path, opts.all, opts.level, opts.no_ignore)?;

    let mut tree = TreeNode::build(&root_path, &entries_by_parent)
        .ok_or_else(|| anyhow::anyhow!("Failed to build tree"))?;

    if let Ok(meta) = std::fs::metadata(&root_path) {
        tree.modified = meta.modified().unwrap_or(tree.modified);
    }

    apply_pattern_filter(&mut tree, opts.pattern.as_deref(), opts.exclude.as_deref())?;

    sort_nodes(&mut tree, opts.sort);

    let render_opts = RenderOpts {
        show_sizes: opts.sizes,
        show_times: opts.times,
        show_icons: !opts.no_icons,
        show_colors: !opts.no_color,
        dirs_only: opts.dirs_only,
        files_only: opts.files_only,
    };

    let (dirs, files, total_size) = render::render(&tree, &render_opts)?;

    let size_str = render::human_size(total_size);

    if !opts.dirs_only && !opts.files_only {
        let dir_label = if dirs == 1 {
            "directory"
        } else {
            "directories"
        };
        let file_label = if files == 1 { "file" } else { "files" };
        println!();
        println!(
            "{} {}, {} {}, {}",
            dirs, dir_label, files, file_label, size_str
        );
    } else if opts.dirs_only {
        let dir_label = if dirs == 1 {
            "directory"
        } else {
            "directories"
        };
        println!();
        println!("{} {}", dirs, dir_label);
    } else if opts.files_only {
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
) -> Result<()> {
    let include_re = include
        .map(|p| regex::Regex::new(p).with_context(|| format!("Invalid include pattern: {p}")))
        .transpose()?;
    let exclude_re = exclude
        .map(|p| regex::Regex::new(p).with_context(|| format!("Invalid exclude pattern: {p}")))
        .transpose()?;

    filter_children(node, &include_re, &exclude_re);
    Ok(())
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
        let keep = include.as_ref().is_none_or(|re| re.is_match(&child.name));
        let skip = exclude.as_ref().is_some_and(|re| re.is_match(&child.name));
        keep && !skip
    });

    for child in &mut node.children {
        if child.is_dir {
            filter_children(child, include, exclude);
        }
    }
}
