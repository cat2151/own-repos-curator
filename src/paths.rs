use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

const APP_DIR_NAME: &str = "own-repos-curator";
const DATA_DIR_NAME: &str = "data";
const MANAGED_REPO_DIR_NAME: &str = "managed-repos";
const CONFIG_FILE_NAME: &str = "config.toml";
const HISTORY_FILE_NAME: &str = "history.json";
const HATENA_SYNC_APP_DIR_NAME: &str = "own-repos-curator-to-hatena";
const CACHE_DIR_NAME: &str = "cache";
const URL_CACHE_FILE_NAME: &str = "url.json";

pub fn app_local_dir_path() -> Result<PathBuf> {
    let base = dirs::data_local_dir().context("failed to locate LocalAppData directory")?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn data_dir_path() -> Result<PathBuf> {
    Ok(app_local_dir_path()?.join(DATA_DIR_NAME))
}

pub fn managed_repo_dir_path() -> Result<PathBuf> {
    Ok(app_local_dir_path()?.join(MANAGED_REPO_DIR_NAME))
}

pub fn config_dir_path() -> Result<PathBuf> {
    app_local_dir_path()
}

pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join(CONFIG_FILE_NAME))
}

pub fn history_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join(HISTORY_FILE_NAME))
}

pub fn migrate_legacy_roaming_config_files_to_local() -> Result<()> {
    let Some(legacy_dir) = dirs::config_dir().map(|base| base.join(APP_DIR_NAME)) else {
        return Ok(());
    };
    let local_dir = config_dir_path()?;

    if legacy_dir == local_dir {
        return Ok(());
    }

    migrate_legacy_roaming_file_to_local(&legacy_dir, &local_dir, CONFIG_FILE_NAME)?;
    migrate_legacy_roaming_file_to_local(&legacy_dir, &local_dir, HISTORY_FILE_NAME)?;
    Ok(())
}

fn migrate_legacy_roaming_file_to_local(
    legacy_dir: &Path,
    local_dir: &Path,
    file_name: &str,
) -> Result<()> {
    let legacy_path = legacy_dir.join(file_name);
    let local_path = local_dir.join(file_name);

    if local_path.exists() || !legacy_path.is_file() {
        return Ok(());
    }

    fs::create_dir_all(local_dir).with_context(|| {
        format!(
            "failed to create local config directory: {}",
            local_dir.display()
        )
    })?;
    fs::copy(&legacy_path, &local_path).with_context(|| {
        format!(
            "failed to migrate legacy roaming config file from {} to {}",
            legacy_path.display(),
            local_path.display()
        )
    })?;
    Ok(())
}

pub fn hatena_url_cache_file_path() -> Result<PathBuf> {
    let base = dirs::data_local_dir().context("failed to locate LocalAppData directory")?;
    Ok(base
        .join(HATENA_SYNC_APP_DIR_NAME)
        .join(CACHE_DIR_NAME)
        .join(URL_CACHE_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::{
        app_local_dir_path, config_dir_path, config_file_path, history_file_path,
        migrate_legacy_roaming_file_to_local,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_PATH_SEQ: AtomicU64 = AtomicU64::new(0);

    struct TestRoot {
        path: PathBuf,
    }

    impl TestRoot {
        fn new(prefix: &str) -> Self {
            let unique = TEST_PATH_SEQ.fetch_add(1, Ordering::Relaxed);
            Self {
                path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(".test-local-data")
                    .join(format!("{prefix}-{unique}")),
            }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn config_paths_use_local_app_dir() {
        let local_dir = app_local_dir_path().expect("local app dir should resolve");

        assert_eq!(
            config_dir_path().expect("config dir should resolve"),
            local_dir
        );
        assert_eq!(
            config_file_path().expect("config file should resolve"),
            local_dir.join("config.toml")
        );
        assert_eq!(
            history_file_path().expect("history file should resolve"),
            local_dir.join("history.json")
        );
    }

    #[test]
    fn migrates_legacy_roaming_file_when_local_file_is_missing() {
        let root = TestRoot::new("path-migration");
        let legacy_dir = root.path().join("roaming");
        let local_dir = root.path().join("local");
        fs::create_dir_all(&legacy_dir).expect("legacy dir should be created");
        fs::write(legacy_dir.join("config.toml"), "legacy").expect("legacy file should be written");

        migrate_legacy_roaming_file_to_local(&legacy_dir, &local_dir, "config.toml")
            .expect("legacy file should migrate");

        assert_eq!(
            fs::read_to_string(local_dir.join("config.toml")).expect("local file should exist"),
            "legacy"
        );
        assert_eq!(
            fs::read_to_string(legacy_dir.join("config.toml")).expect("legacy file should remain"),
            "legacy"
        );
    }

    #[test]
    fn migration_keeps_existing_local_file() {
        let root = TestRoot::new("path-migration-existing");
        let legacy_dir = root.path().join("roaming");
        let local_dir = root.path().join("local");
        fs::create_dir_all(&legacy_dir).expect("legacy dir should be created");
        fs::create_dir_all(&local_dir).expect("local dir should be created");
        fs::write(legacy_dir.join("history.json"), "legacy")
            .expect("legacy file should be written");
        fs::write(local_dir.join("history.json"), "local").expect("local file should be written");

        migrate_legacy_roaming_file_to_local(&legacy_dir, &local_dir, "history.json")
            .expect("migration should skip existing local file");

        assert_eq!(
            fs::read_to_string(local_dir.join("history.json")).expect("local file should remain"),
            "local"
        );
    }
}
