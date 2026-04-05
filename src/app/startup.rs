use super::{
    background::StartupJobs, App, AppHistory, GroupBindingMode, GroupInput, GroupManager,
    HelpScreen, TagBindingMode, TagInput, TagManager, TextEditor,
};
use crate::model::RepoData;
use anyhow::Result;
use ratatui::widgets::ListState;
use std::{
    collections::{BTreeSet, VecDeque},
    path::PathBuf,
};

pub(super) fn load_app() -> Result<App> {
    let (data, data_path) = RepoData::load_or_init()?;
    let history_path = crate::paths::history_file_path()?;
    let (history, history_load_error) = match AppHistory::load_from_path(&history_path) {
        Ok(history) => (history, None),
        Err(error) => (AppHistory::default(), Some(error.to_string())),
    };
    let mut app = build_app(data, data_path, history);
    if let Some(error) = history_load_error {
        app.push_debug_log(format!("app history load fallback: {error}"));
    }
    app.start_background_startup();
    Ok(app)
}

fn build_app(data: RepoData, data_path: PathBuf, history: AppHistory) -> App {
    let mut app = App {
        data,
        data_path,
        list_state: ListState::default(),
        help_screen: None::<HelpScreen>,
        status_message: "ローカルデータを表示しました".to_string(),
        editor: None::<TextEditor>,
        tag_manager: None::<TagManager>,
        tag_input: None::<TagInput>,
        tag_binding_mode: None::<TagBindingMode>,
        group_manager: None::<GroupManager>,
        group_input: None::<GroupInput>,
        group_binding_mode: None::<GroupBindingMode>,
        group_filter: None,
        tag_filter: BTreeSet::new(),
        filter_mode: None,
        registered_tag_page: 0,
        registered_group_page: 0,
        sort_mode: super::SortMode::Created,
        desc_display_mode: history.desc_display_mode,
        debug_log_expanded: false,
        debug_log: VecDeque::new(),
        debug_log_seq: 0,
        startup_jobs: StartupJobs::idle(),
    };

    app.push_debug_log("app loaded");
    app.sync_selection();
    app
}

#[cfg(test)]
mod tests {
    use super::build_app;
    use crate::{
        app::{AppHistory, DescDisplayMode},
        model::RepoData,
    };
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_HISTORY_PATH_SEQ: AtomicU64 = AtomicU64::new(0);

    fn unique_test_history_path() -> PathBuf {
        let unique = TEST_HISTORY_PATH_SEQ.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".test-local-data")
            .join(format!("missing-history-{unique}"))
            .join("history.json")
    }

    #[test]
    fn build_app_restores_desc_display_mode_from_history() {
        let app = build_app(
            RepoData::empty(),
            PathBuf::from("repos.json"),
            AppHistory {
                desc_display_mode: DescDisplayMode::LeftShortAndLong,
            },
        );

        assert_eq!(app.desc_display_mode(), DescDisplayMode::LeftShortAndLong);
    }

    #[test]
    fn missing_history_defaults_to_right_pane_mode() {
        let history =
            AppHistory::load_from_path(&unique_test_history_path()).expect("history load");

        assert_eq!(history.desc_display_mode, DescDisplayMode::RightPane);
    }
}
