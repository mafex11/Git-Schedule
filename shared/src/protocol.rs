use crate::types::{DaemonStatus, Schedule, ScheduleId, ScheduleStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request from CLI to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Create a new scheduled commit
    CreateSchedule {
        message: String,
        scheduled_at: DateTime<Utc>,
        repo_path: PathBuf,
        branch: String,
        patch_file: PathBuf,
        push_after: bool,
    },
    /// List schedules, optionally filtered by status
    ListSchedules {
        status_filter: Option<ScheduleStatus>,
    },
    /// Get a specific schedule by ID
    GetSchedule {
        id: ScheduleId,
    },
    /// Cancel a scheduled commit
    CancelSchedule {
        id: ScheduleId,
    },
    /// Update a schedule's message and/or time
    UpdateSchedule {
        id: ScheduleId,
        message: Option<String>,
        scheduled_at: Option<DateTime<Utc>>,
    },
    /// Get daemon status
    GetStatus,
    /// Retry a failed schedule (re-stage files)
    RetryFailed {
        id: ScheduleId,
    },
    /// Shutdown the daemon
    Shutdown,
    /// Ping to check if daemon is alive
    Ping,
}

/// Response from daemon to CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Response {
    /// Operation succeeded
    Ok,
    /// Single schedule returned
    Schedule(Schedule),
    /// List of schedules returned
    Schedules(Vec<Schedule>),
    /// Daemon status returned
    Status(DaemonStatus),
    /// Error occurred
    Error { message: String },
    /// Pong response to ping
    Pong,
    /// Schedule created, returns the new schedule
    Created(Schedule),
}

impl Response {
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

/// Serialize a request to JSON bytes with newline delimiter
pub fn serialize_request(req: &Request) -> anyhow::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(req)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Serialize a response to JSON bytes with newline delimiter
pub fn serialize_response(res: &Response) -> anyhow::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(res)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Deserialize a request from JSON bytes
pub fn deserialize_request(bytes: &[u8]) -> anyhow::Result<Request> {
    Ok(serde_json::from_slice(bytes)?)
}

/// Deserialize a response from JSON bytes
pub fn deserialize_response(bytes: &[u8]) -> anyhow::Result<Response> {
    Ok(serde_json::from_slice(bytes)?)
}
