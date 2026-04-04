use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(windows)]
use std::process::Stdio;

pub(crate) const REPO_OWNER: &str = "cat2151";
pub(crate) const REPO_NAME: &str = "own-repos-curator";
const BIN_NAMES: &[&str] = &["repocurator"];

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

fn launch_self_update(owner: &str, repo: &str, bins: &[&str]) -> anyhow::Result<()> {
    let py_content = generate_py_script(owner, repo, bins);
    let py_path = unique_tmp_path();

    fs::write(&py_path, &py_content)?;
    spawn_python(&py_path)?;
    Ok(())
}

fn escape_py_single_quoted(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            _ => out.push(ch),
        }
    }
    out
}

fn generate_py_script(owner: &str, repo: &str, bins: &[&str]) -> String {
    let repo_url = format!("https://github.com/{owner}/{repo}");
    let repo_url_escaped = escape_py_single_quoted(&repo_url);
    let install_parts = format!(
        "['cargo', 'install', '--force', '--git', '{}']",
        repo_url_escaped
    );

    let launch_stmts: String = if bins.is_empty() {
        let repo_escaped = escape_py_single_quoted(repo);
        format!("subprocess.Popen(['{}'], **popen_kwargs)\n", repo_escaped)
    } else {
        bins.iter()
            .map(|bin| {
                let bin_escaped = escape_py_single_quoted(bin);
                format!("subprocess.Popen(['{}'], **popen_kwargs)\n", bin_escaped)
            })
            .collect()
    };

    format!(
        r#"import subprocess
import os
import sys

if sys.platform == 'win32':
    DETACHED_PROCESS = 0x00000008
    popen_kwargs = {{
        'creationflags': DETACHED_PROCESS,
        'stdin': subprocess.DEVNULL,
        'stdout': subprocess.DEVNULL,
        'stderr': subprocess.DEVNULL,
    }}
else:
    popen_kwargs = {{}}

subprocess.run({install_parts}, check=True, **popen_kwargs)

{launch_stmts}
try:
    os.remove(__file__)
except OSError:
    pass
"#,
        install_parts = install_parts,
        launch_stmts = launch_stmts,
    )
}

fn unique_tmp_path() -> PathBuf {
    let pid = std::process::id();
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let filename = format!("cat_self_update_{}_{}.py", pid, timestamp_nanos);
    std::env::temp_dir().join(filename)
}

fn spawn_python(py_path: &Path) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const DETACHED_PROCESS: u32 = 0x0000_0008;
        Command::new("python")
            .arg(py_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(DETACHED_PROCESS)
            .spawn()?;
    }

    #[cfg(not(windows))]
    {
        Command::new("python3").arg(py_path).spawn()?;
    }

    Ok(())
}

pub fn run_self_update() -> anyhow::Result<bool> {
    launch_self_update(REPO_OWNER, REPO_NAME, BIN_NAMES)
        .map_err(|err| anyhow::anyhow!("failed to launch self-update helper: {err}"))?;
    println!("Running: {}", install_cmd());
    println!("The application will now exit so the updater can replace the binary.");
    Ok(true)
}

#[cfg(test)]
#[path = "self_update_tests.rs"]
mod tests;
