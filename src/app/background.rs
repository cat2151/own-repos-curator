use super::App;
use crate::{
    config::AppConfig,
    github::{apply_fetched_repos, fetch_remote_repos, FetchedRepo},
    json_auto_push::{maybe_push_json_snapshot, push_json_snapshot_manually, AutoPushOutcome},
    model::RepoData,
};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

const SPINNER_FRAMES: [&str; 4] = ["-", "\\", "|", "/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonPushKind {
    StartupAuto,
    Manual,
}

pub(crate) struct StartupJobs {
    github_sync: Option<Receiver<Result<Vec<FetchedRepo>, String>>>,
    json_push: Option<Receiver<Result<JsonPushWorkerResult, String>>>,
    json_push_kind: Option<JsonPushKind>,
    spinner_frame: usize,
    #[cfg(test)]
    json_push_spawn_enabled: bool,
    #[cfg(test)]
    held_json_push_sender: Option<mpsc::Sender<Result<JsonPushWorkerResult, String>>>,
}

struct JsonPushWorkerResult {
    kind: JsonPushKind,
    outcome: AutoPushOutcome,
    last_json_commit_push_date: Option<String>,
}

impl StartupJobs {
    pub(crate) fn idle() -> Self {
        Self {
            github_sync: None,
            json_push: None,
            json_push_kind: None,
            spinner_frame: 0,
            #[cfg(test)]
            json_push_spawn_enabled: true,
            #[cfg(test)]
            held_json_push_sender: None,
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

        if self.json_push.is_some() {
            let label = match self.json_push_kind.unwrap_or(JsonPushKind::StartupAuto) {
                JsonPushKind::StartupAuto => "起動時JSON自動pushをバックグラウンド実行中",
                JsonPushKind::Manual => "JSON手動pushをバックグラウンド実行中",
            };
            return Some(format!("[{}] {label}", SPINNER_FRAMES[self.spinner_frame]));
        }

        None
    }

    pub(crate) fn advance_spinner(&mut self) {
        if self.github_sync.is_none() && self.json_push.is_none() {
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

    fn start_startup_json_auto_push(&mut self, data: RepoData) {
        self.start_json_push(data, JsonPushKind::StartupAuto);
    }

    pub(crate) fn try_start_manual_json_push(
        &mut self,
        data: RepoData,
    ) -> Result<(), &'static str> {
        if self.github_sync.is_some() {
            return Err("起動時バックグラウンド処理の完了後に再実行してください");
        }
        if self.json_push.is_some() {
            return Err("JSON pushはすでに実行中です");
        }

        self.start_json_push(data, JsonPushKind::Manual);
        Ok(())
    }

    fn start_json_push(&mut self, data: RepoData, kind: JsonPushKind) {
        #[cfg(test)]
        if !self.json_push_spawn_enabled {
            let (tx, rx) = mpsc::channel();
            self.held_json_push_sender = Some(tx);
            self.json_push = Some(rx);
            self.json_push_kind = Some(kind);
            return;
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = run_json_push(data, kind).map_err(|error| error.to_string());
            let _ = tx.send(result);
        });
        self.json_push = Some(rx);
        self.json_push_kind = Some(kind);
    }

    fn take_github_sync_result(&mut self) -> Option<Result<Vec<FetchedRepo>, String>> {
        take_receiver_result(
            &mut self.github_sync,
            "startup github sync worker disconnected",
        )
    }

    fn take_json_push_result(&mut self) -> Option<Result<JsonPushWorkerResult, String>> {
        let result = take_receiver_result(&mut self.json_push, "json push worker disconnected");
        if result.is_some() {
            #[cfg(test)]
            {
                self.held_json_push_sender = None;
            }
        }
        result
    }

    #[cfg(test)]
    pub(crate) fn with_test_github_sync_result(result: Result<Vec<FetchedRepo>, String>) -> Self {
        let (tx, rx) = mpsc::channel();
        tx.send(result)
            .expect("test github sync result should send");
        Self {
            github_sync: Some(rx),
            json_push: None,
            json_push_kind: None,
            spinner_frame: 0,
            json_push_spawn_enabled: false,
            held_json_push_sender: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_auto_push_result(
        outcome: Result<(AutoPushOutcome, Option<String>), String>,
    ) -> Self {
        Self::with_test_json_push_result(JsonPushKind::StartupAuto, outcome)
    }

    #[cfg(test)]
    pub(crate) fn with_test_manual_push_result(
        outcome: Result<(AutoPushOutcome, Option<String>), String>,
    ) -> Self {
        Self::with_test_json_push_result(JsonPushKind::Manual, outcome)
    }

    #[cfg(test)]
    pub(crate) fn idle_without_json_push_spawn() -> Self {
        let mut jobs = Self::idle();
        jobs.json_push_spawn_enabled = false;
        jobs
    }

    #[cfg(test)]
    fn with_test_json_push_result(
        kind: JsonPushKind,
        outcome: Result<(AutoPushOutcome, Option<String>), String>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let result = outcome.map(
            |(outcome, last_json_commit_push_date)| JsonPushWorkerResult {
                kind,
                outcome,
                last_json_commit_push_date,
            },
        );
        tx.send(result).expect("test auto push result should send");
        Self {
            github_sync: None,
            json_push: Some(rx),
            json_push_kind: Some(kind),
            spinner_frame: 0,
            json_push_spawn_enabled: false,
            held_json_push_sender: None,
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

fn run_json_push(mut data: RepoData, kind: JsonPushKind) -> anyhow::Result<JsonPushWorkerResult> {
    let config = AppConfig::load_or_init()?;
    let outcome = match kind {
        JsonPushKind::StartupAuto => maybe_push_json_snapshot(&mut data, &config)?,
        JsonPushKind::Manual => push_json_snapshot_manually(&mut data, &config)?,
    };
    Ok(JsonPushWorkerResult {
        kind,
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
        if let Some(result) = self.startup_jobs.take_json_push_result() {
            self.finish_json_push(result);
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
        self.startup_jobs
            .start_startup_json_auto_push(self.data.clone());
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
            Err(_) => self
                .startup_jobs
                .json_push_kind
                .unwrap_or(JsonPushKind::StartupAuto),
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
                        self.startup_jobs.json_push_kind = None;
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

        self.startup_jobs.json_push_kind = None;
    }
}
