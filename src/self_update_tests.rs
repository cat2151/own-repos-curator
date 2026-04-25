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
fn main_branch_uses_main() {
    assert_eq!(MAIN_BRANCH, "main");
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
fn update_uses_default_cargo_install_crate_selection() {
    assert_eq!(INSTALL_CRATES, &[] as &[&str]);
}
