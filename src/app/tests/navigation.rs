use super::super::{helpers::sort_repo_indices, SortMode};
use super::common::{app_with_registered_tags, app_with_repos, cleanup_app_file, key, repo};
use crossterm::event::KeyCode;

#[test]
fn move_down_stops_at_last_repo() {
    let mut app = app_with_repos(vec![
        repo("first", "2026-03-01T00:00:00Z", None),
        repo("second", "2026-03-02T00:00:00Z", None),
    ]);

    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.selected_index(), Some(1));

    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.selected_index(), Some(1));

    cleanup_app_file(&app);
}

#[test]
fn move_up_stops_at_first_repo() {
    let mut app = app_with_repos(vec![
        repo("first", "2026-03-01T00:00:00Z", None),
        repo("second", "2026-03-02T00:00:00Z", None),
    ]);

    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.selected_index(), Some(1));

    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.selected_index(), Some(0));

    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.selected_index(), Some(0));

    cleanup_app_file(&app);
}

#[test]
fn page_down_moves_selection_by_page_step_and_stops_at_last_repo() {
    let repos = (0..15)
        .map(|index| repo(&format!("repo-{index}"), "2026-03-01T00:00:00Z", None))
        .collect();
    let mut app = app_with_repos(repos);

    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(app.selected_index(), Some(10));

    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(app.selected_index(), Some(14));

    cleanup_app_file(&app);
}

#[test]
fn page_up_moves_selection_by_page_step_and_stops_at_first_repo() {
    let repos = (0..15)
        .map(|index| repo(&format!("repo-{index}"), "2026-03-01T00:00:00Z", None))
        .collect();
    let mut app = app_with_repos(repos);

    app.handle_key(key(KeyCode::PageDown));
    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(app.selected_index(), Some(14));

    app.handle_key(key(KeyCode::PageUp));
    assert_eq!(app.selected_index(), Some(4));

    app.handle_key(key(KeyCode::PageUp));
    assert_eq!(app.selected_index(), Some(0));

    cleanup_app_file(&app);
}

#[test]
fn tag_manager_move_down_stops_at_last_tag() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string(), "tui".to_string()],
    );
    app.open_tag_manager();

    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(
        app.tag_manager.as_ref().map(|manager| manager.selected),
        Some(1)
    );

    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(
        app.tag_manager.as_ref().map(|manager| manager.selected),
        Some(1)
    );

    cleanup_app_file(&app);
}

#[test]
fn tag_manager_move_up_stops_at_first_tag() {
    let mut app = app_with_registered_tags(
        vec![repo("solo", "2026-03-01T00:00:00Z", None)],
        vec!["rust".to_string(), "tui".to_string()],
    );
    app.open_tag_manager();

    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(
        app.tag_manager.as_ref().map(|manager| manager.selected),
        Some(1)
    );

    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(
        app.tag_manager.as_ref().map(|manager| manager.selected),
        Some(0)
    );

    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(
        app.tag_manager.as_ref().map(|manager| manager.selected),
        Some(0)
    );

    cleanup_app_file(&app);
}

#[test]
fn empty_tag_input_stays_open() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);
    app.open_tag_manager();

    app.handle_key(key(KeyCode::Char('n')));
    app.handle_key(key(KeyCode::Enter));

    assert!(app.tag_input.is_some());
    assert_eq!(app.status_message, "tagは空にできません");

    cleanup_app_file(&app);
}

#[test]
fn sort_repo_indices_by_created_desc() {
    let repos = vec![
        repo(
            "older",
            "2026-01-01T00:00:00Z",
            Some("2026-04-01T00:00:00Z"),
        ),
        repo(
            "newer",
            "2026-03-01T00:00:00Z",
            Some("2026-03-05T00:00:00Z"),
        ),
        repo("middle", "2026-02-01T00:00:00Z", None),
    ];
    let mut indices = vec![0, 1, 2];

    sort_repo_indices(&mut indices, &repos, SortMode::Created);

    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn sort_repo_indices_by_updated_desc_with_created_fallback() {
    let repos = vec![
        repo("fallback", "2026-03-10T00:00:00Z", None),
        repo(
            "stale",
            "2026-01-01T00:00:00Z",
            Some("2026-01-02T00:00:00Z"),
        ),
        repo(
            "fresh",
            "2025-12-01T00:00:00Z",
            Some("2026-04-01T00:00:00Z"),
        ),
    ];
    let mut indices = vec![0, 1, 2];

    sort_repo_indices(&mut indices, &repos, SortMode::Modified);

    assert_eq!(indices, vec![2, 0, 1]);
}
