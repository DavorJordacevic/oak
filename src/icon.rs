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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symlink_icon() {
        assert_eq!(get_icon("anyfile", false, true), "\u{1f517}");
    }

    #[test]
    fn directory_icon() {
        assert_eq!(get_icon("somedir", true, false), "\u{1f4c1}");
    }

    #[test]
    fn symlink_icon_beats_dir() {
        assert_eq!(get_icon("somedir", true, true), "\u{1f517}");
    }

    #[test]
    fn makefile_icon() {
        assert_eq!(get_icon("Makefile", false, false), "\u{2699}");
    }

    #[test]
    fn dockerfile_icon() {
        assert_eq!(get_icon("Dockerfile", false, false), "\u{25a3}");
    }

    #[test]
    fn license_icon() {
        assert_eq!(get_icon("LICENSE", false, false), "\u{00a7}");
    }

    #[test]
    fn readme_icon() {
        assert_eq!(get_icon("README.md", false, false), "\u{1f4d8}");
    }

    #[test]
    fn gitignore_icon() {
        assert_eq!(get_icon(".gitignore", false, false), "\u{2387}");
    }

    #[test]
    fn rust_icon() {
        assert_eq!(get_icon("main.rs", false, false), "\u{1f980}");
    }

    #[test]
    fn python_icon() {
        assert_eq!(get_icon("app.py", false, false), "\u{1f40d}");
    }

    #[test]
    fn javascript_icon() {
        assert_eq!(get_icon("app.js", false, false), "\u{1f4dc}");
        assert_eq!(get_icon("app.ts", false, false), "\u{1f4dc}");
    }

    #[test]
    fn markdown_icon() {
        assert_eq!(get_icon("notes.md", false, false), "\u{1f4dd}");
    }

    #[test]
    fn config_icon() {
        assert_eq!(get_icon("config.json", false, false), "\u{2699}");
        assert_eq!(get_icon("config.toml", false, false), "\u{2699}");
    }

    #[test]
    fn image_icon() {
        assert_eq!(get_icon("photo.png", false, false), "\u{1f5bc}");
        assert_eq!(get_icon("photo.jpg", false, false), "\u{1f5bc}");
    }

    #[test]
    fn video_icon() {
        assert_eq!(get_icon("movie.mp4", false, false), "\u{25b7}");
    }

    #[test]
    fn audio_icon() {
        assert_eq!(get_icon("song.mp3", false, false), "\u{266a}");
    }

    #[test]
    fn archive_icon() {
        assert_eq!(get_icon("archive.zip", false, false), "\u{1f4e6}");
    }

    #[test]
    fn pdf_icon() {
        assert_eq!(get_icon("doc.pdf", false, false), "\u{1f4d5}");
    }

    #[test]
    fn docx_icon() {
        assert_eq!(get_icon("report.docx", false, false), "\u{1f4c4}");
    }

    #[test]
    fn xlsx_icon() {
        assert_eq!(get_icon("data.xlsx", false, false), "\u{1f4ca}");
    }

    #[test]
    fn pptx_icon() {
        assert_eq!(get_icon("slides.pptx", false, false), "\u{1f4bd}");
    }

    #[test]
    fn font_icon() {
        assert_eq!(get_icon("font.ttf", false, false), "\u{1f524}");
    }

    #[test]
    fn unknown_icon() {
        assert_eq!(get_icon("unknown.xyz", false, false), "\u{1f4c4}");
    }

    #[test]
    fn no_extension_icon() {
        assert_eq!(get_icon("noext", false, false), "\u{1f4c4}");
    }
}
