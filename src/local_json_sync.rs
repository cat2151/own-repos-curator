use crate::paths::data_dir_path;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

const DATA_FILE_NAME: &str = "repos.json";
const LOCAL_DATA_REPO_DIR_NAME: &str = "own-repos-curator-data";

pub fn maybe_copy_json_to_local_data_repo(source_path: &Path, json: &str) -> Result<()> {
    let primary_data_path = data_dir_path()?.join(DATA_FILE_NAME);
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    maybe_copy_json_to_local_data_repo_with_paths(
        source_path,
        json,
        &primary_data_path,
        workspace_root,
    )
}

fn maybe_copy_json_to_local_data_repo_with_paths(
    source_path: &Path,
    json: &str,
    primary_data_path: &Path,
    workspace_root: &Path,
) -> Result<()> {
    if source_path != primary_data_path {
        return Ok(());
    }

    let Some(target_path) = local_data_repo_json_path(workspace_root)? else {
        return Ok(());
    };

    fs::write(&target_path, json).with_context(|| {
        format!(
            "failed to write local data repo json: {}",
            target_path.display()
        )
    })
}

fn local_data_repo_json_path(workspace_root: &Path) -> Result<Option<PathBuf>> {
    let Some(parent_dir) = workspace_root.parent() else {
        return Ok(None);
    };

    let data_repo_dir = parent_dir.join(LOCAL_DATA_REPO_DIR_NAME);
    if !data_repo_dir.is_dir() {
        return Ok(None);
    }

    Ok(Some(data_repo_dir.join(DATA_FILE_NAME)))
}

#[cfg(test)]
mod tests {
    use super::maybe_copy_json_to_local_data_repo_with_paths;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn copies_json_when_source_is_primary_data_path_and_sibling_repo_exists() {
        let fixture = TestFixture::new();
        let primary_data_path = fixture.app_data_dir.join("repos.json");
        let source_path = primary_data_path.clone();

        fs::create_dir_all(&fixture.app_data_dir).expect("app data dir should be created");
        fs::create_dir_all(&fixture.data_repo_dir).expect("data repo dir should be created");

        maybe_copy_json_to_local_data_repo_with_paths(
            &source_path,
            "{\"repos\":[]}",
            &primary_data_path,
            &fixture.workspace_root,
        )
        .expect("copy should succeed");

        let copied = fs::read_to_string(fixture.data_repo_dir.join("repos.json"))
            .expect("copied json should be readable");
        assert_eq!(copied, "{\"repos\":[]}");
    }

    #[test]
    fn skips_copy_when_sibling_repo_does_not_exist() {
        let fixture = TestFixture::new();
        let primary_data_path = fixture.app_data_dir.join("repos.json");

        fs::create_dir_all(&fixture.app_data_dir).expect("app data dir should be created");

        maybe_copy_json_to_local_data_repo_with_paths(
            &primary_data_path,
            "{\"repos\":[]}",
            &primary_data_path,
            &fixture.workspace_root,
        )
        .expect("missing sibling repo should be ignored");

        assert!(!fixture.data_repo_dir.join("repos.json").exists());
    }

    #[test]
    fn skips_copy_for_non_primary_data_path() {
        let fixture = TestFixture::new();
        let primary_data_path = fixture.app_data_dir.join("repos.json");
        let source_path = fixture.root.join("elsewhere").join("repos.json");

        fs::create_dir_all(&fixture.app_data_dir).expect("app data dir should be created");
        fs::create_dir_all(&fixture.data_repo_dir).expect("data repo dir should be created");
        fs::create_dir_all(source_path.parent().expect("source parent should exist"))
            .expect("source parent should be created");

        maybe_copy_json_to_local_data_repo_with_paths(
            &source_path,
            "{\"repos\":[]}",
            &primary_data_path,
            &fixture.workspace_root,
        )
        .expect("non-primary data path should be ignored");

        assert!(!fixture.data_repo_dir.join("repos.json").exists());
    }

    struct TestFixture {
        root: PathBuf,
        workspace_root: PathBuf,
        app_data_dir: PathBuf,
        data_repo_dir: PathBuf,
    }

    impl TestFixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be monotonic")
                .as_nanos();
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".test-local-data")
                .join(format!("local-json-sync-{unique}"));
            let workspace_root = root.join("own-repos-curator");
            let app_data_dir = root.join("app-data");
            let data_repo_dir = root.join("own-repos-curator-data");

            fs::create_dir_all(&workspace_root).expect("workspace root should be created");

            Self {
                root,
                workspace_root,
                app_data_dir,
                data_repo_dir,
            }
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
