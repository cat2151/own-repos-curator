use super::{
    background::{JsonPushKind, JsonPushWorkerResult},
    background_progress::SyncProgressKind,
    App, SyncProgressState,
};
use crate::{
    github::{apply_fetched_repos, FetchedRepo},
    json_auto_push::AutoPushOutcome,
};

impl App {
    pub(crate) fn start_background_startup(&mut self) {
        self.startup_jobs = super::background::StartupJobs::start();
        self.push_debug_log("startup github sync scheduled in background");
    }

    pub(crate) fn background_status_message(&self) -> Option<String> {
        self.startup_jobs.activity_label()
    }

    pub(crate) fn sync_progress_state(&self) -> Option<SyncProgressState> {
        self.startup_jobs.sync_progress_state()
    }

    pub(crate) fn tick_background_jobs(&mut self) {
        self.startup_jobs.advance_spinner();

        if let Some((kind, result)) = self.startup_jobs.take_github_sync_result() {
            self.finish_github_sync(kind, result);
        }
        if let Some(result) = self.startup_jobs.take_json_push_result() {
            self.finish_json_push(result);
        }
    }

    fn finish_github_sync(
        &mut self,
        kind: SyncProgressKind,
        result: Result<Vec<FetchedRepo>, String>,
    ) {
        let status_label = github_sync_status_label(kind);
        let debug_label = github_sync_debug_label(kind);

        match result {
            Ok(fetched) => {
                let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
                let summary = apply_fetched_repos(&mut self.data, fetched);
                self.restore_selection(selected_repo_name.as_deref());

                match self.persist_data() {
                    Ok(()) => {
                        self.status_message = format!(
                            "{status_label}完了: {}件追加 / {}件更新",
                            summary.added, summary.updated
                        );
                    }
                    Err(error) => {
                        self.status_message = format!("{status_label}の保存失敗: {error}");
                        self.push_debug_log(format!("{debug_label} persist failed: {error}"));
                    }
                }

                self.push_debug_log(format!(
                    "{debug_label} succeeded: added={} updated={}",
                    summary.added, summary.updated
                ));
            }
            Err(error) => {
                self.status_message = format!("{status_label}失敗: {error} (r で再試行)");
                self.push_debug_log(format!("{debug_label} failed: {error}"));
            }
        }

        if matches!(kind, SyncProgressKind::StartupGithub) {
            self.push_debug_log("startup json auto push scheduled in background");
            self.startup_jobs
                .start_startup_json_auto_push(self.data.clone());
        }
    }

    pub(crate) fn start_manual_github_sync(&mut self) {
        match self.startup_jobs.try_start_manual_github_sync() {
            Ok(()) => {
                self.status_message = "GitHub同期を開始".to_string();
                self.push_debug_log("manual github sync scheduled in background");
            }
            Err(message) => {
                self.status_message = message.to_string();
                self.push_debug_log(format!("manual github sync skipped: {message}"));
            }
        }
    }

    pub(crate) fn start_manual_json_push(&mut self) {
        match self
            .startup_jobs
            .try_start_manual_json_push(self.data.clone())
        {
            Ok(()) => {
                self.status_message = "JSON手動pushを開始".to_string();
                self.push_debug_log("manual json push scheduled in background");
            }
            Err(message) => {
                self.status_message = message.to_string();
                self.push_debug_log(format!("manual json push skipped: {message}"));
            }
        }
    }

    fn finish_json_push(&mut self, result: Result<JsonPushWorkerResult, String>) {
        let kind = match &result {
            Ok(worker) => worker.kind,
            Err(_) => self.startup_jobs.json_push_kind_or_startup_auto(),
        };

        match result {
            Ok(worker) => {
                if let Some(date) = worker.last_json_commit_push_date {
                    self.data.meta.last_json_commit_push_date = date;
                    if let Err(error) = self.persist_data() {
                        self.status_message = format!(
                            "{}状態の保存失敗: {error}",
                            json_push_status_label(worker.kind)
                        );
                        self.push_debug_log(format!(
                            "{} persist failed: {error}",
                            json_push_debug_label(worker.kind)
                        ));
                        self.startup_jobs.clear_json_push_kind();
                        return;
                    }
                }

                self.status_message = json_push_status(worker.kind, &worker.outcome);
                self.push_debug_log(json_push_debug_log(worker.kind, &worker.outcome));
            }
            Err(error) => {
                self.status_message = format!("{}失敗: {error}", json_push_status_label(kind));
                self.push_debug_log(format!("{} failed: {error}", json_push_debug_label(kind)));
            }
        }

        self.startup_jobs.clear_json_push_kind();
    }
}

fn github_sync_status_label(kind: SyncProgressKind) -> &'static str {
    match kind {
        SyncProgressKind::StartupGithub => "起動時GitHub同期",
        SyncProgressKind::ManualGithub => "GitHub同期",
        SyncProgressKind::StartupAutoJsonPush | SyncProgressKind::ManualJsonPush => "GitHub同期",
    }
}

fn github_sync_debug_label(kind: SyncProgressKind) -> &'static str {
    match kind {
        SyncProgressKind::StartupGithub => "startup github sync",
        SyncProgressKind::ManualGithub => "manual github sync",
        SyncProgressKind::StartupAutoJsonPush | SyncProgressKind::ManualJsonPush => "github sync",
    }
}

fn json_push_status_label(kind: JsonPushKind) -> &'static str {
    match kind {
        JsonPushKind::StartupAuto => "JSON自動push",
        JsonPushKind::Manual => "JSON手動push",
    }
}

fn json_push_debug_label(kind: JsonPushKind) -> &'static str {
    match kind {
        JsonPushKind::StartupAuto => "startup json auto push",
        JsonPushKind::Manual => "manual json push",
    }
}

fn json_push_status(kind: JsonPushKind, outcome: &AutoPushOutcome) -> String {
    let label = json_push_status_label(kind);
    match outcome {
        AutoPushOutcome::Disabled => format!("{label}: repo未設定"),
        AutoPushOutcome::SkippedToday { .. } => format!("{label}: 本日は実行済み"),
        AutoPushOutcome::UpToDate { .. } => format!("{label}: 変更なし"),
        AutoPushOutcome::Pushed { .. } => format!("{label}完了"),
    }
}

fn json_push_debug_log(kind: JsonPushKind, outcome: &AutoPushOutcome) -> String {
    let label = json_push_debug_label(kind);
    match outcome {
        AutoPushOutcome::Disabled => format!("{label} skipped: repo not configured"),
        AutoPushOutcome::SkippedToday { date } => {
            format!("{label} skipped: already pushed on {date}")
        }
        AutoPushOutcome::UpToDate { repo, date } => {
            format!("{label} matched remote snapshot: repo={repo} date={date}")
        }
        AutoPushOutcome::Pushed {
            repo,
            date,
            commit_id,
        } => {
            format!("{label} succeeded: repo={repo} date={date} commit={commit_id}")
        }
    }
}
