use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::sort::SortBy;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub level: Option<usize>,
    pub all: Option<bool>,
    pub sizes: Option<bool>,
    pub times: Option<bool>,
    pub pattern: Option<String>,
    pub find: Option<String>,
    pub find_text: Option<String>,
    pub find_regex: Option<String>,
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
    pub find_regex: Option<String>,
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
                "find_regex" => config.find_regex = Some(parse_string(value, idx)?),
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
        if let Some(find_regex) = &self.find_regex {
            output.push_str(&format!(
                "find_regex = \"{}\"\n",
                escape_string(find_regex)
            ));
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
        find_regex: cli.find_regex.or(config.find_regex),
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
