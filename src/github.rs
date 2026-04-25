use crate::{
    model::{Repo, RepoData, DEFAULT_GROUP_NAME},
    process::output_with_timeout,
};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::{process::Command, time::Duration};

const GH_COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy)]
pub struct SyncSummary {
    pub added: usize,
    pub updated: usize,
}

#[derive(Debug, Clone)]
pub struct FetchedRepo {
    pub(crate) name: String,
    pub(crate) created_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
    pub(crate) github_desc: String,
}

pub fn fetch_remote_repos_with_progress(
    mut progress: impl FnMut(&str),
) -> Result<Vec<FetchedRepo>> {
    progress("GitHub CLIのログインユーザーを確認中 (`gh api user`)");
    let owner = current_login()?;
    progress(&format!("public repo一覧を取得中 (`gh repo list {owner}`)"));
    fetch_repos(&owner)
}

pub fn apply_fetched_repos(data: &mut RepoData, fetched: Vec<FetchedRepo>) -> SyncSummary {
    let mut added = 0;
    let mut updated = 0;

    for repo in fetched {
        if let Some(existing) = data.repos.iter_mut().find(|item| item.name == repo.name) {
            let mut changed = false;
            if existing.created_at != repo.created_at {
                existing.created_at = repo.created_at;
                changed = true;
            }
            if existing.updated_at != Some(repo.updated_at) {
                existing.updated_at = Some(repo.updated_at);
                changed = true;
            }
            if existing.github_desc != repo.github_desc {
                existing.github_desc = repo.github_desc;
                changed = true;
            }
            if changed {
                updated += 1;
            }
            continue;
        }

        data.repos.push(Repo {
            name: repo.name,
            url: String::new(),
            created_at: repo.created_at,
            updated_at: Some(repo.updated_at),
            github_desc: repo.github_desc,
            desc_short: String::new(),
            desc_long: String::new(),
            group: DEFAULT_GROUP_NAME.to_string(),
            tags: Vec::new(),
        });
        data.ensure_registered_group(DEFAULT_GROUP_NAME);
        added += 1;
    }

    data.meta.github_desc_updated_at = Utc::now().date_naive().to_string();
    data.sort_repos();

    SyncSummary { added, updated }
}

fn current_login() -> Result<String> {
    let mut command = Command::new("gh");
    command.args(["api", "user", "--jq", ".login"]);
    let output = output_with_timeout(&mut command, GH_COMMAND_TIMEOUT, "gh api user")?;

    if !output.status.success() {
        return Err(anyhow!(
            "gh api user failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let login = String::from_utf8(output.stdout)
        .context("failed to decode GitHub login output")?
        .trim()
        .to_string();

    if login.is_empty() {
        return Err(anyhow!("gh api user returned an empty login"));
    }

    Ok(login)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepo {
    name: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    description: Option<String>,
}

fn fetch_repos(owner: &str) -> Result<Vec<FetchedRepo>> {
    let mut command = Command::new("gh");
    command.args([
        "repo",
        "list",
        owner,
        "--source",
        "--visibility",
        "public",
        "--no-archived",
        "--limit",
        "1000",
        "--json",
        "name,createdAt,updatedAt,description",
    ]);
    let output = output_with_timeout(
        &mut command,
        GH_COMMAND_TIMEOUT,
        &format!("gh repo list for {owner}"),
    )?;

    if !output.status.success() {
        return Err(anyhow!(
            "gh repo list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let repos: Vec<GithubRepo> =
        serde_json::from_slice(&output.stdout).context("failed to parse gh repo list JSON")?;
    Ok(repos
        .into_iter()
        .map(|repo| FetchedRepo {
            name: repo.name,
            created_at: repo.created_at,
            updated_at: repo.updated_at,
            github_desc: repo.description.unwrap_or_default(),
        })
        .collect())
}
