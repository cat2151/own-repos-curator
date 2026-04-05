use super::editor::{new_single_line_textarea, normalize_single_line_textarea, textarea_text};
use crossterm::event::KeyEvent;
use std::collections::BTreeSet;
use tui_textarea::TextArea;

#[derive(Clone, Copy)]
pub(crate) enum EditorField {
    ShortDesc,
    LongDesc,
}

#[derive(Clone)]
pub(crate) struct TextEditor {
    pub(crate) field: EditorField,
    pub(crate) textarea: TextArea<'static>,
}

#[derive(Clone)]
pub(crate) struct TagManager {
    pub(crate) selected: usize,
}

#[derive(Clone)]
pub(crate) struct TagInput {
    pub(crate) mode: TagInputMode,
    pub(crate) textarea: TextArea<'static>,
}

#[derive(Clone)]
pub(crate) struct TagBindingMode {
    pub(crate) repo_index: usize,
    pub(crate) repo_name: String,
    pub(crate) original_tags: BTreeSet<String>,
    pub(crate) pending_tags: BTreeSet<String>,
}

#[derive(Clone)]
pub(crate) struct GroupManager {
    pub(crate) selected: usize,
}

#[derive(Clone)]
pub(crate) struct GroupInput {
    pub(crate) mode: GroupInputMode,
    pub(crate) textarea: TextArea<'static>,
}

#[derive(Clone)]
pub(crate) struct GroupBindingMode {
    pub(crate) repo_index: usize,
    pub(crate) repo_name: String,
    pub(crate) original_group: String,
    pub(crate) pending_group: String,
}

#[derive(Clone)]
pub(crate) struct FilterMode {
    pub(crate) focus: FilterModeFocus,
    pub(crate) original_group: Option<String>,
    pub(crate) pending_group: Option<String>,
    pub(crate) original_tags: BTreeSet<String>,
    pub(crate) pending_tags: BTreeSet<String>,
    pub(crate) selected_repo_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterModeFocus {
    Group,
    Tag,
}

#[derive(Clone)]
pub(crate) enum TagInputMode {
    CreateAndAssignToSelectedRepo,
    CreateRegisteredOnly,
    RenameGlobal { from: String },
}

#[derive(Clone)]
pub(crate) enum GroupInputMode {
    CreateAndAssignToSelectedRepo,
    CreateRegisteredOnly,
    RenameGlobal { from: String },
}

impl TagInput {
    pub(crate) fn new(mode: TagInputMode, initial_text: &str) -> Self {
        Self {
            mode,
            textarea: new_single_line_textarea(initial_text),
        }
    }

    pub(crate) fn value(&self) -> String {
        textarea_text(&self.textarea, " ")
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        self.textarea.input(key);
        normalize_single_line_textarea(&mut self.textarea);
    }
}

impl GroupInput {
    pub(crate) fn new(mode: GroupInputMode, initial_text: &str) -> Self {
        Self {
            mode,
            textarea: new_single_line_textarea(initial_text),
        }
    }

    pub(crate) fn value(&self) -> String {
        textarea_text(&self.textarea, " ")
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        self.textarea.input(key);
        normalize_single_line_textarea(&mut self.textarea);
    }
}

pub(crate) struct TagCatalogState {
    pub(crate) entries: Vec<TagCatalogEntry>,
    pub(crate) page: usize,
    pub(crate) page_count: usize,
    pub(crate) total_tags: usize,
    pub(crate) active_filter_count: usize,
    pub(crate) filter_mode_active: bool,
}

pub(crate) struct GroupCatalogState {
    pub(crate) entries: Vec<GroupCatalogEntry>,
    pub(crate) page: usize,
    pub(crate) page_count: usize,
    pub(crate) total_groups: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagBinding {
    pub(crate) key: char,
    pub(crate) tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupBinding {
    pub(crate) key: char,
    pub(crate) group: String,
}

pub(crate) struct TagCatalogEntry {
    pub(crate) key: char,
    pub(crate) filter_active: bool,
    pub(crate) tag: String,
}

pub(crate) struct GroupCatalogEntry {
    pub(crate) key: char,
    pub(crate) selected: bool,
    pub(crate) group: String,
}

pub(crate) struct TagManagerState {
    pub(crate) entries: Vec<TagManagerEntry>,
    pub(crate) selected: usize,
}

pub(crate) struct GroupManagerState {
    pub(crate) entries: Vec<GroupManagerEntry>,
    pub(crate) selected: usize,
}

pub(crate) struct TagManagerEntry {
    pub(crate) tag: String,
}

pub(crate) struct GroupManagerEntry {
    pub(crate) group: String,
}

pub(crate) struct TagBindingModeState {
    pub(crate) repo_name: String,
    pub(crate) pending_count: usize,
    pub(crate) added_tags: Vec<String>,
    pub(crate) removed_tags: Vec<String>,
}

pub(crate) struct GroupBindingModeState {
    pub(crate) repo_name: String,
    pub(crate) original_group: String,
    pub(crate) pending_group: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FilterModeState {
    pub(crate) focus: FilterModeFocus,
    pub(crate) active_group: Option<String>,
    pub(crate) active_tags: Vec<String>,
    pub(crate) visible_repo_count: usize,
    pub(crate) total_repo_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupSummaryEntry {
    pub(crate) group: String,
    pub(crate) count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedRepoTagDetailState {
    pub(crate) repo_name: String,
    pub(crate) tag_count: usize,
    pub(crate) entries: Vec<SelectedRepoTagDetailEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedRepoTagDetailEntry {
    pub(crate) key_hint: String,
    pub(crate) tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedRepoDescState {
    pub(crate) repo_name: String,
    pub(crate) github_desc: String,
    pub(crate) desc_short: String,
    pub(crate) desc_long: String,
    pub(crate) group: String,
    pub(crate) group_key_hint: String,
}
