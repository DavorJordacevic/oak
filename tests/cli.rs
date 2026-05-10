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
    assert!(stdout.contains("├── a/"), "stdout was: {stdout}");
    assert!(stdout.contains("└── b/"), "stdout was: {stdout}");
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
