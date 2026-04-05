use crate::{
    config::{AppConfig, GitHubRepoRef},
    data_link::spawn_hatena_sync,
    model::RepoData,
    paths::managed_repo_dir_path,
};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SNAPSHOT_FILE_NAME: &str = "repos.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoPushOutcome {
    Disabled,
    SkippedToday {
        date: String,
    },
    UpToDate {
        repo: String,
        date: String,
    },
    Pushed {
        repo: String,
        date: String,
        commit_id: String,
    },
}

pub fn maybe_push_json_snapshot(
    data: &mut RepoData,
    config: &AppConfig,
) -> Result<AutoPushOutcome> {
    let Some(repo) = config.json_auto_push.repo.as_ref() else {
        return Ok(AutoPushOutcome::Disabled);
    };

    let today = Utc::now().date_naive().to_string();
    if data.meta.last_json_commit_push_date == today {
        return Ok(AutoPushOutcome::SkippedToday { date: today });
    }

    let mut snapshot = data.clone();
    snapshot.meta.last_json_commit_push_date = today.clone();

    let outcome = push_snapshot(&snapshot, repo, &today)?;
    data.meta.last_json_commit_push_date = today;
    Ok(outcome)
}

fn push_snapshot(
    snapshot: &RepoData,
    repo: &GitHubRepoRef,
    today: &str,
) -> Result<AutoPushOutcome> {
    ensure_gh_auth()?;
    let repo_dir = ensure_managed_clone(repo)?;
    ensure_clean_worktree(&repo_dir)?;
    git(&repo_dir, ["pull", "--ff-only"])?;
    ensure_clean_worktree(&repo_dir)?;

    let snapshot_path = write_snapshot(&repo_dir, snapshot)?;
    if !path_has_changes(&repo_dir, &snapshot_path)? {
        return Ok(AutoPushOutcome::UpToDate {
            repo: repo.as_str().to_string(),
            date: today.to_string(),
        });
    }

    let snapshot_file = snapshot_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("failed to resolve snapshot file name"))?;
    git(&repo_dir, ["add", "--", snapshot_file])?;
    ensure_only_snapshot_is_staged(&repo_dir, snapshot_file)?;

    let message = format!("chore: update {SNAPSHOT_FILE_NAME} ({today})");
    spawn_hatena_sync()?;
    git(&repo_dir, ["commit", "-m", &message])?;
    git(&repo_dir, ["push"])?;
    let commit_id = git_stdout(&repo_dir, ["rev-parse", "HEAD"])?;

    Ok(AutoPushOutcome::Pushed {
        repo: repo.as_str().to_string(),
        date: today.to_string(),
        commit_id,
    })
}

fn ensure_gh_auth() -> Result<()> {
    let output = Command::new("gh")
        .args(["auth", "status"])
        .output()
        .context("failed to run `gh auth status`")?;
    ensure_success("gh auth status", &output)?;
    Ok(())
}

fn ensure_managed_clone(repo: &GitHubRepoRef) -> Result<PathBuf> {
    let base_dir = managed_repo_dir_path()?;
    fs::create_dir_all(&base_dir).with_context(|| {
        format!(
            "failed to create managed repo directory: {}",
            base_dir.display()
        )
    })?;

    let repo_dir = base_dir.join(repo.directory_name());
    if !repo_dir.exists() {
        let clone_target = repo_dir.to_string_lossy().to_string();
        let output = Command::new("gh")
            .args(["repo", "clone", repo.as_str(), &clone_target])
            .output()
            .with_context(|| format!("failed to clone backup repo {}", repo.as_str()))?;
        ensure_success("gh repo clone", &output)?;
    }

    if !repo_dir.join(".git").exists() {
        return Err(anyhow!(
            "managed backup repo path is not a git clone: {}",
            repo_dir.display()
        ));
    }

    ensure_remote_origin_matches(&repo_dir, repo)?;
    Ok(repo_dir)
}

fn ensure_remote_origin_matches(repo_dir: &Path, repo: &GitHubRepoRef) -> Result<()> {
    let origin = git_stdout(repo_dir, ["config", "--get", "remote.origin.url"])?;
    let normalized_origin = normalize_origin_url(&origin)
        .ok_or_else(|| anyhow!("unsupported origin URL format: {origin}"))?;

    if normalized_origin != repo.as_str() {
        return Err(anyhow!(
            "managed backup repo origin mismatch: expected {} but found {}",
            repo.as_str(),
            origin
        ));
    }

    Ok(())
}

fn ensure_clean_worktree(repo_dir: &Path) -> Result<()> {
    let status = git_stdout(repo_dir, ["status", "--porcelain"])?;
    if status.trim().is_empty() {
        return Ok(());
    }

    Err(anyhow!(
        "managed backup repo has local changes; refusing automatic push"
    ))
}

fn write_snapshot(repo_dir: &Path, snapshot: &RepoData) -> Result<PathBuf> {
    let snapshot_path = repo_dir.join(SNAPSHOT_FILE_NAME);
    let json =
        serde_json::to_string_pretty(snapshot).context("failed to serialize JSON snapshot")?;
    fs::write(&snapshot_path, json)
        .with_context(|| format!("failed to write snapshot: {}", snapshot_path.display()))?;
    Ok(snapshot_path)
}

fn path_has_changes(repo_dir: &Path, path: &Path) -> Result<bool> {
    let path_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("failed to resolve changed path name"))?;
    let status = git_stdout(repo_dir, ["status", "--porcelain", "--", path_name])?;
    Ok(!status.trim().is_empty())
}

fn ensure_only_snapshot_is_staged(repo_dir: &Path, snapshot_file: &str) -> Result<()> {
    let staged = git_stdout(repo_dir, ["diff", "--cached", "--name-only"])?;
    let staged_files = staged
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if staged_files == [snapshot_file] {
        return Ok(());
    }

    Err(anyhow!(
        "unexpected staged files in managed backup repo: {}",
        staged_files.join(", ")
    ))
}

fn git<const N: usize>(repo_dir: &Path, args: [&str; N]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .with_context(|| format!("failed to run git command in {}", repo_dir.display()))?;
    ensure_success(&format!("git {}", args.join(" ")), &output)
}

fn git_stdout<const N: usize>(repo_dir: &Path, args: [&str; N]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .with_context(|| format!("failed to run git command in {}", repo_dir.display()))?;
    ensure_success(&format!("git {}", args.join(" ")), &output)?;
    String::from_utf8(output.stdout)
        .map(|stdout| stdout.trim().to_string())
        .context("failed to decode git stdout")
}

fn ensure_success(command: &str, output: &Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        return Err(anyhow!("{command} failed with status {}", output.status));
    }

    Err(anyhow!("{command} failed: {stderr}"))
}

fn normalize_origin_url(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    let without_git_suffix = trimmed.strip_suffix(".git").unwrap_or(trimmed);

    for prefix in [
        "https://github.com/",
        "http://github.com/",
        "git@github.com:",
        "ssh://git@github.com/",
    ] {
        if let Some(repo) = without_git_suffix.strip_prefix(prefix) {
            return Some(repo.to_string());
        }
    }

    if let Ok(repo) = GitHubRepoRef::parse(without_git_suffix) {
        return Some(repo.as_str().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{normalize_origin_url, AutoPushOutcome};
    use crate::{
        config::{AppConfig, GitHubRepoRef, JsonAutoPushConfig},
        model::{Meta, RepoData},
    };

    #[test]
    fn skips_when_today_was_already_pushed() {
        let today = chrono::Utc::now().date_naive().to_string();
        let mut data = RepoData {
            meta: Meta {
                github_desc_updated_at: String::new(),
                last_json_commit_push_date: today.clone(),
            },
            registered_tags: Vec::new(),
            registered_groups: vec!["ungrouped".to_string()],
            repos: Vec::new(),
        };
        let config = AppConfig {
            json_auto_push: JsonAutoPushConfig {
                repo: Some(GitHubRepoRef::parse("cat2151/backups").expect("valid repo")),
            },
        };

        let outcome =
            super::maybe_push_json_snapshot(&mut data, &config).expect("push should skip");

        assert_eq!(outcome, AutoPushOutcome::SkippedToday { date: today });
    }

    #[test]
    fn normalizes_https_and_ssh_origins() {
        assert_eq!(
            normalize_origin_url("https://github.com/cat2151/backups.git"),
            Some("cat2151/backups".to_string())
        );
        assert_eq!(
            normalize_origin_url("git@github.com:cat2151/backups.git"),
            Some("cat2151/backups".to_string())
        );
        assert_eq!(
            normalize_origin_url("ssh://git@github.com/cat2151/backups.git"),
            Some("cat2151/backups".to_string())
        );
    }

    #[test]
    fn normalize_origin_url_rejects_non_repo_values() {
        assert_eq!(normalize_origin_url("github"), None);
    }
}
