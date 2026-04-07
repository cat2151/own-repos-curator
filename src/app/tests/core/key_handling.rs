use super::*;

#[test]
fn tag_manager_state_is_none_until_opened() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);

    assert!(app.tag_manager_state().is_none());

    app.open_tag_manager();

    assert!(app.tag_manager_state().is_some());

    cleanup_app_file(&app);
}

#[test]
fn pressing_n_on_main_screen_opens_tag_input_when_no_registered_tags_exist() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);

    app.handle_key(key(KeyCode::Char('n')));

    assert!(app.tag_manager.is_none());
    assert!(matches!(
        app.tag_input.as_ref().map(|input| &input.mode),
        Some(TagInputMode::CreateAndAssignToSelectedRepo)
    ));
    assert_eq!(app.status_message, "新規tag: Enterで保存 / Escでキャンセル");

    cleanup_app_file(&app);
}

#[test]
fn pressing_n_on_main_screen_opens_tag_input_even_when_registered_tags_exist() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(key(KeyCode::Char('n')));

    assert!(matches!(
        app.tag_input.as_ref().map(|input| &input.mode),
        Some(TagInputMode::CreateAndAssignToSelectedRepo)
    ));
    assert_eq!(app.status_message, "新規tag: Enterで保存 / Escでキャンセル");

    cleanup_app_file(&app);
}

#[test]
fn pressing_q_on_main_screen_quits() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    let event = app.handle_key(key(KeyCode::Char('q')));

    assert!(matches!(event, AppEvent::Quit));

    cleanup_app_file(&app);
}

#[test]
fn pressing_q_in_tag_manager_quits() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );
    app.open_tag_manager();

    let event = app.handle_key(key(KeyCode::Char('q')));

    assert!(matches!(event, AppEvent::Quit));

    cleanup_app_file(&app);
}

#[test]
fn pressing_q_while_typing_tag_input_keeps_input_open() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);
    app.handle_key(key(KeyCode::Char('n')));

    let event = app.handle_key(key(KeyCode::Char('q')));

    assert!(matches!(event, AppEvent::Continue));
    assert_eq!(
        app.tag_input.as_ref().map(|input| input.value()),
        Some("q".to_string())
    );

    cleanup_app_file(&app);
}

#[test]
fn tag_binding_keys_include_q() {
    assert!(TAG_KEYS.contains(&'q'));
}

#[test]
fn pressing_ctrl_n_on_main_screen_opens_tag_input() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);

    app.handle_key(ctrl_key('n'));

    assert!(matches!(
        app.tag_input.as_ref().map(|input| &input.mode),
        Some(TagInputMode::CreateAndAssignToSelectedRepo)
    ));
    assert_eq!(app.status_message, "新規tag: Enterで保存 / Escでキャンセル");

    cleanup_app_file(&app);
}

#[test]
fn pressing_ctrl_g_on_main_screen_opens_group_input() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);

    app.handle_key(ctrl_key('g'));

    assert!(matches!(
        app.group_input.as_ref().map(|input| &input.mode),
        Some(GroupInputMode::CreateAndAssignToSelectedRepo)
    ));
    assert_eq!(
        app.status_message,
        "新規group: Enterで保存 / Escでキャンセル"
    );

    cleanup_app_file(&app);
}

#[test]
fn pressing_shift_t_on_main_screen_opens_tag_manager() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(shift_key('t'));

    assert!(app.tag_manager.is_some());
    assert_eq!(
        app.status_message,
        "tag manager: j/k移動 n新規 r改名 Escで閉じる"
    );

    cleanup_app_file(&app);
}

#[test]
fn pressing_shift_g_on_main_screen_opens_group_manager() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(shift_key('g'));

    assert!(app.group_manager.is_some());
    assert_eq!(
        app.status_message,
        "group manager: j/k移動 n新規 r改名 Escで閉じる"
    );

    cleanup_app_file(&app);
}

#[test]
fn pressing_tab_on_main_screen_no_longer_opens_tag_manager() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(key(KeyCode::Tab));

    assert!(app.tag_manager.is_none());

    cleanup_app_file(&app);
}
