use super::super::{AppEvent, HelpScreen};
use super::common::{app_with_registered_tags, cleanup_app_file, key, repo};
use crossterm::event::KeyCode;

#[test]
fn pressing_n_in_empty_tag_manager_opens_tag_input_overlay() {
    let mut app =
        app_with_registered_tags(vec![repo("solo", "2026-03-01T00:00:00Z", None)], Vec::new());
    app.open_tag_manager();

    app.handle_key(key(KeyCode::Char('n')));

    assert!(matches!(
        app.tag_input.as_ref().map(|input| &input.mode),
        Some(super::super::TagInputMode::CreateRegisteredOnly)
    ));
    assert_eq!(app.status_message, "新規tag: Enterで保存 / Escでキャンセル");

    cleanup_app_file(&app);
}

#[test]
fn entering_new_tag_adds_it_to_tag_manager_list() {
    let mut app =
        app_with_registered_tags(vec![repo("solo", "2026-03-01T00:00:00Z", None)], Vec::new());
    app.open_tag_manager();

    app.handle_key(key(KeyCode::Char('n')));
    for ch in ['r', 'u', 's', 't'] {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));

    assert!(app.tag_input.is_none());
    assert!(app.data.repos[0].tags.is_empty());
    assert_eq!(app.data.registered_tags, vec!["rust".to_string()]);

    let state = app.tag_manager_state().expect("tag manager state");
    assert_eq!(state.entries.len(), 1);
    assert_eq!(state.entries[0].tag, "rust");
    assert_eq!(app.status_message, "tag追加(global): rust");

    cleanup_app_file(&app);
}

#[test]
fn tag_input_delete_removes_character_at_cursor_position() {
    let mut app =
        app_with_registered_tags(vec![repo("solo", "2026-03-01T00:00:00Z", None)], Vec::new());

    app.handle_key(key(KeyCode::Char('n')));
    for ch in "abcd".chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Delete));
    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.data.repos[0].tags, vec!["abd".to_string()]);
    assert_eq!(app.data.registered_tags, vec!["abd".to_string()]);

    cleanup_app_file(&app);
}

#[test]
fn tag_manager_state_lists_registered_tags() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.open_tag_manager();

    let state = app.tag_manager_state().expect("tag manager state");
    assert_eq!(state.entries.len(), 1);
    assert_eq!(state.entries[0].tag, "rust");

    cleanup_app_file(&app);
}

#[test]
fn space_in_tag_manager_no_longer_toggles_selected_repo_tag() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.tags = vec!["rust".to_string()];
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);
    app.open_tag_manager();

    app.handle_key(key(KeyCode::Char(' ')));

    assert_eq!(app.data.repos[0].tags, vec!["rust".to_string()]);
    assert_eq!(
        app.status_message,
        "tag manager: j/k移動 n新規 r改名 Escで閉じる"
    );

    cleanup_app_file(&app);
}

#[test]
fn pressing_d_in_tag_manager_no_longer_deletes_registered_tags() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.tags = vec!["rust".to_string()];
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);
    app.open_tag_manager();

    app.handle_key(key(KeyCode::Char('d')));

    assert_eq!(app.data.registered_tags, vec!["rust".to_string()]);
    assert_eq!(app.data.repos[0].tags, vec!["rust".to_string()]);
    assert_eq!(
        app.status_message,
        "tag manager: j/k移動 n新規 r改名 Escで閉じる"
    );

    cleanup_app_file(&app);
}

#[test]
fn pressing_t_enters_tag_binding_mode() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));

    assert!(app.tag_binding_mode.is_some());
    assert_eq!(app.status_message, "tag紐付けモード開始: ? で専用help");

    cleanup_app_file(&app);
}

#[test]
fn pressing_question_mark_on_main_screen_opens_main_help() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(key(KeyCode::Char('?')));

    assert_eq!(app.help_screen, Some(HelpScreen::Main));
    assert!(app.tag_binding_mode.is_none());

    cleanup_app_file(&app);
}

#[test]
fn pressing_question_mark_in_tag_binding_mode_opens_tag_binding_help() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));
    app.handle_key(key(KeyCode::Char('?')));

    assert_eq!(app.help_screen, Some(HelpScreen::TagBinding));
    assert!(app.tag_binding_mode.is_some());

    cleanup_app_file(&app);
}

#[test]
fn escape_closes_tag_binding_help_before_leaving_mode() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));
    app.handle_key(key(KeyCode::Char('?')));
    app.handle_key(key(KeyCode::Esc));

    assert_eq!(app.help_screen, None);
    assert!(app.tag_binding_mode.is_some());

    cleanup_app_file(&app);
}

#[test]
fn tag_binding_mode_bottom_hint_points_to_save_cancel_and_help() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));

    let hint = app.bottom_hint();

    assert_eq!(hint, "Enter:save Esc:cancel ?:help");

    cleanup_app_file(&app);
}

#[test]
fn lowercase_tag_shortcut_requires_tag_binding_mode() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('a')));

    assert!(app.data.repos[0].tags.is_empty());
    assert!(!app.status_message.contains("tag追加"));

    cleanup_app_file(&app);
}

#[test]
fn lowercase_tag_shortcut_assigns_registered_tag_after_confirm() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));
    app.handle_key(key(KeyCode::Char('r')));

    assert!(app.data.repos[0].tags.is_empty());

    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.data.repos[0].tags, vec!["rust".to_string()]);
    assert_eq!(app.status_message, "tag更新: selected");

    cleanup_app_file(&app);
}

#[test]
fn uppercase_tag_shortcut_removes_registered_tag_from_selected_repo_only_after_confirm() {
    let mut selected = repo("selected", "2026-03-02T00:00:00Z", None);
    selected.tags = vec!["rust".to_string()];
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));
    app.handle_key(key(KeyCode::Char('R')));
    assert_eq!(app.data.repos[0].tags, vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Enter));

    assert!(app.data.repos[0].tags.is_empty());
    assert_eq!(app.data.registered_tags, vec!["rust".to_string()]);
    assert_eq!(app.status_message, "tag更新: selected");

    cleanup_app_file(&app);
}

#[test]
fn escape_in_tag_binding_mode_discards_pending_changes() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));
    app.handle_key(key(KeyCode::Char('r')));
    app.handle_key(key(KeyCode::Esc));

    assert!(app.tag_binding_mode.is_none());
    assert!(app.data.repos[0].tags.is_empty());
    assert_eq!(app.status_message, "tag紐付けをキャンセル");

    cleanup_app_file(&app);
}

#[test]
fn q_does_not_quit_inside_tag_binding_mode_and_can_bind_q_slot() {
    let selected = repo("selected", "2026-03-02T00:00:00Z", None);
    let mut app = app_with_registered_tags(vec![selected], vec!["quick".to_string()]);

    app.handle_key(key(KeyCode::Char('t')));

    let event = app.handle_key(key(KeyCode::Char('q')));

    assert!(matches!(event, AppEvent::Continue));
    assert!(app.tag_binding_mode.is_some());

    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.data.repos[0].tags, vec!["quick".to_string()]);
    assert_eq!(app.status_message, "tag更新: selected");

    cleanup_app_file(&app);
}
