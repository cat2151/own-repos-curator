use super::super::super::EditorField;
use super::super::common::{
    app_with_registered_tags, cleanup_app_file, ctrl_key, key, repo, shift_key,
};
use crossterm::event::KeyCode;

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
