use cat_self_update_lib::{check_remote_commit, self_update as launch_self_update, CheckResult};
use std::sync::OnceLock;

pub(crate) const REPO_OWNER: &str = "cat2151";
pub(crate) const REPO_NAME: &str = "own-repos-curator";
const MAIN_BRANCH: &str = "main";
const INSTALL_CRATES: &[&str] = &[];

pub(crate) fn build_commit_hash() -> &'static str {
    env!("BUILD_COMMIT_HASH")
}

pub(crate) fn install_cmd() -> String {
    format!("cargo install --force --git {}", git_url())
}

pub(crate) fn owner_repo() -> &'static str {
    static OWNER_REPO: OnceLock<String> = OnceLock::new();
    OWNER_REPO
        .get_or_init(|| format!("{REPO_OWNER}/{REPO_NAME}"))
        .as_str()
}

fn git_url() -> &'static str {
    static GIT_URL: OnceLock<String> = OnceLock::new();
    GIT_URL
        .get_or_init(|| format!("https://github.com/{}", owner_repo()))
        .as_str()
}

pub fn run_self_update() -> anyhow::Result<bool> {
    launch_self_update(REPO_OWNER, REPO_NAME, INSTALL_CRATES)
        .map_err(|err| anyhow::anyhow!("failed to launch self-update helper: {err}"))?;
    println!("Running: {}", install_cmd());
    println!("The application will now exit so the updater can replace the binary.");
    Ok(true)
}

pub fn check_self_update() -> anyhow::Result<CheckResult> {
    check_remote_commit(REPO_OWNER, REPO_NAME, MAIN_BRANCH, build_commit_hash())
        .map_err(|err| anyhow::anyhow!("failed to check self-update status: {err}"))
}

#[cfg(test)]
#[path = "self_update_tests.rs"]
mod tests;
