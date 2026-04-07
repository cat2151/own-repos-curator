mod background;
mod editor;
mod filter;
mod groups;
mod helpers;
mod key_handling;
mod navigation;
mod persistence;
mod startup;
mod state;
mod tags;
mod view;

#[cfg(test)]
mod tests;

use crate::model::RepoData;
use anyhow::Result;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    path::PathBuf,
};

pub(crate) use self::state::{
    EditorField, FilterMode, FilterModeFocus, FilterModeState, GroupBinding, GroupBindingMode,
    GroupBindingModeState, GroupCatalogEntry, GroupCatalogState, GroupInput, GroupInputMode,
    GroupManager, GroupManagerEntry, GroupManagerState, GroupSummaryEntry, SelectedRepoDescState,
    SelectedRepoTagDetailEntry, SelectedRepoTagDetailState, TagBinding, TagBindingMode,
    TagBindingModeState, TagCatalogEntry, TagCatalogState, TagInput, TagInputMode, TagManager,
    TagManagerEntry, TagManagerState, TextEditor,
};

pub(crate) const TAG_KEYS: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
pub(crate) const REPO_PAGE_STEP: usize = 10;
const DEBUG_LOG_LIMIT: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppEvent {
    Continue,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HelpScreen {
    Main,
    TagBinding,
    GroupBinding,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DescDisplayMode {
    #[default]
    RightPane,
    LeftShort,
    LeftShortAndLong,
}

impl DescDisplayMode {
    fn cycle(self) -> Self {
        match self {
            Self::RightPane => Self::LeftShort,
            Self::LeftShort => Self::LeftShortAndLong,
            Self::LeftShortAndLong => Self::RightPane,
        }
    }

    pub(crate) fn shows_right_desc_pane(self) -> bool {
        matches!(self, Self::RightPane)
    }

    pub(crate) fn shows_inline_short_desc(self) -> bool {
        !matches!(self, Self::RightPane)
    }

    pub(crate) fn shows_inline_long_desc(self) -> bool {
        matches!(self, Self::LeftShortAndLong)
    }

    fn status_message(self) -> &'static str {
        match self {
            Self::RightPane => "desc表示: 右下paneに1行/3行説明",
            Self::LeftShort => "desc表示: 左paneに1行説明",
            Self::LeftShortAndLong => "desc表示: 左paneに1行説明+3行説明",
        }
    }

    fn debug_log_message(self) -> &'static str {
        match self {
            Self::RightPane => "description display mode: right pane detail mode",
            Self::LeftShort => "description display mode: left pane short description",
            Self::LeftShortAndLong => {
                "description display mode: left pane short and long descriptions"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct AppHistory {
    #[serde(default)]
    pub(crate) desc_display_mode: DescDisplayMode,
}

impl AppHistory {
    pub(crate) fn load_from_path(path: &std::path::Path) -> Result<Self> {
        use anyhow::Context;

        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read app history: {}", path.display()))?;
        serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse app history: {}", path.display()))
    }

    pub(crate) fn write_to_path(&self, path: &std::path::Path) -> Result<()> {
        use anyhow::Context;

        let parent = path
            .parent()
            .with_context(|| format!("failed to resolve parent directory: {}", path.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create history directory: {}", parent.display()))?;

        let json = serde_json::to_string_pretty(self).context("failed to serialize app history")?;
        fs::write(path, &json)
            .with_context(|| format!("failed to write app history: {}", path.display()))?;
        Ok(())
    }
}

pub(crate) struct App {
    pub(crate) data: RepoData,
    data_path: PathBuf,
    pub(crate) list_state: ListState,
    pub(crate) help_screen: Option<HelpScreen>,
    pub(crate) status_message: String,
    pub(crate) editor: Option<TextEditor>,
    pub(crate) tag_manager: Option<TagManager>,
    pub(crate) tag_input: Option<TagInput>,
    pub(crate) tag_binding_mode: Option<TagBindingMode>,
    pub(crate) group_manager: Option<GroupManager>,
    pub(crate) group_input: Option<GroupInput>,
    pub(crate) group_binding_mode: Option<GroupBindingMode>,
    pub(crate) group_filter: Option<String>,
    pub(crate) tag_filter: BTreeSet<String>,
    pub(crate) filter_mode: Option<FilterMode>,
    registered_tag_page: usize,
    registered_group_page: usize,
    sort_mode: SortMode,
    desc_display_mode: DescDisplayMode,
    debug_log_expanded: bool,
    debug_log: VecDeque<String>,
    debug_log_seq: usize,
    startup_jobs: background::StartupJobs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortMode {
    Created,
    Modified,
}

impl SortMode {
    fn toggle(self) -> Self {
        match self {
            Self::Created => Self::Modified,
            Self::Modified => Self::Created,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Created => "create",
            Self::Modified => "modify",
        }
    }
}

impl App {
    pub(crate) fn load() -> Result<Self> {
        startup::load_app()
    }
}
