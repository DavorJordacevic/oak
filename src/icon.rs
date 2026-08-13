pub fn get_icon(name: &str, is_dir: bool, is_symlink: bool) -> &'static str {
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
