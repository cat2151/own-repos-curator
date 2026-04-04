use super::super::AppEvent;
use super::common::{app_with_registered_tags, key, repo, shift_key};
use crossterm::event::KeyCode;

#[test]
fn tag_filter_starts_disabled_and_shows_all_repos() {
    let mut rust_repo = repo("rust-repo", "2026-03-02T00:00:00Z", None);
    rust_repo.tags = vec!["rust".to_string()];
    let mut go_repo = repo("go-repo", "2026-03-01T00:00:00Z", None);
    go_repo.tags = vec!["go".to_string()];

    let app = app_with_registered_tags(
        vec![rust_repo, go_repo],
        vec!["rust".to_string(), "go".to_string()],
    );

    assert_eq!(app.visible_repo_indices().len(), 2);
    assert_eq!(app.active_tag_filter_count(), 0);
    assert_eq!(app.tag_filter_title_label(), "filter:off");
}

#[test]
fn pressing_slash_enters_tag_filter_mode() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));

    assert!(app.tag_filter_mode.is_some());
    assert_eq!(
        app.status_message,
        "tag絞り込みモード: a-z ON / A-Z OFF / Enter確定 / Esc取消"
    );
    assert_eq!(
        app.bottom_hint(),
        "a-z:on A-Z:off ←→:page Enter:apply Esc:cancel"
    );
}

#[test]
fn q_does_not_quit_in_filter_mode_and_can_toggle_q_slot() {
    let mut quick_repo = repo("quick-repo", "2026-03-02T00:00:00Z", None);
    quick_repo.tags = vec!["quick".to_string()];
    let mut other_repo = repo("other-repo", "2026-03-01T00:00:00Z", None);
    other_repo.tags = vec!["misc".to_string()];
    let mut app = app_with_registered_tags(
        vec![quick_repo, other_repo],
        vec!["quick".to_string(), "misc".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));
    let event = app.handle_key(key(KeyCode::Char('q')));

    assert_eq!(event, AppEvent::Continue);
    assert!(app.tag_filter_mode.is_some());
    assert_eq!(app.active_tag_filter_tags(), vec!["quick".to_string()]);
    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("quick-repo")
    );

    app.handle_key(shift_key('q'));

    assert_eq!(app.active_tag_filter_count(), 0);
    assert_eq!(app.visible_repo_indices().len(), 2);
}

#[test]
fn enter_confirms_tag_filter_with_all_selected_tags_required() {
    let mut rust_repo = repo("rust", "2026-03-03T00:00:00Z", None);
    rust_repo.tags = vec!["rust".to_string()];
    let mut rust_tui_repo = repo("rust-tui", "2026-03-02T00:00:00Z", None);
    rust_tui_repo.tags = vec!["rust".to_string(), "tui".to_string()];
    let mut tui_repo = repo("tui", "2026-03-01T00:00:00Z", None);
    tui_repo.tags = vec!["tui".to_string()];
    let mut app = app_with_registered_tags(
        vec![rust_repo, rust_tui_repo, tui_repo],
        vec!["rust".to_string(), "tui".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(key(KeyCode::Char('r')));
    app.handle_key(key(KeyCode::Char('t')));

    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("rust-tui")
    );

    app.handle_key(key(KeyCode::Enter));

    assert!(app.tag_filter_mode.is_none());
    assert_eq!(
        app.active_tag_filter_tags(),
        vec!["rust".to_string(), "tui".to_string()]
    );
    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("rust-tui")
    );
    assert_eq!(app.status_message, "tag絞り込み更新: rust, tui");
}

#[test]
fn escape_cancels_pending_filter_and_restores_previous_selection() {
    let mut rust_repo = repo("rust", "2026-03-02T00:00:00Z", None);
    rust_repo.tags = vec!["rust".to_string()];
    let mut go_repo = repo("go", "2026-03-01T00:00:00Z", None);
    go_repo.tags = vec!["go".to_string()];
    let mut app = app_with_registered_tags(
        vec![rust_repo, go_repo],
        vec!["rust".to_string(), "go".to_string()],
    );
    app.handle_key(key(KeyCode::Char('j')));

    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("go")
    );

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(key(KeyCode::Char('r')));

    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("rust")
    );

    app.handle_key(key(KeyCode::Esc));

    assert!(app.tag_filter_mode.is_none());
    assert_eq!(app.active_tag_filter_count(), 0);
    assert_eq!(app.visible_repo_indices().len(), 2);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("go")
    );
    assert_eq!(app.status_message, "tag絞り込みをキャンセル");
}

#[test]
fn tag_catalog_state_marks_active_pending_filters() {
    let mut rust_repo = repo("rust", "2026-03-01T00:00:00Z", None);
    rust_repo.tags = vec!["rust".to_string()];
    let mut app =
        app_with_registered_tags(vec![rust_repo], vec!["rust".to_string(), "go".to_string()]);

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(key(KeyCode::Char('r')));

    let state = app.tag_catalog_state();

    assert!(state.filter_mode_active);
    assert_eq!(state.active_filter_count, 1);
    assert!(state
        .entries
        .iter()
        .any(|entry| entry.tag == "rust" && entry.filter_active));
    assert!(state
        .entries
        .iter()
        .any(|entry| entry.tag == "go" && !entry.filter_active));
}
