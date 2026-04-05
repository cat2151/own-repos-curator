use anyhow::{Context, Result};
use std::process::{Command, Stdio};

const HATENA_SYNC_EXE: &str = if cfg!(windows) {
    "own-repos-curator-to-hatena.exe"
} else {
    "own-repos-curator-to-hatena"
};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) fn spawn_hatena_sync() -> Result<()> {
    spawn_background_process(HATENA_SYNC_EXE)
}

fn spawn_background_process(program: &str) -> Result<()> {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .spawn()
        .with_context(|| format!("failed to start background data linkage process: {program}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::spawn_background_process;

    #[test]
    fn missing_background_process_reports_program_name() {
        let program = "definitely-missing-own-repos-curator-to-hatena-test";
        let error = spawn_background_process(program).expect_err("spawn should fail");
        let message = error.to_string();
        assert!(message.contains("background data linkage process"));
        assert!(message.contains(program));
    }
}
