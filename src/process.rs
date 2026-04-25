use anyhow::{anyhow, Context, Result};
use std::{
    io::{self, Read},
    process::{Command, Output, Stdio},
    thread::{self, JoinHandle},
    time::Duration,
};

const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) fn output_with_timeout(
    command: &mut Command,
    timeout: Duration,
    label: &str,
) -> Result<Output> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to start {label}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture stdout for {label}"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to capture stderr for {label}"))?;
    let stdout_reader = spawn_pipe_reader(stdout);
    let stderr_reader = spawn_pipe_reader(stderr);

    let started = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("failed to wait for {label}"))?
        {
            break status;
        }

        let elapsed = started.elapsed();
        if elapsed >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            let stderr = join_pipe_reader_for_timeout(stderr_reader);
            let stderr = String::from_utf8_lossy(&stderr).trim().to_string();
            if stderr.is_empty() {
                return Err(anyhow!(
                    "{label} timed out after {} seconds",
                    timeout.as_secs()
                ));
            }
            return Err(anyhow!(
                "{label} timed out after {} seconds: {stderr}",
                timeout.as_secs()
            ));
        }

        thread::sleep(POLL_INTERVAL.min(timeout.saturating_sub(elapsed)));
    };

    Ok(Output {
        status,
        stdout: join_pipe_reader(stdout_reader, label, "stdout")?,
        stderr: join_pipe_reader(stderr_reader, label, "stderr")?,
    })
}

fn spawn_pipe_reader<R>(mut pipe: R) -> JoinHandle<io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = Vec::new();
        pipe.read_to_end(&mut output)?;
        Ok(output)
    })
}

fn join_pipe_reader(
    reader: JoinHandle<io::Result<Vec<u8>>>,
    label: &str,
    stream_name: &str,
) -> Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| anyhow!("{label} {stream_name} reader panicked"))?
        .with_context(|| format!("failed to read {stream_name} from {label}"))
}

fn join_pipe_reader_for_timeout(reader: JoinHandle<io::Result<Vec<u8>>>) -> Vec<u8> {
    reader.join().ok().and_then(Result::ok).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::output_with_timeout;
    use std::{process::Command, time::Duration};

    const SLEEP_CHILD_ENV: &str = "OWN_REPOS_CURATOR_TIMEOUT_TEST_CHILD";

    #[test]
    fn output_with_timeout_reports_timeout() {
        let mut command = Command::new(std::env::current_exe().expect("test exe path"));
        command
            .arg("--exact")
            .arg("process::tests::timeout_test_child")
            .env(SLEEP_CHILD_ENV, "1");

        let error = output_with_timeout(
            &mut command,
            Duration::from_millis(100),
            "test sleep command",
        )
        .expect_err("sleeping child should time out");

        assert!(error.to_string().contains("test sleep command timed out"));
    }

    #[test]
    fn timeout_test_child() {
        if std::env::var_os(SLEEP_CHILD_ENV).is_some() {
            std::thread::sleep(Duration::from_secs(5));
        }
    }
}
