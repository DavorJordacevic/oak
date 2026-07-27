use clap::Parser;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum SortBy {
    #[default]
    #[value(help = "Sort by last modified time (most recent first)")]
    Mtime,
    #[value(help = "Sort alphabetically by name")]
    Name,
    #[value(help = "Sort by file size (largest first)")]
    Size,
    #[value(help = "Sort by file extension")]
    Ext,
}

impl SortBy {
    pub fn as_str(self) -> &'static str {
        match self {
            SortBy::Mtime => "mtime",
            SortBy::Name => "name",
            SortBy::Size => "size",
            SortBy::Ext => "ext",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "mtime" => Some(SortBy::Mtime),
            "name" => Some(SortBy::Name),
            "size" => Some(SortBy::Size),
            "ext" => Some(SortBy::Ext),
            _ => None,
        }
    }
}

#[derive(Parser)]
#[command(name = "oak")]
#[command(
    version,
    about = "A modern, fast, gitignore-aware directory listing",
    long_about = None
)]
pub struct Cli {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(short = 'L', long, help = "Maximum display depth")]
    pub level: Option<usize>,

    #[arg(
        short = 'a',
        long,
        conflicts_with = "hide_hidden",
        help = "Show hidden files"
    )]
    pub all: bool,

    #[arg(long, conflicts_with = "all", help = "Hide hidden files")]
    pub hide_hidden: bool,

    #[arg(
        short = 's',
        long,
        conflicts_with = "no_sizes",
        help = "Show file sizes"
    )]
    pub sizes: bool,

    #[arg(long, conflicts_with = "sizes", help = "Hide file sizes")]
    pub no_sizes: bool,

    #[arg(
        short = 't',
        long,
        conflicts_with = "no_times",
        help = "Show modification times"
    )]
    pub times: bool,

    #[arg(long, conflicts_with = "times", help = "Hide modification times")]
    pub no_times: bool,

    #[arg(long, help = "Search for files matching name (fuzzy match)")]
    pub find: Option<String>,

    #[arg(long, help = "Copy output to clipboard")]
    pub clip: bool,

    #[arg(long, help = "Search for text in file contents")]
    pub find_text: Option<String>,

    #[arg(short = 'P', long, help = "Only show files matching pattern (regex)")]
    pub pattern: Option<String>,

    #[arg(short = 'I', long, help = "Exclude files matching pattern (regex)")]
    pub exclude: Option<String>,

    #[arg(long, help = "Save these options as future defaults and exit")]
    pub save_config: bool,

    #[arg(long, help = "Do not read saved config")]
    pub no_config: bool,

    #[arg(
        long,
        conflicts_with = "ignore",
        help = "Do not respect .gitignore / .ignore files"
    )]
    pub no_ignore: bool,

    #[arg(
        long,
        conflicts_with = "no_ignore",
        help = "Respect .gitignore / .ignore files"
    )]
    pub ignore: bool,

    #[arg(long, conflicts_with = "icons", help = "Disable icons")]
    pub no_icons: bool,

    #[arg(long, conflicts_with = "no_icons", help = "Enable icons")]
    pub icons: bool,

    #[arg(long, conflicts_with = "color", help = "Output without color")]
    pub no_color: bool,

    #[arg(long, conflicts_with = "no_color", help = "Enable color output")]
    pub color: bool,

    #[arg(long, conflicts_with = "files_only", help = "Show directories only")]
    pub dirs_only: bool,

    #[arg(long, conflicts_with = "dirs_only", help = "Show files only")]
    pub files_only: bool,

    #[arg(long, conflicts_with = "no_stats", help = "Show statistics")]
    pub stats: bool,

    #[arg(long, conflicts_with = "stats", help = "Hide statistics")]
    pub no_stats: bool,

    #[arg(long, conflicts_with = "no_links", help = "Show symlink targets")]
    pub links: bool,

    #[arg(long, conflicts_with = "links", help = "Hide symlink targets")]
    pub no_links: bool,

    #[arg(
        long,
        conflicts_with = "no_prune",
        help = "Prune empty directories after filtering"
    )]
    pub prune: bool,

    #[arg(
        long,
        conflicts_with = "prune",
        help = "Keep empty directories after filtering"
    )]
    pub no_prune: bool,

    #[arg(long, conflicts_with = "no_du", help = "Show directory size rollups")]
    pub du: bool,

    #[arg(long, conflicts_with = "du", help = "Hide directory size rollups")]
    pub no_du: bool,

    #[arg(long, conflicts_with = "no_git", help = "Show git status")]
    pub git: bool,

    #[arg(long, conflicts_with = "git", help = "Hide git status")]
    pub no_git: bool,

    #[arg(
        long,
        conflicts_with = "no_git_blame",
        help = "Show last committer per file"
    )]
    pub git_blame: bool,

    #[arg(
        long,
        conflicts_with = "git_blame",
        help = "Hide last committer per file"
    )]
    pub no_git_blame: bool,

    #[arg(long, conflicts_with = "no_perms", help = "Show permissions")]
    pub perms: bool,

    #[arg(long, conflicts_with = "perms", help = "Hide permissions")]
    pub no_perms: bool,

    #[arg(long, help = "Show entries grouped by modification recency")]
    pub timeline: bool,

    #[arg(short = 'S', long, value_enum, help = "Sort order")]
    pub sort: Option<SortBy>,

    #[arg(long, help = "Export tree as JSON")]
    pub json: bool,

    #[arg(long, help = "Export tree as CSV")]
    pub csv: bool,

    #[arg(long, help = "Export tree as Graphviz DOT for piping to dot")]
    pub graph: bool,

    #[arg(long, help = "Export tree as Markdown list")]
    pub md: bool,

    #[arg(long, help = "Export tree as HTML nested list")]
    pub html: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_by_as_str() {
        assert_eq!(SortBy::Mtime.as_str(), "mtime");
        assert_eq!(SortBy::Name.as_str(), "name");
        assert_eq!(SortBy::Size.as_str(), "size");
        assert_eq!(SortBy::Ext.as_str(), "ext");
    }

    #[test]
    fn sort_by_parse_valid() {
        assert!(matches!(SortBy::parse("mtime"), Some(SortBy::Mtime)));
        assert!(matches!(SortBy::parse("name"), Some(SortBy::Name)));
        assert!(matches!(SortBy::parse("size"), Some(SortBy::Size)));
        assert!(matches!(SortBy::parse("ext"), Some(SortBy::Ext)));
    }

    #[test]
    fn sort_by_parse_invalid() {
        assert!(SortBy::parse("banana").is_none());
        assert!(SortBy::parse("").is_none());
        assert!(SortBy::parse("MTIME").is_none());
    }

    #[test]
    fn sort_by_default_is_mtime() {
        assert!(matches!(SortBy::default(), SortBy::Mtime));
    }

    #[test]
    fn sort_by_roundtrip() {
        for sort in &[SortBy::Mtime, SortBy::Name, SortBy::Size, SortBy::Ext] {
            let parsed = SortBy::parse(sort.as_str()).unwrap();
            assert!(matches!((*sort, parsed), (a, b) if a.as_str() == b.as_str()));
        }
    }
}
