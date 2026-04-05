use super::{
    background::StartupJobs, App, DescDisplayMode, GroupBindingMode, GroupInput, GroupManager,
    HelpScreen, TagBindingMode, TagInput, TagManager, TextEditor,
};
use crate::model::RepoData;
use anyhow::Result;
use ratatui::widgets::ListState;
use std::collections::{BTreeSet, VecDeque};

pub(super) fn load_app() -> Result<App> {
    let (data, data_path) = RepoData::load_or_init()?;

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
        desc_display_mode: DescDisplayMode::RightPane,
        debug_log_expanded: false,
        debug_log: VecDeque::new(),
        debug_log_seq: 0,
        startup_jobs: StartupJobs::idle(),
    };

    app.push_debug_log("app loaded");
    app.start_background_startup();
    app.sync_selection();
    Ok(app)
}
