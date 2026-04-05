use super::{App, AppHistory, DEBUG_LOG_LIMIT};
use crate::{github::sync_repo_data, paths::history_file_path};
use anyhow::Result;

impl App {
    pub(crate) fn refresh_from_github(&mut self) {
        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
        match sync_repo_data(&mut self.data).and_then(|summary| {
            self.data.write_to_path(&self.data_path)?;
            Ok(summary)
        }) {
            Ok(summary) => {
                self.restore_selection(selected_repo_name.as_deref());
                self.status_message = format!(
                    "GitHub同期完了: {}件追加 / {}件更新",
                    summary.added, summary.updated
                );
            }
            Err(error) => {
                self.status_message = format!("GitHub同期失敗: {error}");
            }
        }
    }

    pub(crate) fn persist_data(&self) -> Result<()> {
        self.data.write_to_path(&self.data_path)
    }

    pub(crate) fn persist_history(&self) -> Result<()> {
        AppHistory {
            desc_display_mode: self.desc_display_mode,
        }
        .write_to_path(&history_file_path()?)
    }

    pub(crate) fn push_debug_log(&mut self, message: impl Into<String>) {
        let entry = format!("#{:04} {}", self.debug_log_seq, message.into());
        self.debug_log_seq += 1;
        if self.debug_log.len() >= DEBUG_LOG_LIMIT {
            self.debug_log.pop_front();
        }
        self.debug_log.push_back(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::AppHistory;
    use crate::app::DescDisplayMode;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_history_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".test-local-data")
            .join(format!("history-{unique}"))
            .join("history.json")
    }

    #[test]
    fn app_history_roundtrip_preserves_desc_display_mode() {
        let path = unique_history_path();
        AppHistory {
            desc_display_mode: DescDisplayMode::LeftShort,
        }
        .write_to_path(&path)
        .expect("history should be written");

        let restored = AppHistory::load_or_default_from_path(&path);
        let raw = fs::read_to_string(&path).expect("history file should exist");

        assert_eq!(restored.desc_display_mode, DescDisplayMode::LeftShort);
        assert!(raw.contains("\"desc_display_mode\": \"left_short\""));

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
}
