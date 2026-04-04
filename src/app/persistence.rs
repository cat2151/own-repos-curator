use super::{App, DEBUG_LOG_LIMIT};
use crate::github::sync_repo_data;
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

    pub(crate) fn push_debug_log(&mut self, message: impl Into<String>) {
        let entry = format!("#{:04} {}", self.debug_log_seq, message.into());
        self.debug_log_seq += 1;
        if self.debug_log.len() >= DEBUG_LOG_LIMIT {
            self.debug_log.pop_front();
        }
        self.debug_log.push_back(entry);
    }
}
