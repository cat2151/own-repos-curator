use super::{
    App, DescDisplayMode, GroupBindingMode, GroupInput, GroupManager, HelpScreen, TagBindingMode,
    TagFilterMode, TagInput, TagManager, TextEditor,
};
use crate::{
    config::AppConfig,
    github::sync_repo_data,
    json_auto_push::{maybe_push_json_snapshot, AutoPushOutcome},
    model::RepoData,
};
use anyhow::Result;
use ratatui::widgets::ListState;
use std::collections::{BTreeSet, VecDeque};

pub(super) fn load_app() -> Result<App> {
    let (mut data, data_path) = RepoData::load_or_init()?;
    let mut startup_logs = Vec::new();
    let sync_status = run_startup_sync(&mut data, &mut startup_logs);
    let auto_push_status = run_startup_auto_push(&mut data, &mut startup_logs);
    let status_message = format!("{sync_status} | {auto_push_status}");

    data.write_to_path(&data_path)?;

    let mut app = App {
        data,
        data_path,
        list_state: ListState::default(),
        help_screen: None::<HelpScreen>,
        status_message,
        editor: None::<TextEditor>,
        tag_manager: None::<TagManager>,
        tag_input: None::<TagInput>,
        tag_binding_mode: None::<TagBindingMode>,
        group_manager: None::<GroupManager>,
        group_input: None::<GroupInput>,
        group_binding_mode: None::<GroupBindingMode>,
        tag_filter: BTreeSet::new(),
        tag_filter_mode: None::<TagFilterMode>,
        registered_tag_page: 0,
        registered_group_page: 0,
        sort_mode: super::SortMode::Created,
        desc_display_mode: DescDisplayMode::RightPane,
        debug_log_expanded: false,
        debug_log: VecDeque::new(),
        debug_log_seq: 0,
    };

    app.push_debug_log("app loaded");
    for entry in startup_logs {
        app.push_debug_log(entry);
    }
    app.sync_selection();
    Ok(app)
}

fn run_startup_sync(data: &mut RepoData, startup_logs: &mut Vec<String>) -> String {
    match sync_repo_data(data) {
        Ok(summary) => {
            startup_logs.push(format!(
                "startup github sync succeeded: added={} updated={}",
                summary.added, summary.updated
            ));
            format!(
                "起動時GitHub同期完了: {}件追加 / {}件更新",
                summary.added, summary.updated
            )
        }
        Err(error) => {
            startup_logs.push(format!("startup github sync failed: {error}"));
            format!("起動時GitHub同期失敗: {error} (r で再試行)")
        }
    }
}

fn run_startup_auto_push(data: &mut RepoData, startup_logs: &mut Vec<String>) -> String {
    let config = match AppConfig::load_or_init() {
        Ok(config) => config,
        Err(error) => {
            startup_logs.push(format!("startup json auto push config failed: {error}"));
            return format!("JSON自動push設定失敗: {error}");
        }
    };

    match maybe_push_json_snapshot(data, &config) {
        Ok(AutoPushOutcome::Disabled) => {
            startup_logs.push("startup json auto push skipped: repo not configured".to_string());
            "JSON自動push: repo未設定".to_string()
        }
        Ok(AutoPushOutcome::SkippedToday { date }) => {
            startup_logs.push(format!(
                "startup json auto push skipped: already pushed on {date}"
            ));
            "JSON自動push: 本日は実行済み".to_string()
        }
        Ok(AutoPushOutcome::UpToDate { repo, date }) => {
            startup_logs.push(format!(
                "startup json auto push matched remote snapshot: repo={} date={}",
                repo, date
            ));
            "JSON自動push: 変更なし".to_string()
        }
        Ok(AutoPushOutcome::Pushed {
            repo,
            date,
            commit_id,
        }) => {
            startup_logs.push(format!(
                "startup json auto push succeeded: repo={} date={} commit={}",
                repo, date, commit_id
            ));
            "JSON自動push完了".to_string()
        }
        Err(error) => {
            startup_logs.push(format!("startup json auto push failed: {error}"));
            format!("JSON自動push失敗: {error}")
        }
    }
}
