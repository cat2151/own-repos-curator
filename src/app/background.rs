use super::App;
use crate::{
    config::AppConfig,
    github::{apply_fetched_repos, fetch_remote_repos, FetchedRepo},
    json_auto_push::{maybe_push_json_snapshot, AutoPushOutcome},
    model::RepoData,
};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

const SPINNER_FRAMES: [&str; 4] = ["-", "\\", "|", "/"];

pub(crate) struct StartupJobs {
    github_sync: Option<Receiver<Result<Vec<FetchedRepo>, String>>>,
    json_auto_push: Option<Receiver<Result<AutoPushWorkerResult, String>>>,
    spinner_frame: usize,
    #[cfg(test)]
    auto_push_spawn_enabled: bool,
}

struct AutoPushWorkerResult {
    outcome: AutoPushOutcome,
    last_json_commit_push_date: Option<String>,
}

impl StartupJobs {
    pub(crate) fn idle() -> Self {
        Self {
            github_sync: None,
            json_auto_push: None,
            spinner_frame: 0,
            #[cfg(test)]
            auto_push_spawn_enabled: true,
        }
    }

    pub(crate) fn start() -> Self {
        let mut jobs = Self::idle();
        jobs.spawn_github_sync();
        jobs
    }

    pub(crate) fn activity_label(&self) -> Option<String> {
        if self.github_sync.is_some() {
            return Some(format!(
                "[{}] 起動時GitHub同期をバックグラウンド実行中",
                SPINNER_FRAMES[self.spinner_frame]
            ));
        }

        if self.json_auto_push.is_some() {
            return Some(format!(
                "[{}] 起動時JSON自動pushをバックグラウンド実行中",
                SPINNER_FRAMES[self.spinner_frame]
            ));
        }

        None
    }

    pub(crate) fn advance_spinner(&mut self) {
        if self.github_sync.is_none() && self.json_auto_push.is_none() {
            self.spinner_frame = 0;
            return;
        }

        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    fn spawn_github_sync(&mut self) {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = fetch_remote_repos().map_err(|error| error.to_string());
            let _ = tx.send(result);
        });
        self.github_sync = Some(rx);
    }

    fn start_json_auto_push(&mut self, data: RepoData) {
        #[cfg(test)]
        if !self.auto_push_spawn_enabled {
            return;
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = run_startup_auto_push(data).map_err(|error| error.to_string());
            let _ = tx.send(result);
        });
        self.json_auto_push = Some(rx);
    }

    fn take_github_sync_result(&mut self) -> Option<Result<Vec<FetchedRepo>, String>> {
        take_receiver_result(
            &mut self.github_sync,
            "startup github sync worker disconnected",
        )
    }

    fn take_json_auto_push_result(&mut self) -> Option<Result<AutoPushWorkerResult, String>> {
        take_receiver_result(
            &mut self.json_auto_push,
            "startup json auto push worker disconnected",
        )
    }

    #[cfg(test)]
    pub(crate) fn with_test_github_sync_result(result: Result<Vec<FetchedRepo>, String>) -> Self {
        let (tx, rx) = mpsc::channel();
        tx.send(result)
            .expect("test github sync result should send");
        Self {
            github_sync: Some(rx),
            json_auto_push: None,
            spinner_frame: 0,
            auto_push_spawn_enabled: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_auto_push_result(
        outcome: Result<(AutoPushOutcome, Option<String>), String>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let result = outcome.map(
            |(outcome, last_json_commit_push_date)| AutoPushWorkerResult {
                outcome,
                last_json_commit_push_date,
            },
        );
        tx.send(result).expect("test auto push result should send");
        Self {
            github_sync: None,
            json_auto_push: Some(rx),
            spinner_frame: 0,
            auto_push_spawn_enabled: false,
        }
    }
}

fn take_receiver_result<T>(
    receiver: &mut Option<Receiver<Result<T, String>>>,
    disconnected_message: &str,
) -> Option<Result<T, String>> {
    let rx = receiver.as_ref()?;

    match rx.try_recv() {
        Ok(result) => {
            *receiver = None;
            Some(result)
        }
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => {
            *receiver = None;
            Some(Err(disconnected_message.to_string()))
        }
    }
}

fn run_startup_auto_push(mut data: RepoData) -> anyhow::Result<AutoPushWorkerResult> {
    let config = AppConfig::load_or_init()?;
    let outcome = maybe_push_json_snapshot(&mut data, &config)?;
    Ok(AutoPushWorkerResult {
        outcome,
        last_json_commit_push_date: non_empty_string(data.meta.last_json_commit_push_date),
    })
}

fn non_empty_string(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn startup_auto_push_status(outcome: &AutoPushOutcome) -> String {
    match outcome {
        AutoPushOutcome::Disabled => "JSON自動push: repo未設定".to_string(),
        AutoPushOutcome::SkippedToday { .. } => "JSON自動push: 本日は実行済み".to_string(),
        AutoPushOutcome::UpToDate { .. } => "JSON自動push: 変更なし".to_string(),
        AutoPushOutcome::Pushed { .. } => "JSON自動push完了".to_string(),
    }
}

fn startup_auto_push_debug_log(outcome: &AutoPushOutcome) -> String {
    match outcome {
        AutoPushOutcome::Disabled => {
            "startup json auto push skipped: repo not configured".to_string()
        }
        AutoPushOutcome::SkippedToday { date } => {
            format!("startup json auto push skipped: already pushed on {date}")
        }
        AutoPushOutcome::UpToDate { repo, date } => {
            format!("startup json auto push matched remote snapshot: repo={repo} date={date}")
        }
        AutoPushOutcome::Pushed {
            repo,
            date,
            commit_id,
        } => {
            format!("startup json auto push succeeded: repo={repo} date={date} commit={commit_id}")
        }
    }
}

impl App {
    pub(crate) fn start_background_startup(&mut self) {
        self.startup_jobs = StartupJobs::start();
        self.push_debug_log("startup github sync scheduled in background");
    }

    pub(crate) fn background_status_message(&self) -> Option<String> {
        self.startup_jobs.activity_label()
    }

    pub(crate) fn tick_background_jobs(&mut self) {
        self.startup_jobs.advance_spinner();

        if let Some(result) = self.startup_jobs.take_github_sync_result() {
            self.finish_startup_github_sync(result);
        }
        if let Some(result) = self.startup_jobs.take_json_auto_push_result() {
            self.finish_startup_json_auto_push(result);
        }
    }

    fn finish_startup_github_sync(&mut self, result: Result<Vec<FetchedRepo>, String>) {
        match result {
            Ok(fetched) => {
                let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
                let summary = apply_fetched_repos(&mut self.data, fetched);
                self.restore_selection(selected_repo_name.as_deref());

                match self.persist_data() {
                    Ok(()) => {
                        self.status_message = format!(
                            "起動時GitHub同期完了: {}件追加 / {}件更新",
                            summary.added, summary.updated
                        );
                    }
                    Err(error) => {
                        self.status_message = format!("起動時GitHub同期の保存失敗: {error}");
                        self.push_debug_log(format!("startup github sync persist failed: {error}"));
                    }
                }

                self.push_debug_log(format!(
                    "startup github sync succeeded: added={} updated={}",
                    summary.added, summary.updated
                ));
            }
            Err(error) => {
                self.status_message = format!("起動時GitHub同期失敗: {error} (r で再試行)");
                self.push_debug_log(format!("startup github sync failed: {error}"));
            }
        }

        self.push_debug_log("startup json auto push scheduled in background");
        self.startup_jobs.start_json_auto_push(self.data.clone());
    }

    fn finish_startup_json_auto_push(&mut self, result: Result<AutoPushWorkerResult, String>) {
        match result {
            Ok(worker) => {
                if let Some(date) = worker.last_json_commit_push_date {
                    self.data.meta.last_json_commit_push_date = date;
                    if let Err(error) = self.persist_data() {
                        self.status_message = format!("JSON自動push状態の保存失敗: {error}");
                        self.push_debug_log(format!(
                            "startup json auto push persist failed: {error}"
                        ));
                        return;
                    }
                }

                self.status_message = startup_auto_push_status(&worker.outcome);
                self.push_debug_log(startup_auto_push_debug_log(&worker.outcome));
            }
            Err(error) => {
                self.status_message = format!("JSON自動push失敗: {error}");
                self.push_debug_log(format!("startup json auto push failed: {error}"));
            }
        }
    }
}
