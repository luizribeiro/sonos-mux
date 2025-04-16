use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn test_muxd_starts_and_stops() {
    // For Sprint 2, we'll just test that muxd can start and stop cleanly
    // In a real integration test, we would connect to the stream and verify data

    // Start muxd in a separate process
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "muxd",
            "--",
            "--config",
            "../examples/config.toml",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start muxd");

    // Give it some time to start
    thread::sleep(Duration::from_secs(2));

    // Check that the process is still running
    match child.try_wait() {
        Ok(None) => {
            // Process is still running, which is good
            println!("muxd is running successfully");
        }
        Ok(Some(status)) => {
            panic!("muxd exited prematurely with status: {}", status);
        }
        Err(e) => {
            panic!("Failed to check if muxd is running: {}", e);
        }
    }

    // Kill the process
    child.kill().expect("Failed to kill muxd process");

    // Wait for it to exit
    let status = child.wait().expect("Failed to wait for muxd to exit");

    // On Unix, exit code 9 indicates the process was killed by SIGKILL
    #[cfg(unix)]
    {
        assert!(
            status.code().is_none(),
            "Expected muxd to be killed by signal"
        );
    }

    // On Windows, we can't check the specific signal, so we just make sure it exited
    #[cfg(windows)]
    {
        assert!(
            !status.success(),
            "Expected muxd to exit with non-zero status"
        );
    }
}
