use super::{App, AppHistory, DEBUG_LOG_LIMIT};
use crate::{github::sync_repo_data, paths::history_file_path};
use anyhow::Result;
use std::path::Path;

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
        self.persist_history_to_path(&history_file_path()?)
    }

    pub(crate) fn persist_history_to_path(&self, path: &Path) -> Result<()> {
        AppHistory {
            desc_display_mode: self.desc_display_mode,
        }
        .write_to_path(path)
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
    use crate::app::{
        tests::common::{app_with_registered_tags, repo, shift_key},
        DescDisplayMode,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestHistoryPath {
        path: PathBuf,
    }

    impl TestHistoryPath {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self {
                path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(".test-local-data")
                    .join(format!("history-{unique}"))
                    .join("history.json"),
            }
        }

        fn as_path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestHistoryPath {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
            if let Some(parent) = self.path.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
    }

    #[test]
    fn app_history_roundtrip_preserves_desc_display_mode() {
        let path = TestHistoryPath::new();
        AppHistory {
            desc_display_mode: DescDisplayMode::LeftShort,
        }
        .write_to_path(path.as_path())
        .expect("history should be written");

        let restored = AppHistory::load_from_path(path.as_path()).expect("history should load");
        let raw = fs::read_to_string(path.as_path()).expect("history file should exist");

        assert_eq!(restored.desc_display_mode, DescDisplayMode::LeftShort);
        assert!(raw.contains("\"desc_display_mode\": \"left_short\""));
    }

    #[test]
    fn persist_history_to_path_saves_current_desc_display_mode() {
        let path = TestHistoryPath::new();
        let mut app = app_with_registered_tags(
            vec![repo("solo", "2026-03-01T00:00:00Z", None)],
            vec!["rust".to_string()],
        );

        app.handle_key(shift_key('d'));
        app.persist_history_to_path(path.as_path())
            .expect("history should be persisted");

        let restored = AppHistory::load_from_path(path.as_path()).expect("history should load");

        assert_eq!(restored.desc_display_mode, DescDisplayMode::LeftShort);
    }
}
