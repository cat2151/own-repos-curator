use crate::paths::{config_file_path, migrate_legacy_roaming_config_files_to_local};
use anyhow::{anyhow, Context, Result};
use std::{fs, path::Path};

const JSON_AUTO_PUSH_SECTION: &str = "json_auto_push";
const REPO_KEY: &str = "repo";
const DEFAULT_CONFIG: &str = concat!(
    "# own-repos-curator configuration\n",
    "# Set the GitHub repository used for the daily repos.json backup.\n",
    "# Example: repo = \"your-account/your-backup-repo\"\n",
    "[json_auto_push]\n",
    "repo = \"\"\n",
);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppConfig {
    pub json_auto_push: JsonAutoPushConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonAutoPushConfig {
    pub repo: Option<GitHubRepoRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRepoRef(String);

impl AppConfig {
    pub fn load_or_init() -> Result<Self> {
        migrate_legacy_roaming_config_files_to_local()?;
        let path = config_file_path()?;
        Self::load_or_init_from_path(&path)
    }

    fn load_or_init_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            write_default_config(path)?;
        }

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        Self::parse(&raw).with_context(|| format!("failed to parse config: {}", path.display()))
    }

    fn parse(raw: &str) -> Result<Self> {
        let mut current_section = String::new();
        let mut repo = None;

        for (index, raw_line) in raw.lines().enumerate() {
            let line_number = index + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') {
                current_section = parse_section(line)
                    .with_context(|| format!("invalid section at line {line_number}"))?;
                continue;
            }

            let (key, value) = parse_key_value(line)
                .with_context(|| format!("invalid key/value at line {line_number}"))?;

            if current_section == JSON_AUTO_PUSH_SECTION && key == REPO_KEY {
                let parsed = parse_string_literal(value)
                    .with_context(|| format!("invalid repo value at line {line_number}"))?;
                repo = if parsed.trim().is_empty() {
                    None
                } else {
                    Some(GitHubRepoRef::parse(&parsed)?)
                };
            }
        }

        Ok(AppConfig {
            json_auto_push: JsonAutoPushConfig { repo },
        })
    }
}

impl GitHubRepoRef {
    pub fn parse(value: &str) -> Result<Self> {
        let trimmed = value.trim();
        let (owner, repo) = trimmed
            .split_once('/')
            .ok_or_else(|| anyhow!("GitHub repo must be in owner/repo format"))?;

        if !is_valid_repo_component(owner) || !is_valid_repo_component(repo) {
            return Err(anyhow!(
                "GitHub repo may only contain ASCII letters, digits, '.', '_' and '-'"
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn directory_name(&self) -> String {
        self.0.replace('/', "__")
    }
}

fn write_default_config(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("failed to resolve config parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create config directory: {}", parent.display()))?;
    fs::write(path, DEFAULT_CONFIG)
        .with_context(|| format!("failed to write default config: {}", path.display()))?;
    Ok(())
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_section(line: &str) -> Result<String> {
    if !line.ends_with(']') || !line.starts_with('[') {
        return Err(anyhow!("section header must be wrapped with []"));
    }

    let section = line[1..line.len() - 1].trim();
    if section.is_empty() {
        return Err(anyhow!("section name cannot be empty"));
    }

    Ok(section.to_string())
}

fn parse_key_value(line: &str) -> Result<(&str, &str)> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| anyhow!("expected '=' in key/value pair"))?;
    let key = key.trim();
    let value = value.trim();

    if key.is_empty() {
        return Err(anyhow!("key cannot be empty"));
    }

    Ok((key, value))
}

fn parse_string_literal(value: &str) -> Result<String> {
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err(anyhow!("expected a double-quoted string"));
    }

    let inner = &value[1..value.len() - 1];
    if inner.contains('\\') {
        return Err(anyhow!("escape sequences are not supported in config.toml"));
    }

    Ok(inner.to_string())
}

fn is_valid_repo_component(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::{write_default_config, AppConfig, GitHubRepoRef};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".test-local-data")
            .join(format!("config-{unique}"))
            .join("config.toml")
    }

    #[test]
    fn parses_json_auto_push_repo() {
        let config = AppConfig::parse(
            r#"
                [json_auto_push]
                repo = "cat2151/backup-repo"
            "#,
        )
        .expect("config should parse");

        assert_eq!(
            config
                .json_auto_push
                .repo
                .as_ref()
                .map(GitHubRepoRef::as_str),
            Some("cat2151/backup-repo")
        );
    }

    #[test]
    fn empty_repo_disables_json_auto_push() {
        let config = AppConfig::parse(
            r#"
                [json_auto_push]
                repo = ""
            "#,
        )
        .expect("config should parse");

        assert!(config.json_auto_push.repo.is_none());
    }

    #[test]
    fn invalid_repo_is_rejected() {
        let error = AppConfig::parse(
            r#"
                [json_auto_push]
                repo = "../danger"
            "#,
        )
        .expect_err("config should reject invalid repo");

        assert!(error.to_string().contains("GitHub repo"));
    }

    #[test]
    fn load_or_init_from_path_creates_default_config() {
        let path = unique_path();

        let config = AppConfig::load_or_init_from_path(&path).expect("config should initialize");
        let raw = fs::read_to_string(&path).expect("default config should exist");

        assert_eq!(config, AppConfig::default());
        assert!(raw.contains("[json_auto_push]"));

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }

    #[test]
    fn write_default_config_creates_parent_directory() {
        let path = unique_path();

        write_default_config(&path).expect("default config should be written");

        assert!(path.exists());

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
}
