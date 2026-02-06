use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a schedule (8-char UUID prefix)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScheduleId(String);

impl ScheduleId {
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self(id[..8].to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        &self.0
    }
}

impl Default for ScheduleId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ScheduleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of a scheduled commit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleStatus {
    /// Waiting to be executed
    Pending,
    /// Currently being executed
    InProgress,
    /// Successfully committed
    Completed,
    /// Failed to commit (error stored in Schedule.error)
    Failed,
    /// Missed due to machine being asleep/off
    Missed,
}

impl std::fmt::Display for ScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Missed => write!(f, "missed"),
        }
    }
}

/// A scheduled commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Unique identifier
    pub id: ScheduleId,
    /// Commit message
    pub message: String,
    /// When to execute the commit
    pub scheduled_at: DateTime<Utc>,
    /// When the schedule was created
    pub created_at: DateTime<Utc>,
    /// Absolute path to the git repository
    pub repo_path: PathBuf,
    /// Branch name at schedule time
    pub branch: String,
    /// Path to the patch file
    pub patch_file: PathBuf,
    /// Whether to push after commit
    pub push_after: bool,
    /// Current status
    pub status: ScheduleStatus,
    /// Error message if failed
    pub error: Option<String>,
}

impl Schedule {
    pub fn new(
        message: String,
        scheduled_at: DateTime<Utc>,
        repo_path: PathBuf,
        branch: String,
        patch_file: PathBuf,
        push_after: bool,
    ) -> Self {
        Self {
            id: ScheduleId::new(),
            message,
            scheduled_at,
            created_at: Utc::now(),
            repo_path,
            branch,
            patch_file,
            push_after,
            status: ScheduleStatus::Pending,
            error: None,
        }
    }

    pub fn is_due(&self) -> bool {
        self.status == ScheduleStatus::Pending && Utc::now() >= self.scheduled_at
    }

    pub fn is_pending(&self) -> bool {
        self.status == ScheduleStatus::Pending
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, ScheduleStatus::Failed | ScheduleStatus::Missed)
    }
}

/// Daemon status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub pending_count: usize,
    pub failed_count: usize,
    pub next_schedule: Option<Schedule>,
}
