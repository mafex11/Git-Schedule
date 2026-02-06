mod executor;
mod notification;
mod scheduler;
mod server;
mod storage;

use anyhow::Result;
use git_schedule_shared::config;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use storage::Storage;

/// Shared application state
pub struct AppState {
    pub storage: Storage,
    pub start_time: std::time::Instant,
}

impl AppState {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

pub type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let log_dir = config::logs_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "daemon.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(fmt::layer().with_writer(non_blocking))
        .init();

    info!("git-schedule daemon starting...");

    // Ensure config directories exist
    config::ensure_dirs()?;

    // Write PID file
    let pid = std::process::id();
    let pid_file = config::pid_file()?;
    std::fs::write(&pid_file, pid.to_string())?;
    info!("PID {} written to {:?}", pid, pid_file);

    // Initialize storage
    let storage = Storage::new()?;
    let state: SharedState = Arc::new(RwLock::new(AppState::new(storage)));

    // Check for missed schedules on startup
    {
        let mut state_guard = state.write().await;
        let missed = state_guard.storage.mark_missed_schedules()?;
        if missed > 0 {
            info!("{} schedules marked as missed", missed);
            notification::send_missed(missed);
        }
    }

    // Start the scheduler loop in background
    let scheduler_state = state.clone();
    let scheduler_handle = tokio::spawn(async move {
        if let Err(e) = scheduler::run(scheduler_state).await {
            error!("Scheduler error: {}", e);
        }
    });

    // Start the Unix socket server
    let server_state = state.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::run(server_state).await {
            error!("Server error: {}", e);
        }
    });

    info!("Daemon started successfully");

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = scheduler_handle => {
            error!("Scheduler task exited unexpectedly");
        }
        _ = server_handle => {
            error!("Server task exited unexpectedly");
        }
    }

    // Cleanup
    info!("Shutting down...");

    // Remove socket file (Unix only)
    #[cfg(unix)]
    {
        let socket_path = config::socket_path()?;
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }
    }

    if pid_file.exists() {
        std::fs::remove_file(&pid_file)?;
    }

    info!("Daemon stopped");
    Ok(())
}
