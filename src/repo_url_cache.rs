use crate::paths::hatena_url_cache_file_path;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
};

#[derive(Debug, Default)]
pub(crate) struct RepoUrlCache {
    entries: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RepoUrlCacheFile {
    #[serde(default)]
    entries: BTreeMap<String, String>,
}

impl RepoUrlCache {
    pub(crate) fn load_or_empty() -> Self {
        match Self::load() {
            Ok(cache) => cache,
            Err(err) => {
                eprintln!("[repo-url-cache] disabled: {err}");
                Self::default()
            }
        }
    }

    pub(crate) fn resolve(&self, owner: Option<&str>, repo_name: &str) -> Option<&str> {
        let repo_name = repo_name.trim();
        if repo_name.is_empty() {
            return None;
        }

        if let Some(owner) = owner.map(str::trim).filter(|owner| !owner.is_empty()) {
            let key = format!("{owner}/{repo_name}");
            if let Some(url) = self.entries.get(&key) {
                return Some(url.as_str());
            }
        }

        let suffix = format!("/{repo_name}");
        let mut matches = self
            .entries
            .iter()
            .filter(|(key, _)| key.ends_with(&suffix))
            .map(|(_, url)| url.as_str());
        let url = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some(url)
    }

    pub(crate) fn infer_owner<'a>(
        &self,
        repo_names: impl IntoIterator<Item = &'a str>,
    ) -> Option<String> {
        let repo_names = repo_names
            .into_iter()
            .map(str::trim)
            .filter(|repo_name| !repo_name.is_empty())
            .collect::<HashSet<_>>();
        if repo_names.is_empty() {
            return None;
        }

        let mut counts = HashMap::<&str, usize>::new();
        for key in self.entries.keys() {
            let Some((owner, repo_name)) = key.split_once('/') else {
                continue;
            };
            if repo_names.contains(repo_name) {
                *counts.entry(owner).or_insert(0) += 1;
            }
        }

        let mut ranked = counts.into_iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));

        let (owner, count) = ranked.first()?;
        if ranked
            .get(1)
            .is_some_and(|(_, next_count)| next_count == count)
        {
            return None;
        }

        Some((*owner).to_string())
    }

    fn load() -> Result<Self> {
        let path = hatena_url_cache_file_path()?;
        Self::load_from_path(&path)
    }

    fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read repo url cache: {}", path.display()))?;
        let file: RepoUrlCacheFile = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse repo url cache: {}", path.display()))?;
        Ok(Self {
            entries: file.entries,
        })
    }

    #[cfg(test)]
    pub(crate) fn from_entries(entries: &[(&str, &str)]) -> Self {
        Self {
            entries: entries
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RepoUrlCache;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn loads_entries_from_cache_file() {
        let fixture = TestFixture::new();
        let cache_path = fixture.root.join("url.json");
        fs::write(
            &cache_path,
            r#"{
  "updated_on": "2026-04-05",
  "entries": {
    "cat2151/own-repos-curator": "https://github.com/cat2151/own-repos-curator/blob/HEAD/README.ja.md"
  }
}"#,
        )
        .expect("cache file should be written");

        let cache = RepoUrlCache::load_from_path(&cache_path).expect("cache should load");

        assert_eq!(
            cache.resolve(Some("cat2151"), "own-repos-curator"),
            Some("https://github.com/cat2151/own-repos-curator/blob/HEAD/README.ja.md")
        );
    }

    #[test]
    fn infers_owner_from_highest_match_count() {
        let cache = RepoUrlCache::from_entries(&[
            ("cat2151/own-repos-curator", "https://example.com/a"),
            ("cat2151/cat-self-update", "https://example.com/b"),
            ("other/own-repos-curator", "https://example.com/c"),
        ]);

        assert_eq!(
            cache.infer_owner(["own-repos-curator", "cat-self-update"]),
            Some("cat2151".to_string())
        );
    }

    struct TestFixture {
        root: PathBuf,
    }

    impl TestFixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be monotonic")
                .as_nanos();
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".test-local-data")
                .join(format!("repo-url-cache-{unique}"));
            fs::create_dir_all(&root).expect("fixture root should be created");
            Self { root }
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
