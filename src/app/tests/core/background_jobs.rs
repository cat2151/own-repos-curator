use super::*;

#[test]
fn tick_applies_background_startup_sync_without_overwriting_local_metadata() {
    let mut selected = repo(
        "selected",
        "2026-03-01T00:00:00Z",
        Some("2026-03-02T00:00:00Z"),
    );
    selected.desc_short = "keep local desc".to_string();
    selected.group = "tools".to_string();
    selected.tags = vec!["rust".to_string()];
    let mut app = app_with_repos(vec![selected]);
    app.startup_jobs = StartupJobs::with_test_github_sync_result(Ok(vec![
        FetchedRepo {
            name: "selected".to_string(),
            created_at: parse_datetime("2026-03-01T00:00:00Z"),
            updated_at: parse_datetime("2026-04-01T00:00:00Z"),
            github_desc: "synced from github".to_string(),
        },
        FetchedRepo {
            name: "new-repo".to_string(),
            created_at: parse_datetime("2026-04-02T00:00:00Z"),
            updated_at: parse_datetime("2026-04-03T00:00:00Z"),
            github_desc: "brand new".to_string(),
        },
    ]));

    app.tick();

    assert_eq!(app.data.repos.len(), 2);
    assert_eq!(app.data.repos[1].name, "selected");
    assert_eq!(app.data.repos[1].desc_short, "keep local desc");
    assert_eq!(app.data.repos[1].group, "tools");
    assert_eq!(app.data.repos[1].tags, vec!["rust".to_string()]);
    assert_eq!(app.data.repos[1].github_desc, "synced from github");
    assert_eq!(app.data.repos[0].name, "new-repo");
    assert_eq!(app.data.repos[0].group, DEFAULT_GROUP_NAME);
    assert_eq!(
        app.status_message,
        "起動時GitHub同期完了: 1件追加 / 1件更新"
    );

    cleanup_app_file(&app);
}

#[test]
fn tick_applies_background_auto_push_result_to_meta() {
    let mut app = app_with_repos(vec![repo("solo", "2026-03-01T00:00:00Z", None)]);
    app.startup_jobs = StartupJobs::with_test_auto_push_result(Ok((
        AutoPushOutcome::UpToDate {
            repo: "cat2151/backups".to_string(),
            date: "2026-04-05".to_string(),
        },
        Some("2026-04-05".to_string()),
    )));

    app.tick();

    assert_eq!(app.data.meta.last_json_commit_push_date, "2026-04-05");
    assert_eq!(app.status_message, "JSON自動push: 変更なし");

    cleanup_app_file(&app);
}
