mod config;
mod git;
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

    #[arg(
        short = 'a',
        long,
        conflicts_with = "hide_hidden",
        help = "Show hidden files"
    )]
    all: bool,

    #[arg(long, conflicts_with = "all", help = "Hide hidden files")]
    hide_hidden: bool,

    #[arg(
        short = 's',
        long,
        conflicts_with = "no_sizes",
        help = "Show file sizes"
    )]
    sizes: bool,

    #[arg(long, conflicts_with = "sizes", help = "Hide file sizes")]
    no_sizes: bool,

    #[arg(
        short = 't',
        long,
        conflicts_with = "no_times",
        help = "Show modification times"
    )]
    times: bool,

    #[arg(long, conflicts_with = "times", help = "Hide modification times")]
    no_times: bool,

    #[arg(short = 'P', long, help = "Only show files matching pattern (regex)")]
    pattern: Option<String>,

    #[arg(short = 'I', long, help = "Exclude files matching pattern (regex)")]
    exclude: Option<String>,

    #[arg(long, help = "Save these options as future defaults and exit")]
    save_config: bool,

    #[arg(long, help = "Do not read saved config")]
    no_config: bool,

    #[arg(
        long,
        conflicts_with = "ignore",
        help = "Do not respect .gitignore / .ignore files"
    )]
    no_ignore: bool,

    #[arg(
        long,
        conflicts_with = "no_ignore",
        help = "Respect .gitignore / .ignore files"
    )]
    ignore: bool,

    #[arg(long, conflicts_with = "icons", help = "Disable icons")]
    no_icons: bool,

    #[arg(long, conflicts_with = "no_icons", help = "Enable icons")]
    icons: bool,

    #[arg(long, conflicts_with = "color", help = "Output without color")]
    no_color: bool,

    #[arg(long, conflicts_with = "no_color", help = "Enable color output")]
    color: bool,

    #[arg(long, conflicts_with = "files_only", help = "Show directories only")]
    dirs_only: bool,

    #[arg(long, conflicts_with = "dirs_only", help = "Show files only")]
    files_only: bool,

    #[arg(long, conflicts_with = "no_stats", help = "Show statistics")]
    stats: bool,

    #[arg(long, conflicts_with = "stats", help = "Hide statistics")]
    no_stats: bool,

    #[arg(long, conflicts_with = "no_links", help = "Show symlink targets")]
    links: bool,

    #[arg(long, conflicts_with = "links", help = "Hide symlink targets")]
    no_links: bool,

    #[arg(
        long,
        conflicts_with = "no_prune",
        help = "Prune empty directories after filtering"
    )]
    prune: bool,

    #[arg(
        long,
        conflicts_with = "prune",
        help = "Keep empty directories after filtering"
    )]
    no_prune: bool,

    #[arg(long, conflicts_with = "no_du", help = "Show directory size rollups")]
    du: bool,

    #[arg(long, conflicts_with = "du", help = "Hide directory size rollups")]
    no_du: bool,

    #[arg(long, conflicts_with = "no_git", help = "Show git status")]
    git: bool,

    #[arg(long, conflicts_with = "git", help = "Hide git status")]
    no_git: bool,

    #[arg(long, conflicts_with = "no_perms", help = "Show permissions")]
    perms: bool,

    #[arg(long, conflicts_with = "perms", help = "Hide permissions")]
    no_perms: bool,

    #[arg(long, help = "Show entries grouped by modification recency")]
    timeline: bool,

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
        all: bool_override(cli.all, cli.hide_hidden),
        sizes: bool_override(cli.sizes, cli.no_sizes),
        times: bool_override(cli.times, cli.no_times),
        pattern: cli.pattern,
        exclude: cli.exclude,
        no_ignore: bool_override(cli.no_ignore, cli.ignore),
        no_icons: bool_override(cli.no_icons, cli.icons),
        no_color: bool_override(cli.no_color, cli.color),
        dirs_only: if cli.files_only {
            Some(false)
        } else {
            cli.dirs_only.then_some(true)
        },
        files_only: if cli.dirs_only {
            Some(false)
        } else {
            cli.files_only.then_some(true)
        },
        no_stats: bool_override(cli.no_stats, cli.stats),
        no_links: bool_override(cli.no_links, cli.links),
        no_prune: bool_override(cli.no_prune, cli.prune),
        no_du: bool_override(cli.no_du, cli.du),
        no_git: bool_override(cli.no_git, cli.git),
        no_perms: bool_override(cli.no_perms, cli.perms),
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
    if opts.prune && (opts.pattern.is_some() || opts.exclude.is_some()) {
        prune_empty_dirs(&mut tree);
    }

    if opts.git {
        git::annotate(&root_path, &mut tree);
    }

    sort_nodes(&mut tree, opts.sort);

    let render_opts = RenderOpts {
        show_sizes: opts.sizes,
        show_times: opts.times,
        show_icons: !opts.no_icons,
        show_colors: !opts.no_color,
        dirs_only: opts.dirs_only,
        files_only: opts.files_only,
        show_links: opts.links,
        show_stats: opts.stats,
        show_du: opts.du,
        show_git: opts.git,
        show_perms: opts.perms,
    };

    if cli.timeline {
        render::render_timeline(&tree, &render_opts)?;
        return Ok(());
    }

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

    render::print_stats(&tree, &render_opts);

    Ok(())
}

fn bool_override(enable: bool, disable: bool) -> Option<bool> {
    match (enable, disable) {
        (true, true) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
    }
}

fn prune_empty_dirs(node: &mut TreeNode) {
    for child in &mut node.children {
        if child.is_dir {
            prune_empty_dirs(child);
        }
    }
    node.children
        .retain(|child| !child.is_dir || !child.children.is_empty());
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
