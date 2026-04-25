use super::{
    background_progress::{SyncProgressKind, SyncProgressTracker},
    SyncProgressState,
};
use crate::{
    config::AppConfig,
    github::{fetch_remote_repos_with_progress, FetchedRepo},
    json_auto_push::{
        maybe_push_json_snapshot_with_progress, push_json_snapshot_manually_with_progress,
        AutoPushOutcome,
    },
    model::RepoData,
};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

const SPINNER_FRAMES: [&str; 4] = ["-", "\\", "|", "/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum JsonPushKind {
    StartupAuto,
    Manual,
}

pub(crate) struct StartupJobs {
    github_sync: Option<Receiver<GithubSyncEvent>>,
    github_progress: Option<SyncProgressTracker>,
    json_push: Option<Receiver<JsonPushEvent>>,
    json_push_kind: Option<JsonPushKind>,
    json_push_progress: Option<SyncProgressTracker>,
    spinner_frame: usize,
    #[cfg(test)]
    json_push_spawn_enabled: bool,
    #[cfg(test)]
    held_github_sync_sender: Option<mpsc::Sender<GithubSyncEvent>>,
    #[cfg(test)]
    held_json_push_sender: Option<mpsc::Sender<JsonPushEvent>>,
}

enum GithubSyncEvent {
    Progress(String),
    Finished(Result<Vec<FetchedRepo>, String>),
}

enum JsonPushEvent {
    Progress(String),
    Finished(Result<JsonPushWorkerResult, String>),
}

pub(super) struct JsonPushWorkerResult {
    pub(super) kind: JsonPushKind,
    pub(super) outcome: AutoPushOutcome,
    pub(super) last_json_commit_push_date: Option<String>,
}

impl StartupJobs {
    pub(crate) fn idle() -> Self {
        Self {
            github_sync: None,
            github_progress: None,
            json_push: None,
            json_push_kind: None,
            json_push_progress: None,
            spinner_frame: 0,
            #[cfg(test)]
            json_push_spawn_enabled: true,
            #[cfg(test)]
            held_github_sync_sender: None,
            #[cfg(test)]
            held_json_push_sender: None,
        }
    }

    pub(crate) fn start() -> Self {
        let mut jobs = Self::idle();
        jobs.spawn_github_sync(SyncProgressKind::StartupGithub);
        jobs
    }

    pub(super) fn activity_label(&self) -> Option<String> {
        if self.github_sync.is_some() {
            return self
                .github_progress
                .as_ref()
                .map(|progress| progress.activity_label(SPINNER_FRAMES[self.spinner_frame]));
        }

        if self.json_push.is_some() {
            return self
                .json_push_progress
                .as_ref()
                .map(|progress| progress.activity_label(SPINNER_FRAMES[self.spinner_frame]));
        }

        None
    }

    pub(super) fn sync_progress_state(&self) -> Option<SyncProgressState> {
        if self.github_sync.is_some() {
            return self
                .github_progress
                .as_ref()
                .and_then(SyncProgressTracker::state);
        }

        if self.json_push.is_some() {
            return self
                .json_push_progress
                .as_ref()
                .and_then(SyncProgressTracker::state);
        }

        None
    }

    pub(super) fn advance_spinner(&mut self) {
        if self.github_sync.is_none() && self.json_push.is_none() {
            self.spinner_frame = 0;
            return;
        }

        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    fn spawn_github_sync(&mut self, kind: SyncProgressKind) {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let progress_tx = tx.clone();
            let result = fetch_remote_repos_with_progress(|message| {
                let _ = progress_tx.send(GithubSyncEvent::Progress(message.to_string()));
            })
            .map_err(|error| error.to_string());
            let _ = tx.send(GithubSyncEvent::Finished(result));
        });
        self.github_sync = Some(rx);
        self.github_progress = Some(SyncProgressTracker::new(kind, "GitHub同期を準備中"));
    }

    pub(super) fn start_startup_json_auto_push(&mut self, data: RepoData) {
        self.start_json_push(data, JsonPushKind::StartupAuto);
    }

    pub(crate) fn try_start_manual_github_sync(&mut self) -> Result<(), &'static str> {
        if self.github_sync.is_some() || self.json_push.is_some() {
            return Err("起動時バックグラウンド処理の完了後に再実行してください");
        }

        self.spawn_github_sync(SyncProgressKind::ManualGithub);
        Ok(())
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
            self.json_push_progress = Some(SyncProgressTracker::new(
                json_push_progress_kind(kind),
                "JSON pushを準備中",
            ));
            return;
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let progress_tx = tx.clone();
            let result = run_json_push(data, kind, |message| {
                let _ = progress_tx.send(JsonPushEvent::Progress(message.to_string()));
            })
            .map_err(|error| error.to_string());
            let _ = tx.send(JsonPushEvent::Finished(result));
        });
        self.json_push = Some(rx);
        self.json_push_kind = Some(kind);
        self.json_push_progress = Some(SyncProgressTracker::new(
            json_push_progress_kind(kind),
            "JSON pushを準備中",
        ));
    }

    pub(super) fn take_github_sync_result(
        &mut self,
    ) -> Option<(SyncProgressKind, Result<Vec<FetchedRepo>, String>)> {
        loop {
            let rx = self.github_sync.as_ref()?;
            match rx.try_recv() {
                Ok(GithubSyncEvent::Progress(message)) => {
                    if let Some(progress) = self.github_progress.as_mut() {
                        progress.update_phase(message);
                    }
                }
                Ok(GithubSyncEvent::Finished(result)) => {
                    let kind = self.github_progress_kind();
                    self.clear_github_sync();
                    return Some((kind, result));
                }
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    let kind = self.github_progress_kind();
                    self.clear_github_sync();
                    return Some((kind, Err("github sync worker disconnected".to_string())));
                }
            }
        }
    }

    pub(super) fn take_json_push_result(&mut self) -> Option<Result<JsonPushWorkerResult, String>> {
        loop {
            let rx = self.json_push.as_ref()?;
            match rx.try_recv() {
                Ok(JsonPushEvent::Progress(message)) => {
                    if let Some(progress) = self.json_push_progress.as_mut() {
                        progress.update_phase(message);
                    }
                }
                Ok(JsonPushEvent::Finished(result)) => {
                    self.clear_json_push_receiver();
                    return Some(result);
                }
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.clear_json_push_receiver();
                    return Some(Err("json push worker disconnected".to_string()));
                }
            }
        }
    }

    fn github_progress_kind(&self) -> SyncProgressKind {
        self.github_progress
            .as_ref()
            .map(SyncProgressTracker::kind)
            .unwrap_or(SyncProgressKind::StartupGithub)
    }

    fn clear_github_sync(&mut self) {
        self.github_sync = None;
        self.github_progress = None;
        #[cfg(test)]
        {
            self.held_github_sync_sender = None;
        }
    }

    fn clear_json_push_receiver(&mut self) {
        self.json_push = None;
        self.json_push_progress = None;
        #[cfg(test)]
        {
            self.held_json_push_sender = None;
        }
    }

    pub(super) fn json_push_kind_or_startup_auto(&self) -> JsonPushKind {
        self.json_push_kind.unwrap_or(JsonPushKind::StartupAuto)
    }

    pub(super) fn clear_json_push_kind(&mut self) {
        self.json_push_kind = None;
    }

    #[cfg(test)]
    pub(crate) fn with_test_github_sync_result(result: Result<Vec<FetchedRepo>, String>) -> Self {
        let (tx, rx) = mpsc::channel();
        tx.send(GithubSyncEvent::Finished(result))
            .expect("test github sync result should send");
        Self {
            github_sync: Some(rx),
            github_progress: Some(SyncProgressTracker::new(
                SyncProgressKind::StartupGithub,
                "GitHub同期結果待ち",
            )),
            json_push: None,
            json_push_kind: None,
            json_push_progress: None,
            spinner_frame: 0,
            json_push_spawn_enabled: false,
            held_github_sync_sender: None,
            held_json_push_sender: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_running_github_sync_elapsed(
        elapsed: std::time::Duration,
        phase: &str,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            github_sync: Some(rx),
            github_progress: Some(SyncProgressTracker::with_elapsed(
                SyncProgressKind::StartupGithub,
                phase,
                elapsed,
            )),
            json_push: None,
            json_push_kind: None,
            json_push_progress: None,
            spinner_frame: 0,
            json_push_spawn_enabled: false,
            held_github_sync_sender: Some(tx),
            held_json_push_sender: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_running_manual_json_push_elapsed(
        elapsed: std::time::Duration,
        phase: &str,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            github_sync: None,
            github_progress: None,
            json_push: Some(rx),
            json_push_kind: Some(JsonPushKind::Manual),
            json_push_progress: Some(SyncProgressTracker::with_elapsed(
                SyncProgressKind::ManualJsonPush,
                phase,
                elapsed,
            )),
            spinner_frame: 0,
            json_push_spawn_enabled: false,
            held_github_sync_sender: None,
            held_json_push_sender: Some(tx),
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
        tx.send(JsonPushEvent::Finished(result))
            .expect("test auto push result should send");
        Self {
            github_sync: None,
            github_progress: None,
            json_push: Some(rx),
            json_push_kind: Some(kind),
            json_push_progress: Some(SyncProgressTracker::new(
                json_push_progress_kind(kind),
                "JSON push結果待ち",
            )),
            spinner_frame: 0,
            json_push_spawn_enabled: false,
            held_github_sync_sender: None,
            held_json_push_sender: None,
        }
    }
}

fn json_push_progress_kind(kind: JsonPushKind) -> SyncProgressKind {
    match kind {
        JsonPushKind::StartupAuto => SyncProgressKind::StartupAutoJsonPush,
        JsonPushKind::Manual => SyncProgressKind::ManualJsonPush,
    }
}

fn run_json_push(
    mut data: RepoData,
    kind: JsonPushKind,
    mut progress: impl FnMut(&str),
) -> anyhow::Result<JsonPushWorkerResult> {
    let config = AppConfig::load_or_init()?;
    let outcome = match kind {
        JsonPushKind::StartupAuto => {
            maybe_push_json_snapshot_with_progress(&mut data, &config, &mut progress)?
        }
        JsonPushKind::Manual => {
            push_json_snapshot_manually_with_progress(&mut data, &config, &mut progress)?
        }
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
