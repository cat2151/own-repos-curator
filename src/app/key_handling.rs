use super::{
    helpers::{describe_key_code, is_ctrl_char, is_plain_or_ctrl_char, is_quit_key, is_shift_char},
    App, AppEvent, DescDisplayMode, HelpScreen, SortMode,
};
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub(crate) fn tick(&mut self) {
        self.tick_background_jobs();
    }

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

    fn toggle_debug_log_pane(&mut self) {
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

    fn cycle_desc_display_mode(&mut self) {
        self.desc_display_mode = self.desc_display_mode.cycle();
        self.status_message = self.desc_display_mode.status_message().to_string();
        self.push_debug_log(self.desc_display_mode.debug_log_message());
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> AppEvent {
        self.push_debug_log(format!(
            "handle_key: code={} tag_input={} group_input={} editor={} tag_binding={} group_binding={} filter={} tag_manager={} group_manager={} help={}",
            describe_key_code(&key.code),
            self.tag_input.is_some(),
            self.group_input.is_some(),
            self.editor.is_some(),
            self.tag_binding_mode.is_some(),
            self.group_binding_mode.is_some(),
            self.filter_mode.is_some(),
            self.tag_manager.is_some(),
            self.group_manager.is_some(),
            self.help_screen.is_some()
        ));

        if self.tag_input.is_none()
            && self.group_input.is_none()
            && self.editor.is_none()
            && self.tag_binding_mode.is_none()
            && self.group_binding_mode.is_none()
            && self.filter_mode.is_none()
            && is_quit_key(&key)
        {
            return AppEvent::Quit;
        }

        if self.tag_input.is_none()
            && self.group_input.is_none()
            && self.editor.is_none()
            && self.tag_binding_mode.is_none()
            && self.group_binding_mode.is_none()
            && self.filter_mode.is_none()
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
            && self.filter_mode.is_none()
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

        if self.filter_mode.is_some() {
            self.push_debug_log("route -> handle_filter_mode_key");
            self.handle_filter_mode_key(key);
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
                self.begin_filter_mode();
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
