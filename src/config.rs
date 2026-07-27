use std::path::PathBuf;

use anyhow::{Context, Result};

use oak::cli::SortBy;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub level: Option<usize>,
    pub all: Option<bool>,
    pub sizes: Option<bool>,
    pub times: Option<bool>,
    pub pattern: Option<String>,
    pub find: Option<String>,
    pub find_text: Option<String>,
    pub clip: Option<bool>,
    pub exclude: Option<String>,
    pub no_ignore: Option<bool>,
    pub no_icons: Option<bool>,
    pub no_color: Option<bool>,
    pub dirs_only: Option<bool>,
    pub files_only: Option<bool>,
    pub no_stats: Option<bool>,
    pub no_links: Option<bool>,
    pub no_prune: Option<bool>,
    pub no_du: Option<bool>,
    pub no_git: Option<bool>,
    pub no_git_blame: Option<bool>,
    pub no_perms: Option<bool>,
    pub sort: Option<SortBy>,
}

#[derive(Clone, Debug)]
pub struct EffectiveConfig {
    pub level: Option<usize>,
    pub all: bool,
    pub sizes: bool,
    pub times: bool,
    pub pattern: Option<String>,
    pub find: Option<String>,
    pub find_text: Option<String>,
    pub clip: bool,
    pub exclude: Option<String>,
    pub no_ignore: bool,
    pub no_icons: bool,
    pub no_color: bool,
    pub dirs_only: bool,
    pub files_only: bool,
    pub stats: bool,
    pub links: bool,
    pub prune: bool,
    pub du: bool,
    pub git: bool,
    pub git_blame: bool,
    pub perms: bool,
    pub sort: SortBy,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let input = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        Self::parse(&input).with_context(|| format!("Invalid config: {}", path.display()))
    }

    fn parse(input: &str) -> Result<Self> {
        let mut config = Self::default();

        for (idx, line) in input.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                anyhow::bail!("line {}: expected key = value", idx + 1);
            };
            let key = key.trim();
            let value = value.trim();

            match key {
                "level" => config.level = Some(parse_usize(value, idx)?),
                "all" => config.all = Some(parse_bool(value, idx)?),
                "sizes" => config.sizes = Some(parse_bool(value, idx)?),
                "times" => config.times = Some(parse_bool(value, idx)?),
                "pattern" => config.pattern = Some(parse_string(value, idx)?),
                "find" => config.find = Some(parse_string(value, idx)?),
                "find_text" => config.find_text = Some(parse_string(value, idx)?),
                "clip" => config.clip = Some(parse_bool(value, idx)?),
                "exclude" => config.exclude = Some(parse_string(value, idx)?),
                "no_ignore" => config.no_ignore = Some(parse_bool(value, idx)?),
                "no_icons" => config.no_icons = Some(parse_bool(value, idx)?),
                "no_color" => config.no_color = Some(parse_bool(value, idx)?),
                "dirs_only" => config.dirs_only = Some(parse_bool(value, idx)?),
                "files_only" => config.files_only = Some(parse_bool(value, idx)?),
                "no_stats" => config.no_stats = Some(parse_bool(value, idx)?),
                "no_links" => config.no_links = Some(parse_bool(value, idx)?),
                "no_prune" => config.no_prune = Some(parse_bool(value, idx)?),
                "no_du" => config.no_du = Some(parse_bool(value, idx)?),
                "no_git" => config.no_git = Some(parse_bool(value, idx)?),
                "no_git_blame" => config.no_git_blame = Some(parse_bool(value, idx)?),
                "no_perms" => config.no_perms = Some(parse_bool(value, idx)?),
                "sort" => {
                    let sort = parse_string(value, idx)?;
                    config.sort = Some(
                        SortBy::parse(&sort)
                            .ok_or_else(|| anyhow::anyhow!("line {}: invalid sort", idx + 1))?,
                    );
                }
                _ => anyhow::bail!("line {}: unknown key `{}`", idx + 1, key),
            }
        }

        Ok(config)
    }
}

impl EffectiveConfig {
    pub fn save(&self) -> Result<PathBuf> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir: {}", parent.display()))?;
        }
        std::fs::write(&path, self.to_text())
            .with_context(|| format!("Failed to write config: {}", path.display()))?;
        Ok(path)
    }

    fn to_text(&self) -> String {
        let mut output = String::new();
        output.push_str("# Oak config\n");
        if let Some(level) = self.level {
            output.push_str(&format!("level = {level}\n"));
        }
        output.push_str(&format!("all = {}\n", self.all));
        output.push_str(&format!("sizes = {}\n", self.sizes));
        output.push_str(&format!("times = {}\n", self.times));
        if let Some(pattern) = &self.pattern {
            output.push_str(&format!("pattern = \"{}\"\n", escape_string(pattern)));
        }
        if let Some(find) = &self.find {
            output.push_str(&format!("find = \"{}\"\n", escape_string(find)));
        }
        if let Some(find_text) = &self.find_text {
            output.push_str(&format!("find_text = \"{}\"\n", escape_string(find_text)));
        }
        if let Some(exclude) = &self.exclude {
            output.push_str(&format!("exclude = \"{}\"\n", escape_string(exclude)));
        }
        output.push_str(&format!("no_ignore = {}\n", self.no_ignore));
        output.push_str(&format!("no_icons = {}\n", self.no_icons));
        output.push_str(&format!("no_color = {}\n", self.no_color));
        output.push_str(&format!("dirs_only = {}\n", self.dirs_only));
        output.push_str(&format!("files_only = {}\n", self.files_only));
        output.push_str(&format!("no_stats = {}\n", !self.stats));
        output.push_str(&format!("no_links = {}\n", !self.links));
        output.push_str(&format!("no_prune = {}\n", !self.prune));
        output.push_str(&format!("no_du = {}\n", !self.du));
        output.push_str(&format!("no_git = {}\n", !self.git));
        output.push_str(&format!("no_git_blame = {}\n", !self.git_blame));
        output.push_str(&format!("no_perms = {}\n", !self.perms));
        output.push_str(&format!("sort = \"{}\"\n", self.sort.as_str()));
        output
    }
}

pub fn config_path() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(dir).join("oak").join("config"));
    }

    let home = std::env::var_os("HOME")
        .ok_or_else(|| anyhow::anyhow!("Could not determine config path: HOME is not set"))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("oak")
        .join("config"))
}

pub fn merge_config(cli: Config, config: Config) -> EffectiveConfig {
    EffectiveConfig {
        level: cli.level.or(config.level),
        all: cli.all.or(config.all).unwrap_or(false),
        sizes: cli.sizes.or(config.sizes).unwrap_or(true),
        times: cli.times.or(config.times).unwrap_or(false),
        pattern: cli.pattern.or(config.pattern),
        find: cli.find.or(config.find),
        find_text: cli.find_text.or(config.find_text),
        clip: cli.clip.or(config.clip).unwrap_or(false),
        exclude: cli.exclude.or(config.exclude),
        no_ignore: cli.no_ignore.or(config.no_ignore).unwrap_or(false),
        no_icons: cli.no_icons.or(config.no_icons).unwrap_or(false),
        no_color: cli.no_color.or(config.no_color).unwrap_or(false),
        dirs_only: cli.dirs_only.or(config.dirs_only).unwrap_or(false),
        files_only: cli.files_only.or(config.files_only).unwrap_or(false),
        stats: !cli.no_stats.or(config.no_stats).unwrap_or(false),
        links: !cli.no_links.or(config.no_links).unwrap_or(false),
        prune: !cli.no_prune.or(config.no_prune).unwrap_or(false),
        du: !cli.no_du.or(config.no_du).unwrap_or(false),
        git: !cli.no_git.or(config.no_git).unwrap_or(false),
        // Blame requires a Git invocation for every tracked file, so unlike
        // status it is intentionally opt-in.
        git_blame: !cli.no_git_blame.or(config.no_git_blame).unwrap_or(true),
        perms: !cli.no_perms.or(config.no_perms).unwrap_or(false),
        sort: cli.sort.or(config.sort).unwrap_or_default(),
    }
}

fn parse_bool(value: &str, idx: usize) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!("line {}: expected true or false", idx + 1),
    }
}

fn parse_usize(value: &str, idx: usize) -> Result<usize> {
    value
        .parse()
        .with_context(|| format!("line {}: expected positive integer", idx + 1))
}

fn parse_string(value: &str, idx: usize) -> Result<String> {
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        anyhow::bail!("line {}: expected quoted string", idx + 1);
    }

    let raw = &value[1..value.len() - 1];
    Ok(raw.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_config() {
        let config = Config::parse("").unwrap();
        assert!(config.level.is_none());
        assert!(config.all.is_none());
        assert!(config.sizes.is_none());
    }

    #[test]
    fn parse_comments_and_blanks_skipped() {
        let config =
            Config::parse("# a comment\n\nall = true\n\n# another\nno_icons = true").unwrap();
        assert_eq!(config.all, Some(true));
        assert_eq!(config.no_icons, Some(true));
    }

    #[test]
    fn parse_bools() {
        let config = Config::parse("all = true\nsizes = false\ntimes = true").unwrap();
        assert_eq!(config.all, Some(true));
        assert_eq!(config.sizes, Some(false));
        assert_eq!(config.times, Some(true));
    }

    #[test]
    fn parse_level() {
        let config = Config::parse("level = 3").unwrap();
        assert_eq!(config.level, Some(3));
    }

    #[test]
    fn parse_quoted_strings() {
        let config = Config::parse("pattern = \"\\.rs$\"\nexclude = \"target\"").unwrap();
        assert_eq!(config.pattern.as_deref(), Some("\\.rs$"));
        assert_eq!(config.exclude.as_deref(), Some("target"));
    }

    #[test]
    fn parse_escaped_quotes_in_strings() {
        let config = Config::parse("find = \"hello \\\"world\\\"\"").unwrap();
        assert_eq!(config.find.as_deref(), Some("hello \"world\""));
    }

    #[test]
    fn parse_sort_valid() {
        let config = Config::parse("sort = \"name\"").unwrap();
        assert!(matches!(config.sort, Some(SortBy::Name)));
    }

    #[test]
    fn parse_sort_invalid_is_error() {
        let err = Config::parse("sort = \"banana\"").unwrap_err();
        assert!(err.to_string().contains("invalid sort"));
    }

    #[test]
    fn parse_invalid_bool_is_error() {
        let err = Config::parse("all = maybe").unwrap_err();
        assert!(err.to_string().contains("expected true or false"));
    }

    #[test]
    fn parse_invalid_int_is_error() {
        let err = Config::parse("level = abc").unwrap_err();
        assert!(err.to_string().contains("expected positive integer"));
    }

    #[test]
    fn parse_unquoted_string_is_error() {
        let err = Config::parse("pattern = unquoted").unwrap_err();
        assert!(err.to_string().contains("expected quoted string"));
    }

    #[test]
    fn parse_unknown_key_is_error() {
        let err = Config::parse("wizard = true").unwrap_err();
        assert!(err.to_string().contains("unknown key"));
    }

    #[test]
    fn parse_missing_equals_is_error() {
        let err = Config::parse("no_equals_here").unwrap_err();
        assert!(err.to_string().contains("expected key = value"));
    }

    #[test]
    fn merge_defaults() {
        let result = merge_config(Config::default(), Config::default());
        assert!(!result.all);
        assert!(result.sizes);
        assert!(!result.times);
        assert!(!result.no_icons);
        assert!(!result.dirs_only);
        assert!(!result.files_only);
        assert!(result.stats);
        assert!(result.links);
        assert!(result.prune);
        assert!(result.du);
        assert!(result.git);
        assert!(!result.git_blame);
        assert!(result.perms);
    }

    #[test]
    fn merge_cli_overrides_config() {
        let cli = Config {
            all: Some(true),
            ..Config::default()
        };
        let file = Config {
            all: Some(false),
            ..Config::default()
        };
        let result = merge_config(cli, file);
        assert!(result.all);
    }

    #[test]
    fn merge_config_falls_back_to_file() {
        let cli = Config::default();
        let file = Config {
            all: Some(true),
            ..Config::default()
        };
        let result = merge_config(cli, file);
        assert!(result.all);
    }

    #[test]
    fn to_text_roundtrip() {
        let config = EffectiveConfig {
            level: Some(2),
            all: true,
            sizes: true,
            times: false,
            pattern: Some("\\.rs$".to_string()),
            find: None,
            find_text: None,
            clip: false,
            exclude: Some("target".to_string()),
            no_ignore: false,
            no_icons: true,
            no_color: false,
            dirs_only: false,
            files_only: false,
            stats: true,
            links: true,
            prune: true,
            du: true,
            git: true,
            git_blame: false,
            perms: true,
            sort: SortBy::Name,
        };
        let text = config.to_text();
        let parsed = Config::parse(&text).unwrap();
        let merged = merge_config(parsed, Config::default());
        assert_eq!(merged.level, config.level);
        assert_eq!(merged.all, config.all);
        assert_eq!(merged.sizes, config.sizes);
        assert_eq!(merged.times, config.times);
        assert_eq!(merged.pattern, config.pattern);
        assert_eq!(merged.exclude, config.exclude);
        assert_eq!(merged.no_icons, config.no_icons);
        assert_eq!(merged.no_color, config.no_color);
        assert_eq!(merged.sort.as_str(), config.sort.as_str());
    }

    #[test]
    fn parse_string_handles_backslash_escape() {
        let config = Config::parse("pattern = \"C:\\\\Users\\\\test\"").unwrap();
        // \\\\ in TOML-like string → \\ in parsed value
        assert_eq!(config.pattern.as_deref(), Some("C:\\Users\\test"));
    }

    #[test]
    fn escape_string_roundtrip() {
        let original = r#"hello "world" with\backslash"#;
        let escaped = escape_string(original);
        let input = format!("\"{}\"", escaped);
        let parsed = parse_string(&input, 0).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn effective_config_save_values() {
        let ec = EffectiveConfig {
            level: None,
            all: false,
            sizes: true,
            times: false,
            pattern: None,
            find: None,
            find_text: None,
            clip: false,
            exclude: None,
            no_ignore: false,
            no_icons: true,
            no_color: false,
            dirs_only: false,
            files_only: false,
            stats: false,
            links: false,
            prune: false,
            du: false,
            git: false,
            git_blame: false,
            perms: false,
            sort: SortBy::Mtime,
        };
        let text = ec.to_text();
        assert!(text.contains("no_stats = true"));
        assert!(text.contains("no_links = true"));
        assert!(text.contains("no_prune = true"));
        assert!(text.contains("no_du = true"));
        assert!(text.contains("no_git = true"));
        assert!(text.contains("no_perms = true"));
        assert!(text.contains("no_git_blame = true"));
        assert!(text.contains("sort = \"mtime\""));
    }

    #[test]
    fn parse_bool_accepts_true_and_false() {
        assert!(parse_bool("true", 0).unwrap());
        assert!(!parse_bool("false", 0).unwrap());
    }

    #[test]
    fn parse_bool_rejects_other() {
        assert!(parse_bool("yes", 0).is_err());
        assert!(parse_bool("", 0).is_err());
        assert!(parse_bool("TRUE", 0).is_err());
    }

    #[test]
    fn parse_usize_valid_and_invalid() {
        assert_eq!(parse_usize("42", 0).unwrap(), 42);
        assert!(parse_usize("-1", 0).is_err());
        assert!(parse_usize("", 0).is_err());
    }

    #[test]
    fn merge_cli_no_stats() {
        let cli = Config {
            no_stats: Some(true),
            ..Config::default()
        };
        let result = merge_config(cli, Config::default());
        assert!(!result.stats);
    }

    #[test]
    fn merge_no_perms_overrides_file_perms() {
        let cli = Config {
            no_perms: Some(true),
            ..Config::default()
        };
        let file = Config {
            no_perms: Some(false),
            ..Config::default()
        };
        let result = merge_config(cli, file);
        assert!(!result.perms);
    }

    #[test]
    fn merge_clip_enabled() {
        let cli = Config {
            clip: Some(true),
            ..Config::default()
        };
        let result = merge_config(cli, Config::default());
        assert!(result.clip);
    }

    #[test]
    fn merge_find_text_from_cli() {
        let cli = Config {
            find_text: Some("needle".to_string()),
            ..Config::default()
        };
        let result = merge_config(cli, Config::default());
        assert_eq!(result.find_text.as_deref(), Some("needle"));
    }

    #[test]
    fn merge_git_blame_is_opt_in() {
        let cli = Config {
            no_git_blame: Some(false),
            ..Config::default()
        };
        let result = merge_config(cli, Config::default());
        assert!(result.git_blame);
    }

    #[test]
    fn parse_all_known_fields() {
        let input = "\
level = 5
all = true
sizes = false
times = true
pattern = \"\\.rs$\"
find = \"main\"
find_text = \"hello\"
clip = false
exclude = \"target\"
no_ignore = true
no_icons = false
no_color = true
dirs_only = false
files_only = true
no_stats = true
no_links = false
no_prune = false
no_du = true
no_git = false
no_git_blame = true
no_perms = false
sort = \"ext\"
";
        let config = Config::parse(input).unwrap();
        assert_eq!(config.level, Some(5));
        assert_eq!(config.all, Some(true));
        assert_eq!(config.sizes, Some(false));
        assert_eq!(config.times, Some(true));
        assert_eq!(config.pattern.as_deref(), Some("\\.rs$"));
        assert_eq!(config.find.as_deref(), Some("main"));
        assert_eq!(config.find_text.as_deref(), Some("hello"));
        assert_eq!(config.clip, Some(false));
        assert_eq!(config.exclude.as_deref(), Some("target"));
        assert_eq!(config.no_ignore, Some(true));
        assert_eq!(config.no_icons, Some(false));
        assert_eq!(config.no_color, Some(true));
        assert_eq!(config.dirs_only, Some(false));
        assert_eq!(config.files_only, Some(true));
        assert_eq!(config.no_stats, Some(true));
        assert_eq!(config.no_links, Some(false));
        assert_eq!(config.no_prune, Some(false));
        assert_eq!(config.no_du, Some(true));
        assert_eq!(config.no_git, Some(false));
        assert_eq!(config.no_git_blame, Some(true));
        assert_eq!(config.no_perms, Some(false));
        assert!(matches!(config.sort, Some(SortBy::Ext)));
    }
}
