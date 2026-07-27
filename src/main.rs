mod config;
mod git;
mod icon;
mod render;
mod sort;
mod tree;
mod walk;

#[cfg(unix)]
mod capture {
    use std::io::Read;
    use std::os::unix::io::FromRawFd;

    const STDOUT: i32 = 1;

    unsafe extern "C" {
        fn pipe(fds: *mut i32) -> i32;
        fn dup(fd: i32) -> i32;
        fn dup2(old: i32, new: i32) -> i32;
        fn close(fd: i32) -> i32;
    }

    pub fn stdout<F>(f: F) -> Vec<u8>
    where
        F: FnOnce(),
    {
        let mut fds: [i32; 2] = [0; 2];
        unsafe {
            pipe(fds.as_mut_ptr());
            let saved = dup(STDOUT);
            dup2(fds[1], STDOUT);
            close(fds[1]);

            f();

            dup2(saved, STDOUT);
            close(saved);

            let mut buf = Vec::new();
            std::fs::File::from_raw_fd(fds[0])
                .read_to_end(&mut buf)
                .ok();
            buf
        }
    }
}

#[cfg(not(unix))]
mod capture {
    pub fn stdout<F: FnOnce()>(f: F) -> Vec<u8> {
        f();
        Vec::new()
    }
}

use anyhow::{Context, Result};
use clap::Parser;
use fuzzy_matcher::FuzzyMatcher;
use oak::cli::Cli;
use rayon::prelude::*;

use crate::config::{Config, EffectiveConfig, merge_config};
use crate::render::RenderOpts;
use crate::sort::sort_nodes;
use crate::tree::TreeNode;
use crate::walk::walk;

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
        find: cli.find.clone(),
        find_text: cli.find_text.clone(),
        clip: if cli.clip { Some(true) } else { None },
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
        no_git_blame: bool_override(cli.no_git_blame, cli.git_blame),
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

    if let Some(ref find) = opts.find {
        apply_find_filter(&mut tree, find);
        prune_empty_dirs(&mut tree);
    }

    let text_matches = if let Some(ref text) = opts.find_text {
        let m = apply_find_text_filter(&mut tree, text)?;
        prune_empty_dirs(&mut tree);
        m
    } else {
        std::collections::HashMap::new()
    };

    if opts.git {
        git::annotate(&root_path, &mut tree);
    }

    if opts.git_blame {
        git::blame(&root_path, &mut tree);
    }

    sort_nodes(&mut tree, opts.sort);

    if cli.json {
        print_json(&tree);
        return Ok(());
    }
    if cli.csv {
        print_csv(&tree);
        return Ok(());
    }
    if cli.graph {
        print_graph(&tree);
        return Ok(());
    }
    if cli.md {
        print_markdown(&tree);
        return Ok(());
    }
    if cli.html {
        print_html(&tree);
        return Ok(());
    }

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
        show_git_blame: opts.git_blame,
        show_perms: opts.perms,
    };

    let render_result = if opts.clip {
        let captured = capture::stdout(|| {
            run_render(&tree, &render_opts, cli.timeline, &opts).ok();
        });
        let text = String::from_utf8_lossy(&captured);
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text.as_ref());
        }
        print!("{text}");
        Ok(())
    } else {
        run_render(&tree, &render_opts, cli.timeline, &opts)
    };

    render_result?;

    if let Some(ref text) = opts.find_text
        && !text_matches.is_empty()
    {
        print_text_matches(&text_matches, text);
    }

    Ok(())
}

fn print_text_matches(
    matches: &std::collections::HashMap<std::path::PathBuf, Vec<(usize, String)>>,
    search: &str,
) {
    let mut files: Vec<_> = matches.iter().collect();
    files.sort_by_key(|(path, _)| (*path).clone());

    for (path, lines) in &files {
        println!();
        let display = path.display();
        println!("─── {display} ───");
        for (num, content) in lines.iter() {
            print!("  {num:>4} | ");
            highlight_text(content, search);
            println!();
        }
    }
}

fn highlight_text(content: &str, search: &str) {
    use owo_colors::{OwoColorize, Stream::Stdout};

    let search_lower: Vec<char> = search.to_lowercase().chars().collect();
    let chars: Vec<char> = content.chars().collect();
    let lower: Vec<char> = content.to_lowercase().chars().collect();

    let mut i = 0;
    while i < chars.len() {
        if i + search_lower.len() <= lower.len()
            && lower[i..i + search_lower.len()] == search_lower[..]
        {
            let matched: String = chars[i..i + search_lower.len()].iter().collect();
            let styled = matched.if_supports_color(Stdout, |t| t.bright_red());
            let bold_styled = format!("{}", styled);
            print!("{bold_styled}");
            i += search_lower.len();
        } else {
            print!("{}", chars[i]);
            i += 1;
        }
    }
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

fn apply_find_filter(node: &mut TreeNode, find: &str) {
    for child in &mut node.children {
        if child.is_dir {
            apply_find_filter(child, find);
        }
    }

    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    node.children.retain(|child| {
        let name_matches = matcher.fuzzy_match(&child.name, find).is_some();
        if child.is_dir {
            name_matches || !child.children.is_empty()
        } else {
            name_matches
        }
    });
}

type TextMatches = std::collections::HashMap<std::path::PathBuf, Vec<(usize, String)>>;

fn apply_find_text_filter(node: &mut TreeNode, text: &str) -> Result<TextMatches> {
    let total = count_file_leaves(node);

    let (matching_paths, match_details) = if total > 0 {
        let pb = indicatif::ProgressBar::new(total as u64);
        pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:20.cyan/blue}] {pos}/{len} files searching...")
                .unwrap()
                .progress_chars("█▓▒░"),
        );

        let file_paths: Vec<std::path::PathBuf> = collect_file_paths(node);
        let text_lower = text.to_lowercase();

        let results: Vec<(std::path::PathBuf, Vec<(usize, String)>)> = file_paths
            .par_iter()
            .filter_map(|path| {
                pb.inc(1);
                if let Ok(content) = std::fs::read_to_string(path) {
                    let lower = content.to_lowercase();
                    if lower.contains(&text_lower) {
                        let lines: Vec<(usize, String)> = content
                            .lines()
                            .enumerate()
                            .filter(|(_, line)| line.to_lowercase().contains(&text_lower))
                            .map(|(i, line)| (i + 1, line.to_string()))
                            .collect();
                        return Some((path.clone(), lines));
                    }
                }
                None
            })
            .collect();

        pb.finish_and_clear();

        let paths: std::collections::HashSet<_> = results.iter().map(|(p, _)| p.clone()).collect();
        let details: TextMatches = results.into_iter().collect();
        (paths, details)
    } else {
        (std::collections::HashSet::new(), TextMatches::new())
    };

    if total > 0 && matching_paths.is_empty() {
        eprintln!("No files contain \"{text}\"");
    }

    filter_to_matching(node, &matching_paths);
    Ok(match_details)
}

fn collect_file_paths(node: &TreeNode) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    collect_file_paths_recurse(node, &mut paths);
    paths
}

fn collect_file_paths_recurse(node: &TreeNode, paths: &mut Vec<std::path::PathBuf>) {
    if node.is_dir {
        for child in &node.children {
            collect_file_paths_recurse(child, paths);
        }
    } else {
        paths.push(node.path.clone());
    }
}

fn count_file_leaves(node: &TreeNode) -> usize {
    if node.is_dir {
        node.children.iter().map(count_file_leaves).sum()
    } else {
        1
    }
}

fn filter_to_matching(
    node: &mut TreeNode,
    matches: &std::collections::HashSet<std::path::PathBuf>,
) {
    for child in &mut node.children {
        if child.is_dir {
            filter_to_matching(child, matches);
        }
    }
    node.children.retain(|child| {
        if child.is_dir {
            !child.children.is_empty()
        } else {
            matches.contains(&child.path)
        }
    });
}

fn run_render(
    tree: &TreeNode,
    render_opts: &RenderOpts,
    timeline: bool,
    opts: &EffectiveConfig,
) -> Result<()> {
    if timeline {
        render::render_timeline(tree, render_opts)?;
        return Ok(());
    }

    let (dirs, files, total_size) = render::render(tree, render_opts)?;

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

    render::print_stats(tree, render_opts);

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

fn print_json(node: &TreeNode) {
    println!(
        "{}",
        serde_json::to_string_pretty(&node_to_json(node)).unwrap()
    );
}

fn node_to_json(node: &TreeNode) -> serde_json::Value {
    let mut obj = serde_json::json!({
        "name": node.name,
        "path": node.path.to_string_lossy(),
        "size": node.size,
    });
    if node.is_dir {
        let children: Vec<_> = node.children.iter().map(node_to_json).collect();
        obj["type"] = serde_json::json!("directory");
        obj["children"] = serde_json::json!(children);
    } else {
        obj["type"] = serde_json::json!("file");
    }
    obj
}

fn print_csv(node: &TreeNode) {
    println!("path,name,type,size");
    print_csv_recurse(node);
}

fn print_csv_recurse(node: &TreeNode) {
    let type_name = if node.is_dir { "directory" } else { "file" };
    let escaped_path = node.path.to_string_lossy().replace('"', "\"\"");
    let escaped_name = node.name.replace('"', "\"\"");
    println!(
        "\"{escaped_path}\",\"{escaped_name}\",{type_name},{}",
        node.size
    );
    if node.is_dir {
        for child in &node.children {
            print_csv_recurse(child);
        }
    }
}

fn print_graph(node: &TreeNode) {
    println!("digraph oak {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box style=filled];");
    print_graph_recurse(node, &mut 0);
    println!("}}");
}

fn print_graph_recurse(node: &TreeNode, next_id: &mut usize) {
    let id = *next_id;
    *next_id += 1;
    let shape = if node.is_dir {
        "shape=folder fillcolor=lightblue"
    } else {
        "shape=note fillcolor=white"
    };
    let name = escape_dot(&node.name);
    println!("  n{id} [label=\"{name}\", {shape}];");
    for child in &node.children {
        let child_id = *next_id;
        println!("  n{id} -> n{child_id};");
        print_graph_recurse(child, next_id);
    }
}

fn print_markdown(node: &TreeNode) {
    println!("# {}", node.name);
    for child in &node.children {
        print_markdown_recurse(child, "");
    }
}

fn print_markdown_recurse(node: &TreeNode, indent: &str) {
    let icon = if node.is_dir { "📁" } else { "📄" };
    let name = if node.is_dir {
        format!("{}/", node.name)
    } else {
        node.name.clone()
    };
    println!("{indent}- {icon} {name}");
    if node.is_dir {
        let child_indent = format!("{indent}  ");
        for child in &node.children {
            print_markdown_recurse(child, &child_indent);
        }
    }
}

fn print_html(node: &TreeNode) {
    println!("<!DOCTYPE html>");
    println!(
        "<html><head><meta charset=\"utf-8\"><title>{}</title></head>",
        escape_html(&node.name)
    );
    println!("<body>");
    println!("<h1>{}</h1>", escape_html(&node.name));
    println!("<ul>");
    for child in &node.children {
        print_html_recurse(child);
    }
    println!("</ul>");
    println!("</body></html>");
}

fn print_html_recurse(node: &TreeNode) {
    let name = if node.is_dir {
        format!("{}/", node.name)
    } else {
        node.name.clone()
    };
    if node.is_dir {
        println!("<li>{}", escape_html(&name));
        println!("<ul>");
        for child in &node.children {
            print_html_recurse(child);
        }
        println!("</ul>");
        println!("</li>");
    } else {
        let size = render::human_size(node.size);
        println!("<li>{} <small>({size})</small></li>", escape_html(&name));
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
