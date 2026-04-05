use super::super::{AppEvent, FilterModeFocus};
use super::common::{
    app_with_registered_tags, app_with_registered_tags_and_groups, ctrl_key, key, repo, shift_key,
};
use crossterm::event::KeyCode;

#[test]
fn filter_starts_disabled_and_shows_all_repos() {
    let mut rust_repo = repo("rust-repo", "2026-03-02T00:00:00Z", None);
    rust_repo.tags = vec!["rust".to_string()];
    let mut go_repo = repo("go-repo", "2026-03-01T00:00:00Z", None);
    go_repo.tags = vec!["go".to_string()];

    let app = app_with_registered_tags(
        vec![rust_repo, go_repo],
        vec!["rust".to_string(), "go".to_string()],
    );

    assert_eq!(app.visible_repo_indices().len(), 2);
    assert_eq!(app.active_group_filter(), None);
    assert_eq!(app.active_tag_filter_count(), 0);
    assert_eq!(app.tag_filter_title_label(), "filter:off");
}

#[test]
fn pressing_slash_enters_group_filter_mode() {
    let mut tools_repo = repo("tools-repo", "2026-03-01T00:00:00Z", None);
    tools_repo.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![tools_repo],
        vec!["rust".to_string()],
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));

    assert!(app.filter_mode.is_some());
    assert_eq!(
        app.filter_mode.as_ref().map(|mode| mode.focus),
        Some(FilterModeFocus::Group)
    );
    assert_eq!(
        app.status_message,
        "group絞り込みモード: a-z 選択 / A-Z 解除 / Ctrl+Tでtag / Enter確定 / Esc取消"
    );
    assert_eq!(
        app.bottom_hint(),
        "a-z:group A-Z:clear Ctrl+T:tag ←→:page Enter:apply Esc:cancel"
    );
}

#[test]
fn q_does_not_quit_in_group_filter_mode_and_can_select_q_slot() {
    let mut quick_repo = repo("quick-repo", "2026-03-02T00:00:00Z", None);
    quick_repo.group = "quick".to_string();
    let mut other_repo = repo("other-repo", "2026-03-01T00:00:00Z", None);
    other_repo.group = "misc".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![quick_repo, other_repo],
        Vec::new(),
        vec!["quick".to_string(), "misc".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));
    let event = app.handle_key(key(KeyCode::Char('q')));

    assert_eq!(event, AppEvent::Continue);
    assert!(app.filter_mode.is_some());
    assert_eq!(app.active_group_filter(), Some("quick".to_string()));
    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("quick-repo")
    );

    app.handle_key(shift_key('q'));

    assert_eq!(app.active_group_filter(), None);
    assert_eq!(app.visible_repo_indices().len(), 2);
}

#[test]
fn ctrl_t_switches_to_tag_filter_mode_and_ctrl_g_returns_to_group_mode() {
    let mut selected = repo("selected", "2026-03-03T00:00:00Z", None);
    selected.group = "tools".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![selected],
        vec!["rust".to_string()],
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(ctrl_key('t'));

    assert_eq!(
        app.filter_mode.as_ref().map(|mode| mode.focus),
        Some(FilterModeFocus::Tag)
    );
    assert_eq!(
        app.status_message,
        "tag絞り込みモード: a-z ON / A-Z OFF / Ctrl+Gでgroup / Enter確定 / Esc取消"
    );
    assert_eq!(
        app.bottom_hint(),
        "a-z:on A-Z:off Ctrl+G:group ←→:page Enter:apply Esc:cancel"
    );

    app.handle_key(ctrl_key('g'));

    assert_eq!(
        app.filter_mode.as_ref().map(|mode| mode.focus),
        Some(FilterModeFocus::Group)
    );
}

#[test]
fn enter_confirms_group_and_tag_filters_together() {
    let mut rust_web = repo("rust-web", "2026-03-03T00:00:00Z", None);
    rust_web.group = "web".to_string();
    rust_web.tags = vec!["rust".to_string()];
    let mut rust_tools = repo("rust-tools", "2026-03-02T00:00:00Z", None);
    rust_tools.group = "tools".to_string();
    rust_tools.tags = vec!["rust".to_string()];
    let mut go_web = repo("go-web", "2026-03-01T00:00:00Z", None);
    go_web.group = "web".to_string();
    go_web.tags = vec!["go".to_string()];
    let mut app = app_with_registered_tags_and_groups(
        vec![rust_web, rust_tools, go_web],
        vec!["rust".to_string(), "go".to_string()],
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(key(KeyCode::Char('w')));
    app.handle_key(ctrl_key('t'));
    app.handle_key(key(KeyCode::Char('r')));

    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("rust-web")
    );

    app.handle_key(key(KeyCode::Enter));

    assert!(app.filter_mode.is_none());
    assert_eq!(app.active_group_filter(), Some("web".to_string()));
    assert_eq!(app.active_tag_filter_tags(), vec!["rust".to_string()]);
    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(app.status_message, "絞り込み更新: group=web / tags=rust");
}

#[test]
fn uppercase_group_shortcut_clears_pending_group_filter_regardless_of_key() {
    let mut tools_repo = repo("tools", "2026-03-02T00:00:00Z", None);
    tools_repo.group = "tools".to_string();
    let mut web_repo = repo("web", "2026-03-01T00:00:00Z", None);
    web_repo.group = "web".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![tools_repo, web_repo],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(key(KeyCode::Char('w')));
    assert_eq!(app.active_group_filter(), Some("web".to_string()));

    app.handle_key(key(KeyCode::Char('X')));

    assert_eq!(app.active_group_filter(), None);
    assert_eq!(app.visible_repo_indices().len(), 2);
    assert_eq!(app.status_message, "group絞り込み候補 解除");
}

#[test]
fn escape_cancels_pending_filter_and_restores_previous_selection() {
    let mut tools_repo = repo("tools", "2026-03-02T00:00:00Z", None);
    tools_repo.group = "tools".to_string();
    let mut web_repo = repo("web", "2026-03-01T00:00:00Z", None);
    web_repo.group = "web".to_string();
    let mut app = app_with_registered_tags_and_groups(
        vec![tools_repo, web_repo],
        Vec::new(),
        vec!["tools".to_string(), "web".to_string()],
    );
    app.handle_key(key(KeyCode::Char('j')));

    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("web")
    );

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(key(KeyCode::Char('t')));

    assert_eq!(app.visible_repo_indices().len(), 1);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("tools")
    );

    app.handle_key(key(KeyCode::Esc));

    assert!(app.filter_mode.is_none());
    assert_eq!(app.active_group_filter(), None);
    assert_eq!(app.visible_repo_indices().len(), 2);
    assert_eq!(
        app.selected_repo().map(|repo| repo.name.as_str()),
        Some("web")
    );
    assert_eq!(app.status_message, "絞り込みをキャンセル");
}

#[test]
fn tag_catalog_state_marks_active_pending_filters_after_switching_to_tag_mode() {
    let mut rust_repo = repo("rust", "2026-03-01T00:00:00Z", None);
    rust_repo.tags = vec!["rust".to_string()];
    let mut app =
        app_with_registered_tags(vec![rust_repo], vec!["rust".to_string(), "go".to_string()]);

    app.handle_key(key(KeyCode::Char('/')));
    app.handle_key(ctrl_key('t'));
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
