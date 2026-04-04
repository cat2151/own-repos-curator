use super::super::{
    helpers::{summarize_tag_counts, tag_bindings_for_page, tag_page_count, tag_shortcut_for_tag},
    AppEvent, DescDisplayMode, EditorField, GroupInputMode, SortMode, TagInputMode,
    TagSummaryEntry, TAG_KEYS,
};
use super::common::{
    app_with_registered_tags, app_with_repos, cleanup_app_file, ctrl_key, key, repo, shift_key,
};
use crossterm::event::KeyCode;

#[test]
fn tag_page_count_rounds_up() {
    assert_eq!(tag_page_count(0), 0);
    assert_eq!(tag_page_count(1), 1);
    assert_eq!(tag_page_count(TAG_KEYS.len()), 1);
    assert_eq!(tag_page_count(TAG_KEYS.len() + 1), 2);
    assert_eq!(tag_page_count(TAG_KEYS.len() * 2 + 3), 3);
}

#[test]
fn tag_bindings_prefer_tag_initials_on_each_page() {
    let mut tags = TAG_KEYS
        .iter()
        .map(|key| format!("{key}-page-0"))
        .collect::<Vec<_>>();
    tags.extend(["rust", "vibes", "zig"].map(String::from));

    let bindings = tag_bindings_for_page(&tags, 1);

    assert_eq!(bindings.len(), 3);
    assert_eq!(bindings[0].key, 'r');
    assert_eq!(bindings[0].tag, "rust");
    assert_eq!(bindings[1].key, 'v');
    assert_eq!(bindings[2].key, 'z');
}

#[test]
fn tag_bindings_clamp_to_last_page() {
    let mut tags = TAG_KEYS
        .iter()
        .map(|key| format!("{key}-page-0"))
        .collect::<Vec<_>>();
    tags.extend(["rust", "vibes"].map(String::from));

    let bindings = tag_bindings_for_page(&tags, 99);

    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].key, 'r');
    assert_eq!(bindings[0].tag, "rust");
    assert_eq!(bindings[1].key, 'v');
    assert_eq!(bindings[1].tag, "vibes");
}

#[test]
fn tag_shortcut_for_tag_returns_page_and_key() {
    let mut tags = TAG_KEYS
        .iter()
        .map(|key| format!("{key}-page-0"))
        .collect::<Vec<_>>();
    tags.extend(["rust", "vibes"].map(String::from));

    assert_eq!(tag_shortcut_for_tag(&tags, "a-page-0"), Some((0, 'a')));
    assert_eq!(tag_shortcut_for_tag(&tags, "rust"), Some((1, 'r')));
    assert_eq!(tag_shortcut_for_tag(&tags, "missing"), None);
}

#[test]
fn tag_bindings_fall_back_to_next_available_letter_when_initials_collide() {
    let tags = vec![
        "rust".to_string(),
        "ruby".to_string(),
        "rock".to_string(),
        "vibes".to_string(),
    ];

    let bindings = tag_bindings_for_page(&tags, 0);

    assert_eq!(bindings[0].key, 'r');
    assert_eq!(bindings[0].tag, "rust");
    assert_eq!(bindings[1].key, 'u');
    assert_eq!(bindings[1].tag, "ruby");
    assert_eq!(bindings[2].key, 'o');
    assert_eq!(bindings[2].tag, "rock");
    assert_eq!(bindings[3].key, 'v');
    assert_eq!(bindings[3].tag, "vibes");
}

#[test]
fn summarize_tag_counts_aggregates_and_sorts_by_count_then_name() {
    let repo_a = vec!["rust".to_string(), "tui".to_string()];
    let repo_b = vec!["rust".to_string(), "obsidian plugin".to_string()];
    let repo_c = vec!["obsidian plugin".to_string()];

    let summary = summarize_tag_counts([repo_a.as_slice(), repo_b.as_slice(), repo_c.as_slice()]);

    assert_eq!(
        summary,
        vec![
            TagSummaryEntry {
                tag: "obsidian plugin".to_string(),
                count: 2,
            },
            TagSummaryEntry {
                tag: "rust".to_string(),
                count: 2,
            },
            TagSummaryEntry {
                tag: "tui".to_string(),
                count: 1,
            },
        ]
    );
}

#[test]
fn summarize_tag_counts_returns_empty_for_empty_input() {
    let summary = summarize_tag_counts(std::iter::empty::<&[String]>());

    assert!(summary.is_empty());
}

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

#[test]
fn pressing_e_on_main_screen_starts_short_desc_editor() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(key(KeyCode::Char('e')));

    assert!(matches!(
        app.editor.as_ref().map(|editor| editor.field),
        Some(EditorField::ShortDesc)
    ));
    assert_eq!(
        app.status_message,
        "1行説明を編集中: Enterで保存 / Escでキャンセル"
    );

    cleanup_app_file(&app);
}

#[test]
fn pressing_shift_e_on_main_screen_starts_short_desc_editor() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(shift_key('e'));

    assert!(matches!(
        app.editor.as_ref().map(|editor| editor.field),
        Some(EditorField::ShortDesc)
    ));

    cleanup_app_file(&app);
}

#[test]
fn pressing_l_on_main_screen_starts_long_desc_editor() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string()],
    );

    app.handle_key(key(KeyCode::Char('l')));

    assert!(matches!(
        app.editor.as_ref().map(|editor| editor.field),
        Some(EditorField::LongDesc)
    ));
    assert_eq!(
        app.status_message,
        "3行説明を編集中: Ctrl+Sで保存 / Enterで改行 / Escでキャンセル"
    );

    cleanup_app_file(&app);
}

#[test]
fn short_desc_editor_backspace_respects_cursor_position() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.desc_short = "abcd".to_string();
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('e')));
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Backspace));
    app.handle_key(key(KeyCode::Enter));

    assert_eq!(app.data.repos[0].desc_short, "acd");

    cleanup_app_file(&app);
}

#[test]
fn long_desc_editor_inserts_text_at_cursor_position() {
    let mut selected = repo("solo", "2026-03-01T00:00:00Z", None);
    selected.desc_long = "ab".to_string();
    let mut app = app_with_registered_tags(vec![selected], vec!["rust".to_string()]);

    app.handle_key(key(KeyCode::Char('l')));
    app.handle_key(key(KeyCode::Left));
    app.handle_key(key(KeyCode::Char('X')));
    app.handle_key(ctrl_key('s'));

    assert_eq!(app.data.repos[0].desc_long, "aXb");

    cleanup_app_file(&app);
}

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
