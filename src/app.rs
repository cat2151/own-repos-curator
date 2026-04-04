mod editor;
mod filter;
mod groups;
mod helpers;
mod navigation;
mod persistence;
mod startup;
mod state;
mod tags;
mod view;

#[cfg(test)]
mod tests;

use self::helpers::{
    describe_key_code, is_ctrl_char, is_plain_or_ctrl_char, is_quit_key, is_shift_char,
};
use crate::model::RepoData;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::{
    collections::{BTreeSet, VecDeque},
    path::PathBuf,
};

pub(crate) use self::state::{
    EditorField, GroupBinding, GroupBindingMode, GroupBindingModeState, GroupCatalogEntry,
    GroupCatalogState, GroupInput, GroupInputMode, GroupManager, GroupManagerEntry,
    GroupManagerState, SelectedRepoDescState, SelectedRepoTagDetailEntry,
    SelectedRepoTagDetailState, TagBinding, TagBindingMode, TagBindingModeState, TagCatalogEntry,
    TagCatalogState, TagFilterMode, TagFilterModeState, TagInput, TagInputMode, TagManager,
    TagManagerEntry, TagManagerState, TagSummaryEntry, TextEditor,
};

pub(crate) const TAG_KEYS: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescDisplayMode {
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
    pub(crate) tag_filter: BTreeSet<String>,
    pub(crate) tag_filter_mode: Option<TagFilterMode>,
    registered_tag_page: usize,
    registered_group_page: usize,
    sort_mode: SortMode,
    desc_display_mode: DescDisplayMode,
    debug_log_expanded: bool,
    debug_log: VecDeque<String>,
    debug_log_seq: usize,
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

    pub(crate) fn tick(&mut self) {}

    pub(crate) fn help_screen(&self) -> Option<HelpScreen> {
        self.help_screen
    }

    pub(crate) fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    pub(crate) fn desc_display_mode(&self) -> DescDisplayMode {
        self.desc_display_mode
    }

    pub(crate) fn debug_log_expanded(&self) -> bool {
        self.debug_log_expanded
    }

    pub(crate) fn note_raw_key_event(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "raw key: code={} modifiers={:?} kind={:?} state={:?}",
            describe_key_code(&key.code),
            key.modifiers,
            key.kind,
            key.state
        ));
    }

    pub(crate) fn note_ignored_key_event(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "ignored key: code={} because kind={:?}",
            describe_key_code(&key.code),
            key.kind
        ));
    }

    pub(crate) fn debug_log_lines(&self) -> Vec<String> {
        self.debug_log.iter().cloned().collect()
    }

    pub(crate) fn toggle_debug_log_pane(&mut self) {
        self.debug_log_expanded = !self.debug_log_expanded;
        self.status_message = if self.debug_log_expanded {
            "debug log: 画面下部50%".to_string()
        } else {
            "debug log: 1行".to_string()
        };
        self.push_debug_log(if self.debug_log_expanded {
            "debug log pane expanded: bottom 50%"
        } else {
            "debug log pane collapsed: single line"
        });
    }

    pub(crate) fn cycle_desc_display_mode(&mut self) {
        self.desc_display_mode = self.desc_display_mode.cycle();
        self.status_message = self.desc_display_mode.status_message().to_string();
        self.push_debug_log(self.desc_display_mode.debug_log_message());
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> AppEvent {
        self.push_debug_log(format!(
            "handle_key: code={} tag_input={} group_input={} editor={} tag_binding={} group_binding={} tag_filter={} tag_manager={} group_manager={} help={}",
            describe_key_code(&key.code),
            self.tag_input.is_some(),
            self.group_input.is_some(),
            self.editor.is_some(),
            self.tag_binding_mode.is_some(),
            self.group_binding_mode.is_some(),
            self.tag_filter_mode.is_some(),
            self.tag_manager.is_some(),
            self.group_manager.is_some(),
            self.help_screen.is_some()
        ));

        if self.tag_input.is_none()
            && self.group_input.is_none()
            && self.editor.is_none()
            && self.tag_binding_mode.is_none()
            && self.group_binding_mode.is_none()
            && self.tag_filter_mode.is_none()
            && is_quit_key(&key)
        {
            return AppEvent::Quit;
        }

        if self.tag_input.is_none()
            && self.group_input.is_none()
            && self.editor.is_none()
            && self.tag_binding_mode.is_none()
            && self.group_binding_mode.is_none()
            && self.tag_filter_mode.is_none()
            && is_shift_char(&key, 'l')
        {
            self.toggle_debug_log_pane();
            return AppEvent::Continue;
        }

        if self.tag_input.is_none()
            && self.group_input.is_none()
            && self.editor.is_none()
            && self.tag_binding_mode.is_none()
            && self.group_binding_mode.is_none()
            && self.tag_filter_mode.is_none()
            && is_shift_char(&key, 'd')
        {
            self.cycle_desc_display_mode();
            return AppEvent::Continue;
        }

        if self.help_screen.is_some() {
            self.push_debug_log("route -> handle_help_key");
            self.handle_help_key(key);
            return AppEvent::Continue;
        }

        if self.tag_input.is_some() {
            self.push_debug_log("route -> handle_tag_input_key");
            self.handle_tag_input_key(key);
            return AppEvent::Continue;
        }

        if self.group_input.is_some() {
            self.push_debug_log("route -> handle_group_input_key");
            self.handle_group_input_key(key);
            return AppEvent::Continue;
        }

        if self.editor.is_some() {
            self.push_debug_log("route -> handle_editor_key");
            self.handle_editor_key(key);
            return AppEvent::Continue;
        }

        if self.tag_binding_mode.is_some() {
            self.push_debug_log("route -> handle_tag_binding_mode_key");
            self.handle_tag_binding_mode_key(key);
            return AppEvent::Continue;
        }

        if self.group_binding_mode.is_some() {
            self.push_debug_log("route -> handle_group_binding_mode_key");
            self.handle_group_binding_mode_key(key);
            return AppEvent::Continue;
        }

        if self.tag_filter_mode.is_some() {
            self.push_debug_log("route -> handle_tag_filter_mode_key");
            self.handle_tag_filter_mode_key(key);
            return AppEvent::Continue;
        }

        if self.tag_manager.is_some() {
            self.push_debug_log("route -> handle_tag_manager_key");
            self.handle_tag_manager_key(key);
            return AppEvent::Continue;
        }

        if self.group_manager.is_some() {
            self.push_debug_log("route -> handle_group_manager_key");
            self.handle_group_manager_key(key);
            return AppEvent::Continue;
        }

        if is_plain_or_ctrl_char(&key, 'e') {
            self.start_short_desc_edit();
            return AppEvent::Continue;
        }
        if is_plain_or_ctrl_char(&key, 'l') {
            self.start_long_desc_edit();
            return AppEvent::Continue;
        }
        if is_shift_char(&key, 't') {
            self.open_tag_manager();
            return AppEvent::Continue;
        }
        if is_shift_char(&key, 'g') {
            self.open_group_manager();
            return AppEvent::Continue;
        }
        if is_plain_or_ctrl_char(&key, 'n') {
            self.begin_new_tag_input();
            return AppEvent::Continue;
        }
        if is_ctrl_char(&key, 'g') {
            self.begin_new_group_input();
            return AppEvent::Continue;
        }
        if is_plain_or_ctrl_char(&key, 'r') {
            self.refresh_from_github();
            return AppEvent::Continue;
        }
        if is_plain_or_ctrl_char(&key, 's') {
            self.toggle_sort_mode();
            return AppEvent::Continue;
        }

        match key.code {
            KeyCode::Char('?') => {
                self.help_screen = Some(HelpScreen::Main);
                AppEvent::Continue
            }
            KeyCode::Char('t') => {
                self.begin_tag_binding_mode();
                AppEvent::Continue
            }
            KeyCode::Char('g') => {
                self.begin_group_binding_mode();
                AppEvent::Continue
            }
            KeyCode::Char('/') => {
                self.begin_tag_filter_mode();
                AppEvent::Continue
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                AppEvent::Continue
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                AppEvent::Continue
            }
            KeyCode::Left => {
                self.prev_registered_tag_page();
                AppEvent::Continue
            }
            KeyCode::Right => {
                self.next_registered_tag_page();
                AppEvent::Continue
            }
            _ => AppEvent::Continue,
        }
    }
}
