use super::super::{AppEvent, GroupInputMode, HelpScreen};
use super::common::{
    app_with_registered_tags_and_groups, app_with_repos, cleanup_app_file, ctrl_key, key, repo,
};
use crossterm::event::KeyCode;

#[test]
fn pressing_n_in_group_manager_opens_group_input_overlay() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app =
        app_with_registered_tags_and_groups(vec![selected], Vec::new(), vec!["tools".to_string()]);
    app.open_group_manager();

    app.handle_key(key(KeyCode::Char('n')));

    assert!(matches!(
        app.group_input.as_ref().map(|input| &input.mode),
        Some(GroupInputMode::CreateRegisteredOnly)
    ));
    assert_eq!(
        app.status_message,
        "新規group: Enterで保存 / Escでキャンセル"
    );

    cleanup_app_file(&app);
}

#[test]
fn entering_new_group_adds_it_to_group_manager_list() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app =
        app_with_registered_tags_and_groups(vec![selected], Vec::new(), vec!["tools".to_string()]);
    app.open_group_manager();

    app.handle_key(key(KeyCode::Char('n')));
    for ch in "web".chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));

    assert!(app.group_input.is_none());
    assert_eq!(app.data.repos[0].group, "tools");
    assert_eq!(
        app.data.registered_groups,
        vec!["tools".to_string(), "web".to_string()]
    );
    let state = app.group_manager_state().expect("group manager state");
    assert_eq!(state.entries[1].group, "web");
    assert_eq!(state.selected, 1);
    assert_eq!(app.status_message, "group追加(global): web");

    cleanup_app_file(&app);
}

#[test]
fn group_input_inserts_text_at_cursor_position() {
    let mut app = app_with_repos(vec![repo("selected", "2026-03-01T00:00:00Z", None)]);

    app.handle_key(ctrl_key('g'));
    for ch in "abcd".chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Char('X')));
    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.data.repos[0].group, "abXcd");
    assert!(app
        .data
        .registered_groups
        .iter()
        .any(|group| group == "abXcd"));

    cleanup_app_file(&app);
}

#[test]
fn pressing_g_enters_group_binding_mode() {
    let mut selected = repo("selected", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('g')));

    assert!(app.group_binding_mode.is_some());
    assert_eq!(app.status_message, "group割り当てモード開始: ? で専用help");

    cleanup_app_file(&app);
}

#[test]
fn pressing_question_mark_in_group_binding_mode_opens_group_binding_help() {
    let mut selected = repo("selected", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('g')));
    app.handle_key(key(KeyCode::Char('?')));

    assert!(app.group_binding_mode.is_some());
    assert_eq!(app.help_screen, Some(HelpScreen::GroupBinding));

    cleanup_app_file(&app);
}

#[test]
fn group_binding_mode_bottom_hint_points_to_assign_cancel_and_help() {
    let mut selected = repo("selected", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('g')));

    assert_eq!(app.bottom_hint(), "a-z:assign Esc:cancel ?:help");

    cleanup_app_file(&app);
}

#[test]
fn enter_in_group_binding_mode_no_longer_confirms_anything() {
    let mut selected = repo("selected", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('g')));
    app.handle_key(key(KeyCode::Enter));

    assert!(app.group_binding_mode.is_some());
    assert_eq!(app.data.repos[0].group, "tools");
    assert_eq!(app.status_message, "group割り当てモード開始: ? で専用help");

    cleanup_app_file(&app);
}

#[test]
fn lowercase_group_shortcut_assigns_registered_group_immediately() {
    let mut selected = repo("selected", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('g')));
    app.handle_key(key(KeyCode::Char('w')));

    assert_eq!(app.data.repos[0].group, "web");
    assert!(app.group_binding_mode.is_none());
    assert_eq!(app.status_message, "group更新: selected");

    cleanup_app_file(&app);
}

#[test]
fn escape_in_group_binding_mode_cancels_without_changing_group() {
    let mut selected = repo("selected", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('g')));
    app.handle_key(key(KeyCode::Esc));

    assert!(app.group_binding_mode.is_none());
    assert_eq!(app.data.repos[0].group, "tools");
    assert_eq!(app.status_message, "group割り当てをキャンセル");

    cleanup_app_file(&app);
}

#[test]
fn ctrl_g_can_create_and_assign_new_group_to_selected_repo() {
    let mut app = app_with_repos(vec![repo("selected", "2026-03-01T00:00:00Z", None)]);

    app.handle_key(ctrl_key('g'));
    for ch in "web".chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.data.repos[0].group, "web");
    assert!(app
        .data
        .registered_groups
        .iter()
        .any(|group| group == "web"));
    assert_eq!(app.status_message, "group更新: selected");

    cleanup_app_file(&app);
}

#[test]
fn q_does_not_quit_inside_group_binding_mode_and_can_bind_q_slot() {
    let mut selected = repo("selected", "2026-03-01T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        Vec::new(),
        vec!["quick".to_string(), "tools".to_string()],
    );

    app.handle_key(key(KeyCode::Char('g')));
    let event = app.handle_key(key(KeyCode::Char('q')));

    assert!(matches!(event, AppEvent::Continue));
    assert_eq!(app.data.repos[0].group, "quick");
    assert!(app.group_binding_mode.is_none());
    assert_eq!(app.status_message, "group更新: selected");

    cleanup_app_file(&app);
}
