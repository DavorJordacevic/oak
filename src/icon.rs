pub fn get_icon(name: &str, is_dir: bool, is_symlink: bool) -> &'static str {
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
