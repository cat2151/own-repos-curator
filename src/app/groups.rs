use super::{
    helpers::describe_key_code, App, GroupBindingMode, GroupInput, GroupInputMode, GroupManager,
    HelpScreen,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::BTreeSet;

impl App {
    pub(crate) fn begin_group_binding_mode(&mut self) {
        if !self.has_registered_groups() {
            self.status_message =
                "登録済みgroup がまだありません。Ctrl+G で作成できます".to_string();
            self.push_debug_log("begin_group_binding_mode failed: no registered groups");
            return;
        }

        let Some(index) = self.selected_repo_data_index() else {
            self.status_message = "group編集対象のrepoがありません".to_string();
            self.push_debug_log("begin_group_binding_mode failed: no selected repo");
            return;
        };
        let Some(repo) = self.data.repos.get(index) else {
            self.status_message = "group編集対象のrepoがありません".to_string();
            self.push_debug_log("begin_group_binding_mode failed: selected repo index missing");
            return;
        };

        self.help_screen = None;
        self.group_binding_mode = Some(GroupBindingMode {
            repo_index: index,
            repo_name: repo.name.clone(),
            original_group: repo.group.clone(),
            pending_group: repo.group.clone(),
        });
        self.status_message = "group割り当てモード開始: ? で専用help".to_string();
        self.push_debug_log(format!(
            "group_binding_mode opened: repo={} page={}",
            repo.name,
            self.registered_group_page + 1
        ));
    }

    pub(crate) fn handle_group_binding_mode_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_group_binding_mode_key: code={} page={}",
            describe_key_code(&key.code),
            self.registered_group_page + 1
        ));
        match key.code {
            KeyCode::Char('?') => {
                self.help_screen = Some(HelpScreen::GroupBinding);
                self.push_debug_log("group_binding help opened");
            }
            KeyCode::Esc => self.cancel_group_binding_mode(),
            KeyCode::Left => self.prev_registered_group_page(),
            KeyCode::Right => self.next_registered_group_page(),
            KeyCode::Char(ch) if ch.is_ascii_alphabetic() => self.handle_group_shortcut(ch),
            _ => {
                self.push_debug_log("group_binding_mode key ignored");
            }
        }
    }

    pub(crate) fn handle_group_shortcut(&mut self, ch: char) {
        let Some(group) = self.group_for_current_page_shortcut(ch) else {
            if !self.has_registered_groups() {
                self.status_message =
                    "登録済みgroup がまだありません。Ctrl+G で作成できます".to_string();
            }
            return;
        };

        {
            let Some(mode) = self.group_binding_mode.as_mut() else {
                self.push_debug_log("handle_group_shortcut ignored: mode inactive");
                return;
            };
            mode.pending_group = group.clone();
        }

        self.push_debug_log(format!("group shortcut selected: {group}"));
        self.confirm_group_binding_mode();
    }

    pub(crate) fn cancel_group_binding_mode(&mut self) {
        self.help_screen = None;
        self.group_binding_mode = None;
        self.status_message = "group割り当てをキャンセル".to_string();
        self.push_debug_log("group_binding_mode cancelled");
    }

    pub(crate) fn confirm_group_binding_mode(&mut self) {
        let Some(mode) = self.group_binding_mode.as_ref().cloned() else {
            self.push_debug_log("confirm_group_binding_mode skipped: mode inactive");
            return;
        };

        if mode.pending_group == mode.original_group {
            self.help_screen = None;
            self.group_binding_mode = None;
            self.status_message = "group変更なし".to_string();
            self.push_debug_log("group_binding_mode confirmed: no changes");
            return;
        }

        self.data.ensure_registered_group(&mode.pending_group);
        let Some(repo) = self.data.repos.get_mut(mode.repo_index) else {
            self.help_screen = None;
            self.group_binding_mode = None;
            self.status_message = "group編集対象のrepoがありません".to_string();
            self.push_debug_log("confirm_group_binding_mode failed: selected repo index missing");
            return;
        };
        let previous_group = repo.group.clone();
        repo.group = mode.pending_group.clone();

        match self.persist_data() {
            Ok(()) => {
                self.help_screen = None;
                self.group_binding_mode = None;
                self.sync_selection();
                self.status_message = format!("group更新: {}", mode.repo_name);
                self.push_debug_log(format!(
                    "group_binding_mode confirmed and persisted: repo={}",
                    mode.repo_name
                ));
            }
            Err(error) => {
                if let Some(repo) = self.data.repos.get_mut(mode.repo_index) {
                    repo.group = previous_group;
                }
                self.status_message = format!("保存失敗: {error}");
                self.push_debug_log(format!(
                    "group_binding_mode persist failed: repo={} error={error}",
                    mode.repo_name
                ));
            }
        }
    }

    pub(crate) fn open_group_manager(&mut self) {
        let selected = self
            .selected_repo()
            .and_then(|repo| {
                self.registered_groups()
                    .iter()
                    .position(|group| group == &repo.group)
            })
            .unwrap_or(0);
        self.group_manager = Some(GroupManager { selected });
        self.status_message = "group manager: j/k移動 n新規 r改名 Escで閉じる".to_string();
        self.push_debug_log(format!(
            "open_group_manager: registered_groups={}",
            self.registered_groups().len()
        ));
    }

    pub(crate) fn handle_group_manager_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_group_manager_key: code={} registered_groups={}",
            describe_key_code(&key.code),
            self.registered_groups().len()
        ));
        match key.code {
            KeyCode::Esc | KeyCode::Char('g') => {
                self.group_manager = None;
                self.status_message = "group manager を閉じました".to_string();
                self.push_debug_log("group_manager closed");
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.registered_groups().len();
                if let Some(manager) = self.group_manager.as_mut() {
                    manager.selected = if len == 0 {
                        0
                    } else {
                        manager.selected.saturating_add(1).min(len - 1)
                    };
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = self.registered_groups().len();
                if let Some(manager) = self.group_manager.as_mut() {
                    manager.selected = if len == 0 {
                        0
                    } else {
                        manager.selected.saturating_sub(1)
                    };
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => self.begin_new_registered_group_input(),
            KeyCode::Enter if self.registered_groups().is_empty() => {
                self.begin_new_registered_group_input()
            }
            KeyCode::Char('r') => {
                let Some(from) = self.current_group_manager_group() else {
                    self.status_message = "rename対象のgroupがありません".to_string();
                    return;
                };
                self.group_input = Some(GroupInput::new(
                    GroupInputMode::RenameGlobal { from: from.clone() },
                    &from,
                ));
                self.status_message = "group rename: Enterで保存 / Escでキャンセル".to_string();
                self.push_debug_log("group rename overlay opened");
            }
            _ => {
                self.push_debug_log("group_manager key ignored");
            }
        }
    }

    pub(crate) fn begin_new_group_input(&mut self) {
        self.group_input = Some(GroupInput::new(
            GroupInputMode::CreateAndAssignToSelectedRepo,
            "",
        ));
        self.status_message = "新規group: Enterで保存 / Escでキャンセル".to_string();
        self.push_debug_log("group_input overlay opened: CreateAndAssignToSelectedRepo");
    }

    pub(crate) fn begin_new_registered_group_input(&mut self) {
        self.group_input = Some(GroupInput::new(GroupInputMode::CreateRegisteredOnly, ""));
        self.status_message = "新規group: Enterで保存 / Escでキャンセル".to_string();
        self.push_debug_log("group_input overlay opened: CreateRegisteredOnly");
    }

    pub(crate) fn handle_group_input_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_group_input_key: code={} buffer_len={}",
            describe_key_code(&key.code),
            self.group_input
                .as_ref()
                .map(|input| input.value().chars().count())
                .unwrap_or(0)
        ));
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            self.save_group_input();
            return;
        }

        match key.code {
            KeyCode::Enter => self.save_group_input(),
            KeyCode::Esc => {
                self.group_input = None;
                self.status_message = "group入力をキャンセル".to_string();
                self.push_debug_log("group_input cancelled");
            }
            _ => {
                let edited_value = if let Some(input) = self.group_input.as_mut() {
                    input.handle_key(key);
                    Some(input.value())
                } else {
                    None
                };
                if let Some(value) = edited_value {
                    self.push_debug_log(format!("group_input edited: {:?}", value));
                }
            }
        }
    }

    pub(crate) fn save_group_input(&mut self) {
        let Some(input) = self.group_input.as_ref().cloned() else {
            self.push_debug_log("save_group_input skipped: no input");
            return;
        };
        let value = input.value().trim().to_string();
        self.push_debug_log(format!("save_group_input: trimmed_value={:?}", value));
        if value.is_empty() {
            self.status_message = "groupは空にできません".to_string();
            self.push_debug_log("save_group_input rejected: empty");
            return;
        }
        self.group_input = None;

        match input.mode {
            GroupInputMode::CreateAndAssignToSelectedRepo => {
                let Some(index) = self.selected_repo_data_index_mut() else {
                    self.status_message = "group設定対象のrepoがありません".to_string();
                    self.push_debug_log("save_group_input failed: no selected repo");
                    return;
                };
                let repo_name = self.data.repos[index].name.clone();
                let previous_group = self.data.repos[index].group.clone();
                if previous_group == value
                    && self
                        .data
                        .registered_groups
                        .iter()
                        .any(|group| group == &value)
                {
                    self.status_message = "group変更なし".to_string();
                    self.push_debug_log("save_group_input skipped: selected repo already in group");
                    return;
                }

                self.ensure_registered_group(&value);
                self.data.repos[index].group = value.clone();
                match self.persist_data() {
                    Ok(()) => {
                        self.sync_selection();
                        self.select_group_in_manager(&value);
                        self.status_message = format!("group更新: {repo_name}");
                        self.push_debug_log(format!(
                            "group assigned and persisted: repo={} group={:?}",
                            repo_name, value
                        ));
                    }
                    Err(error) => {
                        self.data.repos[index].group = previous_group;
                        self.status_message = format!("保存失敗: {error}");
                        self.push_debug_log(format!(
                            "persist failed after group assign: repo={} error={error}",
                            repo_name
                        ));
                    }
                }
            }
            GroupInputMode::CreateRegisteredOnly => {
                self.ensure_registered_group(&value);
                match self.persist_data() {
                    Ok(()) => {
                        self.sync_selection();
                        self.select_group_in_manager(&value);
                        self.status_message = format!("group追加(global): {value}");
                        self.push_debug_log(format!(
                            "registered group added and persisted: {:?}",
                            value
                        ));
                    }
                    Err(error) => {
                        self.status_message = format!("保存失敗: {error}");
                        self.push_debug_log(format!(
                            "persist failed after registered group add: {error}"
                        ));
                    }
                }
            }
            GroupInputMode::RenameGlobal { from } => {
                for registered_group in &mut self.data.registered_groups {
                    if *registered_group == from {
                        *registered_group = value.clone();
                    }
                }
                normalize_groups(&mut self.data.registered_groups);
                for repo in &mut self.data.repos {
                    if repo.group == from {
                        repo.group = value.clone();
                    }
                }
                self.ensure_registered_group(&value);
                match self.persist_data() {
                    Ok(()) => {
                        self.sync_selection();
                        self.select_group_in_manager(&value);
                        self.status_message = format!("group rename: {from} -> {value}");
                        self.push_debug_log(format!(
                            "group renamed and persisted: {:?} -> {:?}",
                            from, value
                        ));
                    }
                    Err(error) => {
                        self.status_message = format!("保存失敗: {error}");
                        self.push_debug_log(format!("persist failed after group rename: {error}"));
                    }
                }
            }
        }
    }

    pub(crate) fn current_group_manager_group(&self) -> Option<String> {
        let registered_groups = self.registered_groups();
        let selected = self.group_manager.as_ref()?.selected;
        registered_groups.get(selected).cloned()
    }

    pub(crate) fn ensure_registered_group(&mut self, group: &str) {
        self.data.ensure_registered_group(group);
    }

    fn select_group_in_manager(&mut self, group: &str) {
        let selected = self
            .registered_groups()
            .iter()
            .position(|registered_group| registered_group == group)
            .unwrap_or(0);
        if let Some(manager) = self.group_manager.as_mut() {
            manager.selected = selected;
        }
    }
}

fn normalize_groups(groups: &mut Vec<String>) {
    *groups = groups
        .iter()
        .map(|group| group.trim())
        .filter(|group| !group.is_empty())
        .map(|group| group.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
}
