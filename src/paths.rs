use anyhow::{Context, Result};
use std::path::PathBuf;

const APP_DIR_NAME: &str = "own-repos-curator";
const DATA_DIR_NAME: &str = "data";
const MANAGED_REPO_DIR_NAME: &str = "managed-repos";
const CONFIG_FILE_NAME: &str = "config.toml";
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
    let base = dirs::config_dir().context("failed to locate config directory")?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join(CONFIG_FILE_NAME))
}

pub fn hatena_url_cache_file_path() -> Result<PathBuf> {
    let base = dirs::data_local_dir().context("failed to locate LocalAppData directory")?;
    Ok(base
        .join(HATENA_SYNC_APP_DIR_NAME)
        .join(CACHE_DIR_NAME)
        .join(URL_CACHE_FILE_NAME))
}
