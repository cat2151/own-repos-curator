use super::{helpers::describe_key_code, App, FilterMode, FilterModeFocus, HelpScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::BTreeSet;

impl App {
    pub(crate) fn begin_filter_mode(&mut self) {
        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
        let pending_tags = self.tag_filter.clone();
        self.help_screen = None;
        self.filter_mode = Some(FilterMode {
            focus: FilterModeFocus::Group,
            original_group: self.group_filter.clone(),
            pending_group: self.group_filter.clone(),
            original_tags: pending_tags.clone(),
            pending_tags,
            selected_repo_name,
        });
        self.status_message = filter_focus_status_message(FilterModeFocus::Group);
        self.push_debug_log(format!(
            "filter_mode opened: group={:?} tags={}",
            self.active_group_filter(),
            self.active_tag_filter_count()
        ));
    }

    pub(crate) fn handle_filter_mode_key(&mut self, key: KeyEvent) {
        let focus = self
            .filter_mode
            .as_ref()
            .map(|mode| mode.focus)
            .unwrap_or(FilterModeFocus::Group);
        self.push_debug_log(format!(
            "handle_filter_mode_key: code={} focus={focus:?} tag_page={} group_page={}",
            describe_key_code(&key.code),
            self.registered_tag_page + 1,
            self.registered_group_page + 1
        ));

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    self.switch_filter_focus(FilterModeFocus::Tag);
                    return;
                }
                KeyCode::Char('g') | KeyCode::Char('G') => {
                    self.switch_filter_focus(FilterModeFocus::Group);
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Enter => self.confirm_filter_mode(),
            KeyCode::Esc => self.cancel_filter_mode(),
            KeyCode::Char('?') => {
                self.help_screen = Some(HelpScreen::Filter);
                self.push_debug_log("filter help opened");
            }
            KeyCode::Left => match focus {
                FilterModeFocus::Group => self.prev_registered_group_page(),
                FilterModeFocus::Tag => self.prev_registered_tag_page(),
            },
            KeyCode::Right => match focus {
                FilterModeFocus::Group => self.next_registered_group_page(),
                FilterModeFocus::Tag => self.next_registered_tag_page(),
            },
            KeyCode::Char(ch) if ch.is_ascii_alphabetic() => match focus {
                FilterModeFocus::Group => self.handle_group_filter_shortcut(ch),
                FilterModeFocus::Tag => self.handle_tag_filter_shortcut(ch),
            },
            _ => {
                self.push_debug_log("filter_mode key ignored");
            }
        }
    }

    pub(crate) fn effective_tag_filter(&self) -> &BTreeSet<String> {
        self.filter_mode
            .as_ref()
            .map(|mode| &mode.pending_tags)
            .unwrap_or(&self.tag_filter)
    }

    pub(crate) fn effective_group_filter(&self) -> Option<&str> {
        self.filter_mode
            .as_ref()
            .and_then(|mode| mode.pending_group.as_deref())
            .or(self.group_filter.as_deref())
    }

    pub(crate) fn active_tag_filter_count(&self) -> usize {
        self.effective_tag_filter().len()
    }

    pub(crate) fn active_tag_filter_tags(&self) -> Vec<String> {
        self.effective_tag_filter().iter().cloned().collect()
    }

    pub(crate) fn active_group_filter(&self) -> Option<String> {
        self.effective_group_filter().map(str::to_string)
    }

    pub(crate) fn filter_mode_active(&self) -> bool {
        self.filter_mode.is_some()
    }

    pub(crate) fn has_effective_filter(&self) -> bool {
        self.effective_group_filter().is_some() || !self.effective_tag_filter().is_empty()
    }

    pub(crate) fn tag_filter_title_label(&self) -> String {
        let prefix = if self.filter_mode.is_some() {
            "filter*"
        } else {
            "filter"
        };
        let group_filter = self.active_group_filter();
        let tag_count = self.active_tag_filter_count();
        if group_filter.is_none() && tag_count == 0 {
            format!("{prefix}:off")
        } else {
            let mut parts = Vec::new();
            if let Some(group) = group_filter {
                parts.push(format!("g={group}"));
            }
            if tag_count > 0 {
                parts.push(format!("t={tag_count}"));
            }
            format!("{prefix}:{}", parts.join(","))
        }
    }

    fn switch_filter_focus(&mut self, focus: FilterModeFocus) {
        let Some(mode) = self.filter_mode.as_mut() else {
            self.push_debug_log("switch_filter_focus ignored: mode inactive");
            return;
        };

        let changed = mode.focus != focus;
        mode.focus = focus;
        self.status_message = filter_focus_status_message(focus);
        if changed {
            self.push_debug_log(format!("filter_mode focus switched: {focus:?}"));
        }
    }

    fn handle_tag_filter_shortcut(&mut self, ch: char) {
        let Some(tag) = self.tag_for_current_page_shortcut(ch) else {
            if !self.has_registered_tags() {
                self.status_message = "登録済みtag がまだありません。n で作成できます".to_string();
            }
            self.push_debug_log(format!(
                "tag_filter shortcut ignored: no binding for {}",
                ch
            ));
            return;
        };

        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
        let Some(mode) = self.filter_mode.as_mut() else {
            self.push_debug_log("handle_tag_filter_shortcut ignored: mode inactive");
            return;
        };

        if ch.is_ascii_uppercase() {
            if mode.pending_tags.remove(&tag) {
                self.status_message = format!("tag絞り込み候補 OFF: {tag}");
            } else {
                self.status_message = format!("tag絞り込み候補 OFF済み: {tag}");
            }
        } else if mode.pending_tags.insert(tag.clone()) {
            self.status_message = format!("tag絞り込み候補 ON: {tag}");
        } else {
            self.status_message = format!("tag絞り込み候補 ON済み: {tag}");
        }

        self.restore_selection(selected_repo_name.as_deref());
    }

    fn handle_group_filter_shortcut(&mut self, ch: char) {
        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());

        if ch.is_ascii_uppercase() {
            let Some(mode) = self.filter_mode.as_mut() else {
                self.push_debug_log("handle_group_filter_shortcut ignored: mode inactive");
                return;
            };
            if mode.pending_group.take().is_some() {
                self.status_message = "group絞り込み候補 解除".to_string();
            } else {
                self.status_message = "group絞り込み候補 解除済み".to_string();
            }
            self.restore_selection(selected_repo_name.as_deref());
            return;
        }

        let Some(group) = self.group_for_current_page_shortcut(ch) else {
            if !self.has_registered_groups() {
                self.status_message =
                    "登録済みgroup がまだありません。Ctrl+G で作成できます".to_string();
            }
            self.push_debug_log(format!(
                "group_filter shortcut ignored: no binding for {}",
                ch
            ));
            return;
        };

        let Some(mode) = self.filter_mode.as_mut() else {
            self.push_debug_log("handle_group_filter_shortcut ignored: mode inactive");
            return;
        };
        mode.pending_group = Some(group.clone());
        self.status_message = format!("group絞り込み候補: {group}");
        self.restore_selection(selected_repo_name.as_deref());
    }

    pub(crate) fn cancel_filter_mode(&mut self) {
        let Some(mode) = self.filter_mode.take() else {
            self.push_debug_log("cancel_filter_mode skipped: mode inactive");
            return;
        };

        self.restore_selection(mode.selected_repo_name.as_deref());
        self.status_message = "絞り込みをキャンセル".to_string();
        self.push_debug_log("filter_mode cancelled");
    }

    pub(crate) fn confirm_filter_mode(&mut self) {
        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
        let Some(mode) = self.filter_mode.take() else {
            self.push_debug_log("confirm_filter_mode skipped: mode inactive");
            return;
        };

        let changed =
            mode.pending_group != mode.original_group || mode.pending_tags != mode.original_tags;
        self.group_filter = mode.pending_group;
        self.tag_filter = mode.pending_tags;
        self.restore_selection(selected_repo_name.as_deref());
        self.status_message =
            filter_status_message(changed, self.group_filter.as_deref(), &self.tag_filter);
        self.push_debug_log(format!(
            "filter_mode confirmed: group={:?} tags={}",
            self.group_filter,
            self.tag_filter.len()
        ));
    }
}

fn filter_focus_status_message(focus: FilterModeFocus) -> String {
    match focus {
        FilterModeFocus::Group => {
            "group絞り込みモード: a-z 選択 / A-Z 解除 / Ctrl+Tでtag / Enter確定 / Esc取消"
                .to_string()
        }
        FilterModeFocus::Tag => {
            "tag絞り込みモード: a-z ON / A-Z OFF / Ctrl+Gでgroup / Enter確定 / Esc取消".to_string()
        }
    }
}

fn filter_status_message(
    changed: bool,
    group_filter: Option<&str>,
    tag_filter: &BTreeSet<String>,
) -> String {
    if !changed {
        return "絞り込み変更なし".to_string();
    }
    if group_filter.is_none() && tag_filter.is_empty() {
        return "絞り込み解除".to_string();
    }

    let mut parts = Vec::new();
    if let Some(group) = group_filter {
        parts.push(format!("group={group}"));
    }
    if !tag_filter.is_empty() {
        let tags = tag_filter.iter().cloned().collect::<Vec<_>>();
        parts.push(format!("tags={}", tags.join(", ")));
    }
    format!("絞り込み更新: {}", parts.join(" / "))
}
