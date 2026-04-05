use crate::{
    local_json_sync::maybe_copy_json_to_local_data_repo, paths::data_dir_path,
    repo_url_cache::RepoUrlCache,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const DATA_FILE_NAME: &str = "repos.json";
pub const DEFAULT_GROUP_NAME: &str = "ungrouped";

fn default_group_name() -> String {
    DEFAULT_GROUP_NAME.to_string()
}

fn default_registered_groups() -> Vec<String> {
    vec![default_group_name()]
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    #[serde(default)]
    pub github_desc_updated_at: String,
    #[serde(default)]
    pub last_json_commit_push_date: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    #[serde(default)]
    pub url: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    pub github_desc: String,
    pub desc_short: String,
    pub desc_long: String,
    #[serde(default = "default_group_name")]
    pub group: String,
    pub tags: Vec<String>,
}

impl Repo {
    pub fn updated_at_or_created(&self) -> &DateTime<Utc> {
        self.updated_at.as_ref().unwrap_or(&self.created_at)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoData {
    pub meta: Meta,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registered_tags: Vec<String>,
    #[serde(default = "default_registered_groups")]
    pub registered_groups: Vec<String>,
    pub repos: Vec<Repo>,
}

impl RepoData {
    pub fn load_or_init() -> Result<(Self, PathBuf)> {
        let path = Self::data_file_path()?;

        if path.exists() {
            return Ok((Self::read_from_path(&path)?, path));
        }

        let data = Self::empty();
        data.write_to_path(&path)?;
        Ok((data, path))
    }

    pub fn data_file_path() -> Result<PathBuf> {
        Ok(data_dir_path()?.join(DATA_FILE_NAME))
    }

    pub fn read_from_path(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read repo data: {}", path.display()))?;
        let mut data: Self = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse repo data: {}", path.display()))?;
        data.normalize();
        Ok(data)
    }

    pub fn write_to_path(&self, path: &Path) -> Result<()> {
        let parent = path
            .parent()
            .with_context(|| format!("failed to resolve parent directory: {}", path.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create data directory: {}", parent.display()))?;

        let json = self.to_pretty_json()?;
        fs::write(path, &json)
            .with_context(|| format!("failed to write repo data: {}", path.display()))?;
        maybe_copy_json_to_local_data_repo(path, &json)?;
        Ok(())
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        let cache = RepoUrlCache::load_or_empty();
        let normalized = self.prepared_for_serialization(&cache);
        serde_json::to_string_pretty(&normalized).context("failed to serialize repo data")
    }

    pub fn sort_repos(&mut self) {
        self.repos
            .sort_by(|left, right| right.created_at.cmp(&left.created_at));
    }

    pub fn empty() -> Self {
        RepoData {
            meta: Meta::default(),
            registered_tags: Vec::new(),
            registered_groups: default_registered_groups(),
            repos: Vec::new(),
        }
    }

    pub fn ensure_registered_group(&mut self, group: &str) {
        let group = normalized_group_name(group);
        if self
            .registered_groups
            .iter()
            .any(|registered| registered == &group)
        {
            return;
        }
        self.registered_groups.push(group);
        normalize_string_list(&mut self.registered_groups);
    }

    fn normalize(&mut self) {
        normalize_string_list(&mut self.registered_tags);
        normalize_string_list(&mut self.registered_groups);
        self.meta.owner = self.meta.owner.trim().to_string();

        if self.registered_groups.is_empty() {
            self.registered_groups.push(default_group_name());
        }

        for repo in &mut self.repos {
            repo.url = repo.url.trim().to_string();
            repo.group = normalized_group_name(&repo.group);
            normalize_string_list(&mut repo.tags);
            if !self
                .registered_groups
                .iter()
                .any(|registered| registered == &repo.group)
            {
                self.registered_groups.push(repo.group.clone());
            }
        }

        normalize_string_list(&mut self.registered_groups);
        self.sort_repos();
    }

    fn prepared_for_serialization(&self, cache: &RepoUrlCache) -> Self {
        let mut normalized = self.clone();
        normalized.normalize();
        normalized.refresh_repo_urls_from_cache(cache);
        normalized
    }

    fn refresh_repo_urls_from_cache(&mut self, cache: &RepoUrlCache) {
        if self.meta.owner.is_empty() {
            if let Some(owner) = cache.infer_owner(self.repos.iter().map(|repo| repo.name.as_str()))
            {
                self.meta.owner = owner;
            }
        }

        let owner = if self.meta.owner.is_empty() {
            None
        } else {
            Some(self.meta.owner.clone())
        };

        for repo in &mut self.repos {
            if let Some(url) = cache.resolve(owner.as_deref(), &repo.name) {
                repo.url = url.to_string();
            }
        }
    }
}

fn normalized_group_name(group: &str) -> String {
    let trimmed = group.trim();
    if trimmed.is_empty() {
        default_group_name()
    } else {
        trimmed.to_string()
    }
}

fn normalize_string_list(values: &mut Vec<String>) {
    *values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
}

#[cfg(test)]
mod tests {
    use super::{Meta, Repo, RepoData, DEFAULT_GROUP_NAME};
    use crate::repo_url_cache::RepoUrlCache;

    #[test]
    fn missing_url_field_defaults_to_empty_string() {
        let data: RepoData = serde_json::from_str(
            r#"{
  "meta": {},
  "registered_tags": [],
  "registered_groups": ["ungrouped"],
  "repos": [
    {
      "name": "own-repos-curator",
      "created_at": "2026-04-01T00:00:00Z",
      "updated_at": "2026-04-02T00:00:00Z",
      "github_desc": "",
      "desc_short": "",
      "desc_long": "",
      "group": "ungrouped",
      "tags": []
    }
  ]
}"#,
        )
        .expect("old json should deserialize");

        assert_eq!(data.meta.owner, "");
        assert_eq!(data.repos[0].url, "");
    }

    #[test]
    fn prepared_for_serialization_adds_repo_url_from_cache() {
        let cache = RepoUrlCache::from_entries(&[(
            "cat2151/own-repos-curator",
            "https://github.com/cat2151/own-repos-curator/blob/HEAD/README.ja.md",
        )]);
        let data = RepoData {
            meta: Meta {
                github_desc_updated_at: String::new(),
                last_json_commit_push_date: String::new(),
                owner: String::new(),
            },
            registered_tags: Vec::new(),
            registered_groups: vec![DEFAULT_GROUP_NAME.to_string()],
            repos: vec![Repo {
                name: "own-repos-curator".to_string(),
                url: String::new(),
                created_at: "2026-04-01T00:00:00Z".parse().expect("valid datetime"),
                updated_at: Some("2026-04-02T00:00:00Z".parse().expect("valid datetime")),
                github_desc: String::new(),
                desc_short: String::new(),
                desc_long: String::new(),
                group: DEFAULT_GROUP_NAME.to_string(),
                tags: Vec::new(),
            }],
        };

        let normalized = data.prepared_for_serialization(&cache);

        assert_eq!(normalized.meta.owner, "cat2151");
        assert_eq!(
            normalized.repos[0].url,
            "https://github.com/cat2151/own-repos-curator/blob/HEAD/README.ja.md"
        );
    }
}
