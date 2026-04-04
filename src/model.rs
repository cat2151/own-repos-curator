use crate::paths::data_dir_path;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const DATA_FILE_NAME: &str = "repos.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    #[serde(default)]
    pub github_desc_updated_at: String,
    #[serde(default)]
    pub last_json_commit_push_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    pub github_desc: String,
    pub desc_short: String,
    pub desc_long: String,
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
        data.sort_repos();
        Ok(data)
    }

    pub fn write_to_path(&self, path: &Path) -> Result<()> {
        let parent = path
            .parent()
            .with_context(|| format!("failed to resolve parent directory: {}", path.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create data directory: {}", parent.display()))?;

        let json = serde_json::to_string_pretty(self).context("failed to serialize repo data")?;
        fs::write(path, json)
            .with_context(|| format!("failed to write repo data: {}", path.display()))?;
        Ok(())
    }

    pub fn sort_repos(&mut self) {
        self.repos
            .sort_by(|left, right| right.created_at.cmp(&left.created_at));
    }

    pub fn empty() -> Self {
        RepoData {
            meta: Meta::default(),
            registered_tags: Vec::new(),
            repos: Vec::new(),
        }
    }
}
