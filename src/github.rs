use crate::model::{Repo, RepoData, DEFAULT_GROUP_NAME};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub struct SyncSummary {
    pub added: usize,
    pub updated: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepo {
    name: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    description: Option<String>,
}

pub fn sync_repo_data(data: &mut RepoData) -> Result<SyncSummary> {
    let owner = current_login()?;
    let fetched = fetch_repos(&owner)?;

    let mut added = 0;
    let mut updated = 0;

    for repo in fetched {
        let github_desc = repo.description.unwrap_or_default();

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
            if existing.github_desc != github_desc {
                existing.github_desc = github_desc;
                changed = true;
            }
            if changed {
                updated += 1;
            }
            continue;
        }

        data.repos.push(Repo {
            name: repo.name,
            created_at: repo.created_at,
            updated_at: Some(repo.updated_at),
            github_desc,
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

    Ok(SyncSummary { added, updated })
}

fn current_login() -> Result<String> {
    let output = Command::new("gh")
        .args(["api", "user", "--jq", ".login"])
        .output()
        .context("failed to run gh api user")?;

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

fn fetch_repos(owner: &str) -> Result<Vec<GithubRepo>> {
    let output = Command::new("gh")
        .args([
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
        ])
        .output()
        .with_context(|| format!("failed to run gh repo list for {owner}"))?;

    if !output.status.success() {
        return Err(anyhow!(
            "gh repo list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let repos =
        serde_json::from_slice(&output.stdout).context("failed to parse gh repo list JSON")?;
    Ok(repos)
}
