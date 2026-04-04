use super::{helpers::describe_key_code, App, TagFilterMode};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::BTreeSet;

impl App {
    pub(crate) fn begin_tag_filter_mode(&mut self) {
        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
        let pending_tags = self.tag_filter.clone();
        self.tag_filter_mode = Some(TagFilterMode {
            original_tags: pending_tags.clone(),
            pending_tags,
            selected_repo_name,
        });
        self.status_message =
            "tag絞り込みモード: a-z ON / A-Z OFF / Enter確定 / Esc取消".to_string();
        self.push_debug_log(format!(
            "tag_filter_mode opened: active_tags={}",
            self.active_tag_filter_count()
        ));
    }

    pub(crate) fn handle_tag_filter_mode_key(&mut self, key: KeyEvent) {
        self.push_debug_log(format!(
            "handle_tag_filter_mode_key: code={} page={}",
            describe_key_code(&key.code),
            self.registered_tag_page + 1
        ));
        match key.code {
            KeyCode::Enter => self.confirm_tag_filter_mode(),
            KeyCode::Esc => self.cancel_tag_filter_mode(),
            KeyCode::Left => self.prev_registered_tag_page(),
            KeyCode::Right => self.next_registered_tag_page(),
            KeyCode::Char(ch) if ch.is_ascii_alphabetic() => self.handle_tag_filter_shortcut(ch),
            _ => {
                self.push_debug_log("tag_filter_mode key ignored");
            }
        }
    }

    pub(crate) fn effective_tag_filter(&self) -> &BTreeSet<String> {
        self.tag_filter_mode
            .as_ref()
            .map(|mode| &mode.pending_tags)
            .unwrap_or(&self.tag_filter)
    }

    pub(crate) fn active_tag_filter_count(&self) -> usize {
        self.effective_tag_filter().len()
    }

    pub(crate) fn active_tag_filter_tags(&self) -> Vec<String> {
        self.effective_tag_filter().iter().cloned().collect()
    }

    pub(crate) fn tag_filter_mode_active(&self) -> bool {
        self.tag_filter_mode.is_some()
    }

    pub(crate) fn has_effective_tag_filter(&self) -> bool {
        !self.effective_tag_filter().is_empty()
    }

    pub(crate) fn tag_filter_title_label(&self) -> String {
        let prefix = if self.tag_filter_mode.is_some() {
            "filter*"
        } else {
            "filter"
        };
        let count = self.active_tag_filter_count();
        if count == 0 {
            format!("{prefix}:off")
        } else {
            format!("{prefix}:{count}")
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
        let Some(mode) = self.tag_filter_mode.as_mut() else {
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

    pub(crate) fn cancel_tag_filter_mode(&mut self) {
        let Some(mode) = self.tag_filter_mode.take() else {
            self.push_debug_log("cancel_tag_filter_mode skipped: mode inactive");
            return;
        };

        self.restore_selection(mode.selected_repo_name.as_deref());
        self.status_message = "tag絞り込みをキャンセル".to_string();
        self.push_debug_log("tag_filter_mode cancelled");
    }

    pub(crate) fn confirm_tag_filter_mode(&mut self) {
        let selected_repo_name = self.selected_repo().map(|repo| repo.name.clone());
        let Some(mode) = self.tag_filter_mode.take() else {
            self.push_debug_log("confirm_tag_filter_mode skipped: mode inactive");
            return;
        };

        let changed = mode.pending_tags != mode.original_tags;
        self.tag_filter = mode.pending_tags;
        self.restore_selection(selected_repo_name.as_deref());
        self.status_message = tag_filter_status_message(changed, &self.tag_filter);
        self.push_debug_log(format!(
            "tag_filter_mode confirmed: active_tags={}",
            self.tag_filter.len()
        ));
    }
}

fn tag_filter_status_message(changed: bool, tag_filter: &BTreeSet<String>) -> String {
    if !changed {
        return "tag絞り込み変更なし".to_string();
    }
    if tag_filter.is_empty() {
        return "tag絞り込み解除".to_string();
    }

    let tags = tag_filter.iter().cloned().collect::<Vec<_>>();
    format!("tag絞り込み更新: {}", tags.join(", "))
}
