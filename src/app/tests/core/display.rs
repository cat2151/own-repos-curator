use super::super::super::{DescDisplayMode, SortMode, TAG_KEYS};
use super::super::common::{app_with_registered_tags, cleanup_app_file, key, repo, shift_key};
use crossterm::event::KeyCode;

#[test]
fn pressing_s_on_main_screen_toggles_sort_mode() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    assert_eq!(app.sort_mode(), SortMode::Created);

    app.handle_key(key(KeyCode::Char('s')));

    assert_eq!(app.sort_mode(), SortMode::Modified);
    assert_eq!(app.status_message, "sort: modify");

    cleanup_app_file(&app);
}

#[test]
fn bottom_hint_on_main_screen_points_to_quit_and_help() {
    let app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    let hint = app.bottom_hint();

    assert_eq!(hint, "q:quit ?:help");

    cleanup_app_file(&app);
}

#[test]
fn shift_d_cycles_desc_display_modes() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    assert_eq!(app.desc_display_mode(), DescDisplayMode::RightPane);

    app.handle_key(shift_key('d'));

    assert_eq!(app.desc_display_mode(), DescDisplayMode::LeftShort);
    assert_eq!(app.status_message, "desc表示: 左paneに1行説明");

    app.handle_key(shift_key('d'));

    assert_eq!(app.desc_display_mode(), DescDisplayMode::LeftShortAndLong);
    assert_eq!(app.status_message, "desc表示: 左paneに1行説明+3行説明");

    app.handle_key(shift_key('d'));

    assert_eq!(app.desc_display_mode(), DescDisplayMode::RightPane);
    assert_eq!(app.status_message, "desc表示: 右下paneに1行/3行説明");

    cleanup_app_file(&app);
}

#[test]
fn shift_l_toggles_debug_log_pane_between_compact_and_expanded() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    assert!(!app.debug_log_expanded());

    app.handle_key(shift_key('l'));

    assert!(app.debug_log_expanded());
    assert_eq!(app.status_message, "debug log: 画面下部50%");

    app.handle_key(shift_key('l'));

    assert!(!app.debug_log_expanded());
    assert_eq!(app.status_message, "debug log: 1行");

    cleanup_app_file(&app);
}

#[test]
fn selected_repo_tag_detail_state_shows_on_off_key_hints() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.tags = vec!["a-page-0".to_string(), "rust".to_string()];
    let mut registered_tags = TAG_KEYS
        .iter()
        .map(|key| format!("{key}-page-0"))
        .collect::<Vec<_>>();
    registered_tags.push("rust".to_string());
    let app = app_with_registered_tags(vec![selected], registered_tags);

    let state = app
        .selected_repo_tag_detail_state()
        .expect("selected repo tag detail state");

    assert_eq!(state.repo_name, "solo");
    assert_eq!(state.tag_count, 2);
    assert_eq!(state.entries[0].key_hint, "a/A (1/2)");
    assert_eq!(state.entries[0].tag, "a-page-0");
    assert_eq!(state.entries[1].key_hint, "r/R (2/2)");
    assert_eq!(state.entries[1].tag, "rust");

    cleanup_app_file(&app);
}

#[test]
fn selected_repo_desc_state_contains_github_short_and_long_descriptions() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.github_desc = "repo from GitHub".to_string();
    selected.desc_short = "short".to_string();
    selected.desc_long = "line 1\nline 2\nline 3".to_string();
    let app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    let state = app
        .selected_repo_desc_state()
        .expect("selected repo desc state");

    assert_eq!(state.repo_name, "solo");
    assert_eq!(state.github_desc, "repo from GitHub");
    assert_eq!(state.desc_short, "short");
    assert_eq!(state.desc_long, "line 1\nline 2\nline 3");
    assert_eq!(state.group, "ungrouped");
    assert_eq!(state.group_key_hint, "u");

    cleanup_app_file(&app);
}
