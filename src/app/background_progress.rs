use std::time::{Duration, Instant};

const SLOW_SYNC_OVERLAY_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyncProgressKind {
    StartupGithub,
    ManualGithub,
    StartupAutoJsonPush,
    ManualJsonPush,
}

impl SyncProgressKind {
    fn title(self) -> &'static str {
        match self {
            Self::StartupGithub => "起動時GitHub同期",
            Self::ManualGithub => "GitHub同期",
            Self::StartupAutoJsonPush => "起動時JSON自動push",
            Self::ManualJsonPush => "JSON手動push",
        }
    }

    pub(crate) fn activity_label(self) -> &'static str {
        match self {
            Self::StartupGithub => "起動時GitHub同期をバックグラウンド実行中",
            Self::ManualGithub => "GitHub同期をバックグラウンド実行中",
            Self::StartupAutoJsonPush => "起動時JSON自動pushをバックグラウンド実行中",
            Self::ManualJsonPush => "JSON手動pushをバックグラウンド実行中",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyncProgressState {
    pub(crate) title: String,
    pub(crate) phase: String,
    pub(crate) elapsed_secs: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct SyncProgressTracker {
    kind: SyncProgressKind,
    phase: String,
    started_at: Instant,
}

impl SyncProgressTracker {
    pub(crate) fn new(kind: SyncProgressKind, phase: impl Into<String>) -> Self {
        Self {
            kind,
            phase: phase.into(),
            started_at: Instant::now(),
        }
    }

    pub(crate) fn kind(&self) -> SyncProgressKind {
        self.kind
    }

    pub(crate) fn update_phase(&mut self, phase: impl Into<String>) {
        self.phase = phase.into();
    }

    pub(crate) fn activity_label(&self, spinner: &str) -> String {
        format!("[{spinner}] {}", self.kind.activity_label())
    }

    pub(crate) fn state(&self) -> Option<SyncProgressState> {
        let elapsed = self.started_at.elapsed();
        if elapsed < SLOW_SYNC_OVERLAY_DELAY {
            return None;
        }

        Some(SyncProgressState {
            title: self.kind.title().to_string(),
            phase: self.phase.clone(),
            elapsed_secs: elapsed.as_secs(),
        })
    }

    #[cfg(test)]
    pub(crate) fn with_elapsed(
        kind: SyncProgressKind,
        phase: impl Into<String>,
        elapsed: Duration,
    ) -> Self {
        Self {
            kind,
            phase: phase.into(),
            started_at: Instant::now() - elapsed,
        }
    }
}
