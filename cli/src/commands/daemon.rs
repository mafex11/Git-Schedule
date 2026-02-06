use anyhow::{anyhow, Context, Result};
use console::style;
use git_schedule_shared::config;
use std::process::{Command, Stdio};

use crate::client::{is_daemon_running, Client};

pub async fn start() -> Result<()> {
    if is_daemon_running().await {
        println!("{} Daemon is already running", style("●").green());
        return Ok(());
    }

    start_daemon_process()?;

    // Wait for it to start
    for _ in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if is_daemon_running().await {
            println!("{} Daemon started", style("✓").green());
            return Ok(());
        }
    }

    Err(anyhow!("Failed to start daemon"))
}

pub async fn stop() -> Result<()> {
    if !is_daemon_running().await {
        println!("{} Daemon is not running", style("●").dim());
        return Ok(());
    }

    // Try graceful shutdown via socket
    if let Ok(mut client) = Client::connect().await {
        let _ = client.shutdown().await;
    }

    // Wait for it to stop
    for _ in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if !is_daemon_running().await {
            println!("{} Daemon stopped", style("✓").green());
            return Ok(());
        }
    }

    // Force kill via PID
    let pid_file = config::pid_file()?;
    if pid_file.exists() {
        let pid: i32 = std::fs::read_to_string(&pid_file)?.trim().parse()?;

        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output();
        }

        let _ = std::fs::remove_file(&pid_file);
    }

    // Clean up socket
    let socket_path = config::socket_path()?;
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    println!("{} Daemon stopped", style("✓").green());
    Ok(())
}

pub async fn restart() -> Result<()> {
    stop().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    start().await
}

/// Start the daemon process (detached)
pub fn start_daemon_process() -> Result<()> {
    // Ensure config directories exist
    config::ensure_dirs()?;

    // Find the daemon binary
    let daemon_path = find_daemon_binary()?;

    // Start as detached process
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        Command::new(&daemon_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0) // Detach from parent
            .spawn()
            .with_context(|| format!("Failed to start daemon: {}", daemon_path.display()))?;
    }

    #[cfg(not(unix))]
    {
        Command::new(&daemon_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to start daemon: {}", daemon_path.display()))?;
    }

    Ok(())
}

/// Find the daemon binary
fn find_daemon_binary() -> Result<std::path::PathBuf> {
    // Try same directory as CLI binary
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap();
        let daemon_path = exe_dir.join("git-schedule-daemon");
        if daemon_path.exists() {
            return Ok(daemon_path);
        }
    }

    // Try PATH
    if let Ok(output) = Command::new("which").arg("git-schedule-daemon").output() {
        if output.status.success() {
            let path = String::from_utf8(output.stdout)?;
            return Ok(std::path::PathBuf::from(path.trim()));
        }
    }

    // Try cargo target directory (for development)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dev_path = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("target/debug/git-schedule-daemon");
    if dev_path.exists() {
        return Ok(dev_path);
    }

    let release_path = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("target/release/git-schedule-daemon");
    if release_path.exists() {
        return Ok(release_path);
    }

    Err(anyhow!(
        "Could not find git-schedule-daemon binary. Make sure it's installed."
    ))
}
