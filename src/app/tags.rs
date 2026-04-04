use super::{
    helpers::{describe_key_code, normalize_tags},
    App, HelpScreen, TagBindingMode, TagInput, TagInputMode, TagManager,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::BTreeSet;

impl App {
    pub(crate) fn begin_tag_binding_mode(&mut self) {
        if !self.has_registered_tags() {
            self.status_message = "登録済みtag がまだありません。n で作成できます".to_string();
            self.push_debug_log("begin_tag_binding_mode failed: no registered tags");
            return;
        }

        let Some(index) = self.selected_repo_data_index() else {
            self.status_message = "tag編集対象のrepoがありません".to_string();
            self.push_debug_log("begin_tag_binding_mode failed: no selected repo");
            return;
        };
        let Some(repo) = self.data.repos.get(index) else {
            self.status_message = "tag編集対象のrepoがありません".to_string();
            self.push_debug_log("begin_tag_binding_mode failed: selected repo index missing");
            return;
        };

        let pending_tags = repo.tags.iter().cloned().collect::<BTreeSet<_>>();
        self.help_screen = None;
        self.tag_binding_mode = Some(TagBindingMode {
            repo_index: index,
            repo_name: repo.name.clone(),
            original_tags: pending_tags.clone(),
            pending_tags,
        });
        self.status_message = "tag紐付けモード開始: ? で専用help".to_string();
        self.push_debug_log(format!(
            "tag_binding_mode opened: repo={} page={}",
            repo.name,
            self.registered_tag_page + 1
        ));
    }

    pub(crate) fn handle_help_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_help_key: code={} screen={:?}",
            describe_key_code(&key.code),
            self.help_screen
        ));
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                self.help_screen = None;
                self.push_debug_log("help closed");
            }
            _ => {
                self.push_debug_log("help key ignored");
            }
        }
    }

    pub(crate) fn handle_tag_binding_mode_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_tag_binding_mode_key: code={} page={}",
            describe_key_code(&key.code),
            self.registered_tag_page + 1
        ));
        match key.code {
            KeyCode::Char('?') => {
                self.help_screen = Some(HelpScreen::TagBinding);
                self.push_debug_log("tag_binding help opened");
            }
            KeyCode::Enter => self.confirm_tag_binding_mode(),
            KeyCode::Esc => self.cancel_tag_binding_mode(),
            KeyCode::Left => self.prev_registered_tag_page(),
            KeyCode::Right => self.next_registered_tag_page(),
            KeyCode::Char(ch) if ch.is_ascii_alphabetic() => self.handle_tag_shortcut(ch),
            _ => {
                self.push_debug_log("tag_binding_mode key ignored");
            }
        }
    }

    pub(crate) fn handle_tag_shortcut(&mut self, ch: char) {
        let Some(tag) = self.tag_for_current_page_shortcut(ch) else {
            if !self.has_registered_tags() {
                self.status_message = "登録済みtag がまだありません。n で作成できます".to_string();
            }
            return;
        };

        let Some(mode) = self.tag_binding_mode.as_mut() else {
            self.push_debug_log("handle_tag_shortcut ignored: mode inactive");
            return;
        };

        if ch.is_ascii_uppercase() {
            if mode.pending_tags.remove(&tag) {
                self.status_message = format!("tag削除候補: {tag}");
            } else {
                self.status_message = format!("tag未割当: {tag}");
            }
        } else if mode.pending_tags.insert(tag.clone()) {
            self.status_message = format!("tag追加候補: {tag}");
        } else {
            self.status_message = format!("tag追加済み: {tag}");
        }
    }

    pub(crate) fn cancel_tag_binding_mode(&mut self) {
        self.help_screen = None;
        self.tag_binding_mode = None;
        self.status_message = "tag紐付けをキャンセル".to_string();
        self.push_debug_log("tag_binding_mode cancelled");
    }

    pub(crate) fn confirm_tag_binding_mode(&mut self) {
        let Some(mode) = self.tag_binding_mode.as_ref().cloned() else {
            self.push_debug_log("confirm_tag_binding_mode skipped: mode inactive");
            return;
        };

        if mode.pending_tags == mode.original_tags {
            self.help_screen = None;
            self.tag_binding_mode = None;
            self.status_message = "tag変更なし".to_string();
            self.push_debug_log("tag_binding_mode confirmed: no changes");
            return;
        }

        let next_tags = mode.pending_tags.iter().cloned().collect::<Vec<_>>();
        let Some(repo) = self.data.repos.get_mut(mode.repo_index) else {
            self.help_screen = None;
            self.tag_binding_mode = None;
            self.status_message = "tag編集対象のrepoがありません".to_string();
            self.push_debug_log("confirm_tag_binding_mode failed: selected repo index missing");
            return;
        };
        let previous_tags = repo.tags.clone();
        repo.tags = next_tags;

        match self.persist_data() {
            Ok(()) => {
                self.help_screen = None;
                self.tag_binding_mode = None;
                self.sync_selection();
                self.status_message = format!("tag更新: {}", mode.repo_name);
                self.push_debug_log(format!(
                    "tag_binding_mode confirmed and persisted: repo={}",
                    mode.repo_name
                ));
            }
            Err(error) => {
                if let Some(repo) = self.data.repos.get_mut(mode.repo_index) {
                    repo.tags = previous_tags;
                }
                self.status_message = format!("保存失敗: {error}");
                self.push_debug_log(format!(
                    "tag_binding_mode persist failed: repo={} error={error}",
                    mode.repo_name
                ));
            }
        }
    }

    pub(crate) fn open_tag_manager(&mut self) {
        self.tag_manager = Some(TagManager { selected: 0 });
        self.status_message = "tag manager: j/k移動 n新規 r改名 Escで閉じる".to_string();
        self.push_debug_log(format!(
            "open_tag_manager: registered_tags={}",
            self.registered_tags().len()
        ));
    }

    pub(crate) fn handle_tag_manager_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_tag_manager_key: code={} registered_tags={}",
            describe_key_code(&key.code),
            self.registered_tags().len()
        ));
        match key.code {
            KeyCode::Esc | KeyCode::Char('t') => {
                self.tag_manager = None;
                self.status_message = "tag manager を閉じました".to_string();
                self.push_debug_log("tag_manager closed");
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.registered_tags().len();
                if let Some(manager) = self.tag_manager.as_mut() {
                    manager.selected = if len == 0 {
                        0
                    } else {
                        manager.selected.saturating_add(1).min(len - 1)
                    };
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = self.registered_tags().len();
                if let Some(manager) = self.tag_manager.as_mut() {
                    manager.selected = if len == 0 {
                        0
                    } else {
                        manager.selected.saturating_sub(1)
                    };
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => self.begin_new_registered_tag_input(),
            KeyCode::Enter if self.registered_tags().is_empty() => {
                self.begin_new_registered_tag_input()
            }
            KeyCode::Char('r') => {
                let Some(from) = self.current_tag_manager_tag() else {
                    self.status_message = "rename対象のtagがありません".to_string();
                    return;
                };
                self.tag_input = Some(TagInput {
                    mode: TagInputMode::RenameGlobal { from: from.clone() },
                    buffer: from,
                });
                self.status_message = "tag rename: Enterで保存 / Escでキャンセル".to_string();
                self.push_debug_log("tag rename overlay opened");
            }
            _ => {
                self.push_debug_log("tag_manager key ignored");
            }
        }
    }

    pub(crate) fn begin_new_tag_input(&mut self) {
        self.tag_input = Some(TagInput {
            mode: TagInputMode::CreateAndAssignToSelectedRepo,
            buffer: String::new(),
        });
        self.status_message = "新規tag: Enterで保存 / Escでキャンセル".to_string();
        self.push_debug_log("tag_input overlay opened: CreateAndAssignToSelectedRepo");
    }

    pub(crate) fn begin_new_registered_tag_input(&mut self) {
        self.tag_input = Some(TagInput {
            mode: TagInputMode::CreateRegisteredOnly,
            buffer: String::new(),
        });
        self.status_message = "新規tag: Enterで保存 / Escでキャンセル".to_string();
        self.push_debug_log("tag_input overlay opened: CreateRegisteredOnly");
    }

    pub(crate) fn handle_tag_input_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_tag_input_key: code={} buffer_len={}",
            describe_key_code(&key.code),
            self.tag_input
                .as_ref()
                .map(|input| input.buffer.chars().count())
                .unwrap_or(0)
        ));
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            self.save_tag_input();
            return;
        }

        match key.code {
            KeyCode::Enter => self.save_tag_input(),
            KeyCode::Esc => {
                self.tag_input = None;
                self.status_message = "tag入力をキャンセル".to_string();
                self.push_debug_log("tag_input cancelled");
            }
            KeyCode::Backspace => {
                if let Some(input) = self.tag_input.as_mut() {
                    input.buffer.pop();
                }
                self.push_debug_log("tag_input backspace");
            }
            KeyCode::Char(ch) => {
                if let Some(input) = self.tag_input.as_mut() {
                    input.buffer.push(ch);
                }
                self.push_debug_log(format!("tag_input append: {:?}", ch));
            }
            _ => {
                self.push_debug_log("tag_input key ignored");
            }
        }
    }

    pub(crate) fn save_tag_input(&mut self) {
        let Some(input) = self.tag_input.as_ref().cloned() else {
            self.push_debug_log("save_tag_input skipped: no input");
            return;
        };
        let value = input.buffer.trim().to_string();
        self.push_debug_log(format!("save_tag_input: trimmed_value={:?}", value));
        if value.is_empty() {
            self.status_message = "tagは空にできません".to_string();
            self.push_debug_log("save_tag_input rejected: empty");
            return;
        }
        self.tag_input = None;

        match input.mode {
            TagInputMode::CreateAndAssignToSelectedRepo => {
                let Some(index) = self.selected_repo_data_index_mut() else {
                    self.status_message = "tag追加対象のrepoがありません".to_string();
                    self.push_debug_log("save_tag_input failed: no selected repo");
                    return;
                };
                self.ensure_registered_tag(&value);
                if !self.data.repos[index].tags.iter().any(|tag| tag == &value) {
                    self.data.repos[index].tags.push(value.clone());
                    normalize_tags(&mut self.data.repos[index].tags);
                }
                match self.persist_data() {
                    Ok(()) => {
                        self.sync_selection();
                        let selected = self
                            .registered_tags()
                            .iter()
                            .position(|tag| tag == &value)
                            .unwrap_or(0);
                        if let Some(manager) = self.tag_manager.as_mut() {
                            manager.selected = selected;
                        }
                        self.status_message = format!("tag追加: {value}");
                        self.push_debug_log(format!("tag added and persisted: {:?}", value));
                    }
                    Err(error) => {
                        self.status_message = format!("保存失敗: {error}");
                        self.push_debug_log(format!("persist failed after tag add: {error}"));
                    }
                }
            }
            TagInputMode::CreateRegisteredOnly => {
                self.ensure_registered_tag(&value);
                match self.persist_data() {
                    Ok(()) => {
                        self.sync_selection();
                        let selected = self
                            .registered_tags()
                            .iter()
                            .position(|tag| tag == &value)
                            .unwrap_or(0);
                        if let Some(manager) = self.tag_manager.as_mut() {
                            manager.selected = selected;
                        }
                        self.status_message = format!("tag追加(global): {value}");
                        self.push_debug_log(format!(
                            "registered tag added and persisted: {:?}",
                            value
                        ));
                    }
                    Err(error) => {
                        self.status_message = format!("保存失敗: {error}");
                        self.push_debug_log(format!(
                            "persist failed after registered tag add: {error}"
                        ));
                    }
                }
            }
            TagInputMode::RenameGlobal { from } => {
                for registered_tag in &mut self.data.registered_tags {
                    if *registered_tag == from {
                        *registered_tag = value.clone();
                    }
                }
                normalize_tags(&mut self.data.registered_tags);
                for repo in &mut self.data.repos {
                    for tag in &mut repo.tags {
                        if *tag == from {
                            *tag = value.clone();
                        }
                    }
                    normalize_tags(&mut repo.tags);
                }
                match self.persist_data() {
                    Ok(()) => {
                        self.sync_selection();
                        let selected = self
                            .registered_tags()
                            .iter()
                            .position(|tag| tag == &value)
                            .unwrap_or(0);
                        if let Some(manager) = self.tag_manager.as_mut() {
                            manager.selected = selected;
                        }
                        self.status_message = format!("tag rename: {from} -> {value}");
                        self.push_debug_log(format!(
                            "tag renamed and persisted: {:?} -> {:?}",
                            from, value
                        ));
                    }
                    Err(error) => {
                        self.status_message = format!("保存失敗: {error}");
                        self.push_debug_log(format!("persist failed after tag rename: {error}"));
                    }
                }
            }
        }
    }

    pub(crate) fn current_tag_manager_tag(&self) -> Option<String> {
        let registered_tags = self.registered_tags();
        let selected = self.tag_manager.as_ref()?.selected;
        registered_tags.get(selected).cloned()
    }

    pub(crate) fn ensure_registered_tag(&mut self, tag: &str) {
        if self
            .data
            .registered_tags
            .iter()
            .any(|registered| registered == tag)
        {
            return;
        }
        self.data.registered_tags.push(tag.to_string());
        normalize_tags(&mut self.data.registered_tags);
    }
}
