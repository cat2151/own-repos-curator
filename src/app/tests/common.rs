use super::super::{App, DescDisplayMode, SortMode};
use crate::model::{Meta, Repo, RepoData};
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) fn repo(name: &str, created_at: &str, updated_at: Option<&str>) -> Repo {
    Repo {
        name: name.to_string(),
        created_at: parse_datetime(created_at),
        updated_at: updated_at.map(parse_datetime),
        github_desc: String::new(),
        desc_short: String::new(),
        desc_long: String::new(),
        tags: Vec::new(),
    }
}

pub(super) fn parse_datetime(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

pub(super) fn test_data_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".test-local-data")
        .join(format!("app-{unique}"))
        .join("repos.json")
}

pub(super) fn registered_tags_from_repos(repos: &[Repo]) -> Vec<String> {
    repos
        .iter()
        .flat_map(|repo| repo.tags.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn app_with_repos(repos: Vec<Repo>) -> App {
    let registered_tags = registered_tags_from_repos(&repos);
    app_with_registered_tags(repos, registered_tags)
}

pub(super) fn app_with_registered_tags(repos: Vec<Repo>, registered_tags: Vec<String>) -> App {
    let mut app = App {
        data: RepoData {
            meta: Meta {
                github_desc_updated_at: String::new(),
                last_json_commit_push_date: String::new(),
            },
            registered_tags,
            repos,
        },
        data_path: test_data_path(),
        list_state: ListState::default(),
        help_screen: None,
        status_message: String::new(),
        editor: None,
        tag_manager: None,
        tag_input: None,
        tag_binding_mode: None,
        tag_filter: Default::default(),
        tag_filter_mode: None,
        registered_tag_page: 0,
        sort_mode: SortMode::Created,
        desc_display_mode: DescDisplayMode::RightPane,
        debug_log_expanded: false,
        debug_log: VecDeque::new(),
        debug_log_seq: 0,
    };
    app.sync_selection();
    app
}

pub(super) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

pub(super) fn ctrl_key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
}

pub(super) fn shift_key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch.to_ascii_uppercase()), KeyModifiers::SHIFT)
}

pub(super) fn cleanup_app_file(app: &App) {
    if let Some(parent) = app.data_path.parent() {
        let _ = fs::remove_file(&app.data_path);
        let _ = fs::remove_dir(parent);
    }
}
