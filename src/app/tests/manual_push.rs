use super::super::background::StartupJobs;
use super::common::{app_with_repos, cleanup_app_file, repo, shift_key};
use crate::json_auto_push::AutoPushOutcome;

#[test]
fn pressing_shift_p_on_main_screen_starts_manual_json_push_in_background() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);
    app.startup_jobs = StartupJobs::idle_without_json_push_spawn();

    app.handle_key(shift_key('p'));

    assert_eq!(app.status_message, "JSON手動pushを開始");
    assert_eq!(
        app.background_status_message(),
        Some("[-] JSON手動pushをバックグラウンド実行中".to_string())
    );

    cleanup_app_file(&app);
}

#[test]
fn pressing_shift_p_while_startup_jobs_are_running_shows_busy_message() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);
    app.startup_jobs = StartupJobs::with_test_github_sync_result(Ok(Vec::new()));

    app.handle_key(shift_key('p'));

    assert_eq!(
        app.status_message,
        "起動時バックグラウンド処理の完了後に再実行してください"
    );

    cleanup_app_file(&app);
}

#[test]
fn tick_applies_background_manual_push_result_to_meta() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);
    app.startup_jobs = StartupJobs::with_test_manual_push_result(Ok((
        AutoPushOutcome::UpToDate {
            repo: "cat2151/backups".to_string(),
            date: "2026-04-05".to_string(),
        },
        Some("2026-04-05".to_string()),
    )));

    app.tick();

    assert_eq!(app.data.meta.last_json_commit_push_date, "2026-04-05");
    assert_eq!(app.status_message, "JSON手動push: 変更なし");

    cleanup_app_file(&app);
}
