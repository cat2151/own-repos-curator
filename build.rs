fn main() {
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| output.status.success().then_some(output))
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|hash| !hash.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_COMMIT_HASH={hash}");

    const GIT_HEAD_PATH: &str = ".git/HEAD";
    const GIT_PACKED_REFS_PATH: &str = ".git/packed-refs";

    println!("cargo:rerun-if-changed={GIT_HEAD_PATH}");

    if let Ok(head) = std::fs::read_to_string(GIT_HEAD_PATH) {
        let head = head.trim();
        if let Some(ref_path) = head.strip_prefix("ref: ") {
            let ref_path = format!(".git/{ref_path}");
            if std::path::Path::new(&ref_path).exists() {
                println!("cargo:rerun-if-changed={ref_path}");
            }
        }
    }

    if std::path::Path::new(GIT_PACKED_REFS_PATH).exists() {
        println!("cargo:rerun-if-changed={GIT_PACKED_REFS_PATH}");
    }
}
