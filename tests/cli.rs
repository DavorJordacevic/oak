use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn oak() -> Command {
    Command::new(env!("CARGO_BIN_EXE_oak"))
}

struct TempTree {
    path: PathBuf,
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn temp_tree(name: &str) -> TempTree {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("oak-test-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("failed to create temp tree");
    TempTree { path }
}

fn temp_config_home(name: &str) -> TempTree {
    temp_tree(name)
}

#[test]
fn invalid_include_regex_returns_clean_error() {
    let output = oak()
        .args(["--no-color", "--no-icons", "-P", "[", "."])
        .output()
        .expect("failed to run oak");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("Invalid include pattern"),
        "stderr was: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "stderr was: {stderr}");
}

#[test]
fn files_only_still_shows_nested_files() {
    let root = temp_tree("files-only");
    fs::create_dir_all(root.path.join("a")).expect("failed to create nested dir");
    fs::write(root.path.join("a").join("a.txt"), "").expect("failed to write nested file");
    fs::write(root.path.join("root.txt"), "").expect("failed to write root file");

    let output = oak()
        .args(["--no-color", "--no-icons", "--files-only"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("a.txt"), "stdout was: {stdout}");
    assert!(stdout.contains("root.txt"), "stdout was: {stdout}");
    assert!(stdout.contains("2 files"), "stdout was: {stdout}");
}

#[test]
fn dirs_only_uses_connectors_from_visible_children() {
    let root = temp_tree("dirs-only");
    fs::create_dir_all(root.path.join("a")).expect("failed to create first dir");
    fs::create_dir_all(root.path.join("b")).expect("failed to create second dir");
    fs::write(root.path.join("root.txt"), "").expect("failed to write root file");

    let output = oak()
        .args(["--no-color", "--no-icons", "--dirs-only", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("├── drwxr-xr-x  a/"),
        "stdout was: {stdout}"
    );
    assert!(
        stdout.contains("└── drwxr-xr-x  b/"),
        "stdout was: {stdout}"
    );
}

#[test]
fn default_icons_use_mac_terminal_visible_unicode() {
    let output = oak()
        .args(["--no-color", "-L", "0", "."])
        .output()
        .expect("failed to run oak");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("\u{1f4c1}"), "stdout was: {stdout}");
}

#[test]
fn icon_style_option_is_not_supported() {
    let output = oak()
        .args(["--no-color", "--icon-style", "nerd-font", "-L", "0", "."])
        .output()
        .expect("failed to run oak");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(
        stderr.contains("unexpected argument '--icon-style'"),
        "stderr was: {stderr}"
    );
}

#[test]
fn unicode_image_icons_use_picture_symbol() {
    let root = temp_tree("image-icons");
    fs::write(root.path.join("oak.png"), "").expect("failed to write image file");

    let output = oak()
        .args(["--no-color", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("\u{1f5bc} oak.png"), "stdout was: {stdout}");
    assert!(!stdout.contains("\u{25a7} oak.png"), "stdout was: {stdout}");
}

#[test]
fn save_config_writes_defaults_for_future_runs() {
    let root = temp_tree("config-root");
    let config_home = temp_config_home("config-home");
    fs::create_dir_all(root.path.join("hidden")).expect("failed to create hidden dir");
    fs::write(root.path.join(".secret"), "").expect("failed to write hidden file");
    fs::write(root.path.join("visible.txt"), "").expect("failed to write visible file");

    let save = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--all", "--no-icons", "--save-config"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    assert!(
        save.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&save.stderr)
    );

    let output = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains(".secret"), "stdout was: {stdout}");
    assert!(!stdout.contains("\u{1f4c1}"), "stdout was: {stdout}");

    let config_path = config_home.path.join("oak").join("config");
    let config = fs::read_to_string(config_path).expect("failed to read saved config");
    assert!(config.contains("all = true"), "config was: {config}");
    assert!(config.contains("no_icons = true"), "config was: {config}");
}

#[test]
fn no_config_bypasses_saved_defaults() {
    let root = temp_tree("no-config-root");
    let config_home = temp_config_home("no-config-home");
    fs::write(root.path.join(".secret"), "").expect("failed to write hidden file");

    let save = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--all", "--save-config"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    assert!(
        save.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&save.stderr)
    );

    let output = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--no-config"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!stdout.contains(".secret"), "stdout was: {stdout}");
}

#[test]
fn files_only_flag_overrides_saved_dirs_only() {
    let root = temp_tree("config-mode-override");
    let config_home = temp_config_home("config-mode-home");
    fs::create_dir_all(root.path.join("dir")).expect("failed to create dir");
    fs::write(root.path.join("file.txt"), "").expect("failed to write file");

    let save = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--dirs-only", "--save-config"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    assert!(
        save.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&save.stderr)
    );

    let output = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--files-only"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("file.txt"), "stdout was: {stdout}");
    assert!(stdout.contains("1 file"), "stdout was: {stdout}");
    assert!(!stdout.contains("dir/"), "stdout was: {stdout}");
}

#[test]
fn icons_flag_overrides_saved_no_icons() {
    let root = temp_tree("config-icons-override");
    let config_home = temp_config_home("config-icons-home");
    fs::write(root.path.join("file.txt"), "").expect("failed to write file");

    let save = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--no-icons", "--save-config"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    assert!(
        save.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&save.stderr)
    );

    let output = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--icons"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("\u{1f4dd} file.txt"),
        "stdout was: {stdout}"
    );
}

#[test]
#[cfg(unix)]
fn symlinks_show_targets_and_broken_state_by_default() {
    use std::os::unix::fs::symlink;

    let root = temp_tree("links");
    fs::write(root.path.join("target.txt"), "").expect("failed to write target");
    symlink("target.txt", root.path.join("good-link")).expect("failed to create symlink");
    symlink("missing.txt", root.path.join("bad-link")).expect("failed to create broken symlink");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("good-link  10 B  -> target.txt"),
        "stdout was: {stdout}"
    );
    assert!(
        stdout.contains("bad-link  11 B  -> missing.txt [broken]"),
        "stdout was: {stdout}"
    );
}

#[test]
fn stats_and_directory_sizes_are_on_by_default() {
    let root = temp_tree("stats-du");
    fs::create_dir_all(root.path.join("src")).expect("failed to create dir");
    fs::write(root.path.join("src").join("main.rs"), "fn main() {}\n").expect("failed to write rs");
    fs::write(root.path.join("README.md"), "# Readme\n").expect("failed to write md");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("drwxr-xr-x  src/  13 B"),
        "stdout was: {stdout}"
    );
    assert!(stdout.contains("by type:"), "stdout was: {stdout}");
    assert!(stdout.contains("Rust"), "stdout was: {stdout}");
    assert!(stdout.contains("Markdown"), "stdout was: {stdout}");
    assert!(stdout.contains("largest:"), "stdout was: {stdout}");
}

#[test]
fn pattern_filter_prunes_empty_directories_by_default() {
    let root = temp_tree("prune");
    fs::create_dir_all(root.path.join("src")).expect("failed to create matching dir");
    fs::create_dir_all(root.path.join("docs")).expect("failed to create empty dir");
    fs::write(root.path.join("src").join("main.rs"), "").expect("failed to write rs");
    fs::write(root.path.join("docs").join("guide.md"), "").expect("failed to write md");

    let output = oak()
        .args([
            "--no-config",
            "--no-color",
            "--no-icons",
            "-P",
            "\\.rs$",
            "-S",
            "name",
        ])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("src/"), "stdout was: {stdout}");
    assert!(!stdout.contains("docs/"), "stdout was: {stdout}");
}

#[test]
fn git_status_is_shown_by_default_when_available() {
    let root = temp_tree("git");
    let init = Command::new("git").args(["init"]).arg(&root.path).output();
    let Ok(init) = init else {
        return;
    };
    if !init.status.success() {
        return;
    }
    fs::write(root.path.join("new.txt"), "").expect("failed to write untracked file");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("-rw-r--r--  new.txt  0 B  ??"),
        "stdout was: {stdout}"
    );
}

#[test]
fn git_status_rolls_up_to_parent_directories() {
    let root = temp_tree("git-rollup");
    let init = Command::new("git").args(["init"]).arg(&root.path).output();
    let Ok(init) = init else {
        return;
    };
    if !init.status.success() {
        return;
    }

    fs::create_dir_all(root.path.join("src")).expect("failed to create src");
    fs::write(root.path.join("src").join("main.rs"), "fn main() {}\n").expect("failed to write");
    let add = Command::new("git")
        .args(["-C"])
        .arg(&root.path)
        .args(["add", "."])
        .output()
        .expect("failed to git add");
    if !add.status.success() {
        return;
    }
    let commit = Command::new("git")
        .args(["-C"])
        .arg(&root.path)
        .args([
            "-c",
            "user.name=Oak Test",
            "-c",
            "user.email=oak@example.invalid",
            "commit",
            "-m",
            "initial",
        ])
        .output()
        .expect("failed to git commit");
    if !commit.status.success() {
        return;
    }
    fs::write(
        root.path.join("src").join("main.rs"),
        "fn main() { println!(\"hi\"); }\n",
    )
    .expect("failed to modify");

    let output = oak()
        .args([
            "--no-config",
            "--no-color",
            "--no-icons",
            "-L",
            "1",
            "-S",
            "name",
        ])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("drwxr-xr-x  src/  0 B  M"),
        "stdout was: {stdout}"
    );
}

#[test]
#[cfg(unix)]
fn saved_config_can_disable_new_default_features() {
    use std::os::unix::fs::symlink;

    let root = temp_tree("disable-defaults");
    let config_home = temp_config_home("disable-defaults-config");
    fs::write(root.path.join("target.txt"), "").expect("failed to write target");
    symlink("target.txt", root.path.join("link")).expect("failed to create symlink");

    let save = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args([
            "--no-stats",
            "--no-links",
            "--no-du",
            "--no-git",
            "--no-prune",
            "--no-perms",
            "--save-config",
        ])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    assert!(
        save.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&save.stderr)
    );

    let output = oak()
        .env("XDG_CONFIG_HOME", &config_home.path)
        .args(["--no-color", "--no-icons", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !stdout.contains("link  -> target.txt"),
        "stdout was: {stdout}"
    );
    assert!(!stdout.contains("by type:"), "stdout was: {stdout}");

    let config = fs::read_to_string(config_home.path.join("oak").join("config"))
        .expect("failed to read saved config");
    assert!(config.contains("no_stats = true"), "config was: {config}");
    assert!(config.contains("no_links = true"), "config was: {config}");
    assert!(config.contains("no_du = true"), "config was: {config}");
    assert!(config.contains("no_git = true"), "config was: {config}");
    assert!(config.contains("no_prune = true"), "config was: {config}");
    assert!(config.contains("no_perms = true"), "config was: {config}");
}

#[test]
#[cfg(unix)]
fn permissions_are_shown_by_default_and_can_be_disabled() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_tree("perms");
    let file = root.path.join("run.sh");
    fs::write(&file, "").expect("failed to write file");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755))
        .expect("failed to set permissions");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("-rwxr-xr-x  run.sh"),
        "stdout was: {stdout}"
    );

    let output = oak()
        .args([
            "--no-config",
            "--no-color",
            "--no-icons",
            "--no-perms",
            "-S",
            "name",
        ])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!stdout.contains("-rwxr-xr-x"), "stdout was: {stdout}");
}

#[test]
fn timeline_groups_entries_by_recency() {
    let root = temp_tree("timeline");
    fs::create_dir_all(root.path.join("src")).expect("failed to create src");
    fs::write(root.path.join("src").join("main.rs"), "").expect("failed to write file");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "--timeline"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Today:"), "stdout was: {stdout}");
    assert!(stdout.contains("src/"), "stdout was: {stdout}");
    assert!(stdout.contains("src/main.rs"), "stdout was: {stdout}");
    assert!(!stdout.contains("├──"), "stdout was: {stdout}");
}

#[test]
fn timeline_respects_color_output() {
    let root = temp_tree("timeline-color");
    fs::write(root.path.join("main.rs"), "").expect("failed to write file");

    let output = oak()
        .env("FORCE_COLOR", "1")
        .args(["--no-config", "--color", "--timeline"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("\u{1b}["), "stdout was: {stdout:?}");
    assert!(stdout.contains("Today:"), "stdout was: {stdout}");
}

#[test]
fn clip_copies_output() {
    let root = temp_tree("clip");
    fs::write(root.path.join("hello.txt"), "world").expect("failed to write file");
    fs::write(root.path.join("other.md"), "content").expect("failed to write file");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "--clip", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("hello.txt"),
        "stdout should contain the tree, was: {stdout}"
    );
    assert!(
        stdout.contains("other.md"),
        "stdout should contain the tree, was: {stdout}"
    );
}

#[test]
fn find_shows_only_matching_files() {
    let root = temp_tree("find");
    fs::create_dir_all(root.path.join("src")).expect("failed to create src dir");
    fs::create_dir_all(root.path.join("docs")).expect("failed to create docs dir");
    fs::create_dir_all(root.path.join("target")).expect("failed to create target dir");
    fs::write(root.path.join("src").join("main.rs"), "").expect("failed to write main.rs");
    fs::write(root.path.join("src").join("lib.rs"), "").expect("failed to write lib.rs");
    fs::write(root.path.join("docs").join("guide.md"), "").expect("failed to write guide.md");
    fs::write(root.path.join("README.md"), "").expect("failed to write readme");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "--find", "main", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("src/"), "stdout was: {stdout}");
    assert!(stdout.contains("main.rs"), "stdout was: {stdout}");
    assert!(!stdout.contains("lib.rs"), "stdout was: {stdout}");
    assert!(!stdout.contains("docs/"), "stdout was: {stdout}");
    assert!(!stdout.contains("target/"), "stdout was: {stdout}");
    assert!(!stdout.contains("README.md"), "stdout was: {stdout}");
    assert!(!stdout.contains("guide.md"), "stdout was: {stdout}");
}

#[test]
fn find_is_case_insensitive() {
    let root = temp_tree("find-case");
    fs::create_dir_all(root.path.join("Src")).expect("failed to create Src dir");
    fs::write(root.path.join("Src").join("Main.Rs"), "").expect("failed to write Main.Rs");
    fs::write(root.path.join("README.md"), "").expect("failed to write README");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "--find", "main", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Src/"), "stdout was: {stdout}");
    assert!(stdout.contains("Main.Rs"), "stdout was: {stdout}");
    assert!(!stdout.contains("README.md"), "stdout was: {stdout}");
}

#[test]
fn find_text_shows_files_containing_search_text() {
    let root = temp_tree("find-text");
    fs::create_dir_all(root.path.join("src")).expect("failed to create src dir");
    fs::create_dir_all(root.path.join("docs")).expect("failed to create docs dir");
    fs::write(root.path.join("src").join("main.rs"), "fn main() { println!(\"hello\"); }")
        .expect("failed to write main.rs");
    fs::write(root.path.join("src").join("lib.rs"), "pub fn helper() {}")
        .expect("failed to write lib.rs");
    fs::write(root.path.join("docs").join("guide.md"), "# Guide\nhello world")
        .expect("failed to write guide.md");
    fs::write(root.path.join("README.md"), "# Project\nwelcome").expect("failed to write readme");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "--find-text", "hello", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("src/"), "stdout was: {stdout}");
    assert!(stdout.contains("main.rs"), "stdout was: {stdout}");
    assert!(!stdout.contains("lib.rs"), "stdout was: {stdout}");
    assert!(stdout.contains("docs/"), "stdout was: {stdout}");
    assert!(stdout.contains("guide.md"), "stdout was: {stdout}");
    assert!(!stdout.contains("README.md"), "stdout was: {stdout}");
}

#[test]
fn find_text_is_case_insensitive() {
    let root = temp_tree("find-text-ci");
    fs::write(root.path.join("HELLO.txt"), "Hello World").expect("failed to write file");
    fs::write(root.path.join("bye.txt"), "goodbye").expect("failed to write file");

    let output = oak()
        .args(["--no-config", "--no-color", "--no-icons", "--find-text", "hello", "-S", "name"])
        .arg(&root.path)
        .output()
        .expect("failed to run oak");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("HELLO.txt"), "stdout was: {stdout}");
    assert!(!stdout.contains("bye.txt"), "stdout was: {stdout}");
}
