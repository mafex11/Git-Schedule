use anyhow::{Context, Result};
use std::path::PathBuf;

/// Maximum number of scheduled commits allowed
pub const MAX_SCHEDULES: usize = 10;

/// Maximum schedule time in hours
pub const MAX_SCHEDULE_HOURS: i64 = 24;

/// Get the base configuration directory (~/.git-schedule)
pub fn base_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".git-schedule"))
}

/// Get the schedules.json file path
pub fn schedules_file() -> Result<PathBuf> {
    Ok(base_dir()?.join("schedules.json"))
}

/// Get the patches directory
pub fn patches_dir() -> Result<PathBuf> {
    Ok(base_dir()?.join("patches"))
}

/// Get the logs directory
pub fn logs_dir() -> Result<PathBuf> {
    Ok(base_dir()?.join("logs"))
}

/// Get the daemon PID file path
pub fn pid_file() -> Result<PathBuf> {
    Ok(base_dir()?.join("daemon.pid"))
}

/// Get the Unix socket path for IPC
pub fn socket_path() -> Result<PathBuf> {
    Ok(base_dir()?.join("daemon.sock"))
}

/// Ensure all config directories exist
pub fn ensure_dirs() -> Result<()> {
    let dirs = [base_dir()?, patches_dir()?, logs_dir()?];
    for dir in dirs {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_dir() {
        let dir = base_dir().unwrap();
        assert!(dir.ends_with(".git-schedule"));
    }
}
