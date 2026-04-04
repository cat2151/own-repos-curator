use super::*;

#[test]
fn build_commit_hash_is_not_empty() {
    assert!(!build_commit_hash().is_empty());
}

#[test]
fn owner_repo_uses_expected_slug() {
    assert_eq!(owner_repo(), "cat2151/own-repos-curator");
}

#[test]
fn install_command_contains_install_git_url() {
    let cmd = install_cmd();
    assert!(cmd.contains("cargo install --force --git"));
    assert!(cmd.contains("own-repos-curator"));
}

#[test]
fn install_command_targets_repository_url() {
    assert_eq!(
        install_cmd(),
        "cargo install --force --git https://github.com/cat2151/own-repos-curator"
    );
}

#[test]
fn update_relaunches_repocurator_binary() {
    assert_eq!(BIN_NAMES, &["repocurator"]);
}

#[test]
fn py_script_contains_repo_url() {
    let script = generate_py_script(REPO_OWNER, REPO_NAME, BIN_NAMES);
    assert!(script.contains("https://github.com/cat2151/own-repos-curator"));
}

#[test]
fn py_script_contains_cargo_install() {
    let script = generate_py_script(REPO_OWNER, REPO_NAME, BIN_NAMES);
    assert!(script.contains("cargo"));
    assert!(script.contains("install"));
    assert!(script.contains("--force"));
    assert!(script.contains("--git"));
}

#[test]
fn py_script_launches_repocurator_binary() {
    let script = generate_py_script(REPO_OWNER, REPO_NAME, BIN_NAMES);
    assert!(script.contains("'repocurator'"));
}

#[test]
fn py_script_escapes_single_quotes() {
    assert_eq!(escape_py_single_quoted("a'b"), "a\\'b");
    assert_eq!(escape_py_single_quoted("a\\b"), "a\\\\b");
}

#[test]
fn unique_tmp_path_has_expected_prefix() {
    let path = unique_tmp_path();
    let name = path.file_name().and_then(|name| name.to_str()).unwrap();
    assert!(name.starts_with("cat_self_update_"));
    assert!(name.ends_with(".py"));
}
