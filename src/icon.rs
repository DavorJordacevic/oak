#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum IconStyle {
    #[default]
    /// Icons that render in stock terminal fonts
    Unicode,
    /// Nerd Font private-use icons
    NerdFont,
}

pub fn get_icon(name: &str, is_dir: bool, is_symlink: bool, style: IconStyle) -> &'static str {
    match style {
        IconStyle::Unicode => get_unicode_icon(name, is_dir, is_symlink),
        IconStyle::NerdFont => get_nerd_font_icon(name, is_dir, is_symlink),
    }
}

fn get_unicode_icon(name: &str, is_dir: bool, is_symlink: bool) -> &'static str {
    if is_symlink {
        return "\u{1f517}";
    }
    if is_dir {
        return "\u{1f4c1}";
    }

    let name_lower = name.to_lowercase();
    match name_lower.as_str() {
        "makefile" | "gnumakefile" => return "\u{2699}",
        "dockerfile" | ".dockerignore" => return "\u{25a3}",
        "license" | "licence" => return "\u{00a7}",
        "readme.md" | "readme" => return "\u{1f4d8}",
        ".gitignore" | ".gitattributes" | ".gitmodules" => return "\u{2387}",
        _ => {}
    }

    let ext = name_lower.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "\u{1f980}",
        "py" | "pyc" | "pyo" | "pyd" => "\u{1f40d}",
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" => "\u{1f4dc}",
        "go" | "java" | "rb" | "php" | "c" | "h" | "cpp" | "cxx" | "cc" | "hpp" | "cs"
        | "swift" | "kt" | "kts" | "scala" | "dart" | "lua" | "r" | "sql" | "sh" | "bash"
        | "zsh" | "fish" | "ps1" | "html" | "css" | "scss" | "sass" | "vue" | "svelte" | "elm"
        | "ex" | "erl" | "hs" | "clj" | "ml" | "nim" => "\u{2328}",
        "md" | "mdx" | "markdown" | "txt" | "rst" => "\u{1f4dd}",
        "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" | "conf" | "lock" | "env"
        | "envrc" => "\u{2699}",
        "svg" | "png" | "jpg" | "jpeg" | "gif" | "ico" | "webp" | "bmp" => "\u{1f5bc}",
        "mp4" | "mkv" | "webm" | "avi" | "mov" => "\u{25b7}",
        "mp3" | "wav" | "flac" | "ogg" | "aac" => "\u{266a}",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "tgz" => "\u{1f4e6}",
        "pdf" => "\u{1f4d5}",
        "doc" | "docx" => "\u{1f4c4}",
        "xls" | "xlsx" => "\u{1f4ca}",
        "ppt" | "pptx" => "\u{1f4bd}",
        "ttf" | "otf" | "woff" | "woff2" | "eot" => "\u{1f524}",
        _ => "\u{1f4c4}",
    }
}

fn get_nerd_font_icon(name: &str, is_dir: bool, is_symlink: bool) -> &'static str {
    if is_symlink {
        return "\u{f0c1}";
    }
    if is_dir {
        return "\u{f07b}";
    }

    let name_lower = name.to_lowercase();

    match name_lower.as_str() {
        "makefile" | "gnumakefile" => return "\u{e60d}",
        "dockerfile" => return "\u{e7b0}",
        "license" | "licence" => return "\u{e60e}",
        "readme.md" | "readme" => return "\u{e609}",
        _ => {}
    }

    let ext = name.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "\u{e7a8}",
        "py" | "pyc" | "pyo" | "pyd" => "\u{e73c}",
        "js" | "mjs" | "cjs" => "\u{e74e}",
        "jsx" => "\u{e7ba}",
        "ts" => "\u{e628}",
        "tsx" => "\u{e799}",
        "go" => "\u{e627}",
        "java" | "class" | "jar" => "\u{e61a}",
        "rb" | "rake" | "gemspec" => "\u{e791}",
        "php" => "\u{e73d}",
        "c" | "h" => "\u{e61e}",
        "cpp" | "cxx" | "cc" | "c++" | "hpp" | "hxx" | "hh" => "\u{e61d}",
        "cs" => "\u{e61c}",
        "swift" => "\u{e755}",
        "kt" | "kts" => "\u{e634}",
        "scala" => "\u{e734}",
        "dart" => "\u{e798}",
        "lua" => "\u{e620}",
        "r" | "rmd" => "\u{e68a}",
        "sql" | "sqlite" | "db" => "\u{e706}",
        "sh" | "bash" | "zsh" | "fish" => "\u{e795}",
        "ps1" | "psm1" | "psd1" => "\u{e795}",
        "html" | "htm" => "\u{e736}",
        "css" => "\u{e749}",
        "scss" | "sass" => "\u{e749}",
        "less" => "\u{e758}",
        "vue" => "\u{e6a5}",
        "svelte" => "\u{e7ab}",
        "elm" => "\u{e62b}",
        "ex" | "exs" | "eex" => "\u{e62d}",
        "erl" | "hrl" => "\u{e7b1}",
        "hs" | "lhs" => "\u{e777}",
        "clj" | "cljs" | "cljc" | "edn" => "\u{e76a}",
        "ml" | "mli" => "\u{e67a}",
        "nim" => "\u{e677}",
        "md" | "mdx" | "markdown" => "\u{e609}",
        "json" => "\u{e60b}",
        "yaml" | "yml" => "\u{e60a}",
        "toml" => "\u{e615}",
        "xml" | "xsl" | "xsd" => "\u{e619}",
        "lock" => "\u{e60d}",
        "gitignore" | "gitattributes" | "gitmodules" => "\u{e602}",
        "env" | "envrc" => "\u{e615}",
        "dockerignore" => "\u{e7b0}",
        "svg" | "png" | "jpg" | "jpeg" | "gif" | "ico" | "webp" | "bmp" => "\u{f1c5}",
        "mp4" | "mkv" | "webm" | "avi" | "mov" => "\u{f1c8}",
        "mp3" | "wav" | "flac" | "ogg" | "aac" => "\u{f1c7}",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "tgz" => "\u{f1c6}",
        "pdf" => "\u{f1c1}",
        "doc" | "docx" => "\u{f1c2}",
        "xls" | "xlsx" => "\u{f1c3}",
        "ppt" | "pptx" => "\u{f1c4}",
        "ttf" | "otf" | "woff" | "woff2" | "eot" => "\u{f031}",
        _ => "\u{f016}",
    }
}
