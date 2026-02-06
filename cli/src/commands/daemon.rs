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
        let pid: u32 = std::fs::read_to_string(&pid_file)?.trim().parse()?;

        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output();
        }

        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output();
        }

        let _ = std::fs::remove_file(&pid_file);
    }

    // Clean up socket (Unix only)
    #[cfg(unix)]
    {
        let socket_path = config::socket_path()?;
        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }
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

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;

        Command::new(&daemon_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .spawn()
            .with_context(|| format!("Failed to start daemon: {}", daemon_path.display()))?;
    }

    Ok(())
}

/// Find the daemon binary
fn find_daemon_binary() -> Result<std::path::PathBuf> {
    #[cfg(unix)]
    let daemon_name = "git-schedule-daemon";
    #[cfg(windows)]
    let daemon_name = "git-schedule-daemon.exe";

    // Try same directory as CLI binary
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap();
        let daemon_path = exe_dir.join(daemon_name);
        if daemon_path.exists() {
            return Ok(daemon_path);
        }
    }

    // Try PATH (use "which" on Unix, "where" on Windows)
    #[cfg(unix)]
    let path_cmd = "which";
    #[cfg(windows)]
    let path_cmd = "where";

    if let Ok(output) = Command::new(path_cmd).arg(daemon_name).output() {
        if output.status.success() {
            let path = String::from_utf8(output.stdout)?;
            // On Windows, "where" may return multiple paths, take the first one
            let first_path = path.lines().next().unwrap_or("").trim();
            if !first_path.is_empty() {
                return Ok(std::path::PathBuf::from(first_path));
            }
        }
    }

    // Try cargo target directory (for development)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    #[cfg(unix)]
    {
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
    }

    #[cfg(windows)]
    {
        let dev_path = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .join("target\\debug\\git-schedule-daemon.exe");
        if dev_path.exists() {
            return Ok(dev_path);
        }

        let release_path = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .join("target\\release\\git-schedule-daemon.exe");
        if release_path.exists() {
            return Ok(release_path);
        }
    }

    Err(anyhow!(
        "Could not find git-schedule-daemon binary. Make sure it's installed."
    ))
}
