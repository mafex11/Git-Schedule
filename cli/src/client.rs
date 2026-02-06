use anyhow::{anyhow, Context, Result};
use git_schedule_shared::{
    config, deserialize_response, serialize_request, DaemonStatus, Request, Response, Schedule,
    ScheduleId, ScheduleStatus,
};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Client for communicating with the daemon
pub struct Client;

impl Client {
    /// Connect to the daemon (placeholder for API compatibility)
    pub async fn connect() -> Result<Self> {
        // Just verify the socket exists
        let socket_path = config::socket_path()?;
        if !socket_path.exists() {
            return Err(anyhow!("Daemon socket not found"));
        }
        Ok(Self)
    }

    /// Send a request and receive a response (creates new connection each time)
    async fn request(&mut self, req: Request) -> Result<Response> {
        let socket_path = config::socket_path()?;

        // Create new connection for each request
        let mut stream = timeout(CONNECT_TIMEOUT, UnixStream::connect(&socket_path))
            .await
            .context("Connection timed out")?
            .with_context(|| format!("Failed to connect to daemon at {:?}", socket_path))?;

        let bytes = serialize_request(&req)?;

        // Send request and flush
        stream.write_all(&bytes).await?;
        stream.flush().await?;

        // Read response
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();

        timeout(REQUEST_TIMEOUT, reader.read_line(&mut line))
            .await
            .context("Request timed out")?
            .context("Failed to read response")?;

        if line.is_empty() {
            return Err(anyhow!("Empty response from daemon"));
        }

        let response = deserialize_response(line.as_bytes())?;
        Ok(response)
    }

    /// Create a new scheduled commit
    pub async fn create_schedule(
        &mut self,
        message: String,
        scheduled_at: DateTime<Utc>,
        repo_path: PathBuf,
        branch: String,
        patch_file: PathBuf,
        push_after: bool,
    ) -> Result<Schedule> {
        let req = Request::CreateSchedule {
            message,
            scheduled_at,
            repo_path,
            branch,
            patch_file,
            push_after,
        };

        match self.request(req).await? {
            Response::Created(schedule) => Ok(schedule),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// List all schedules, optionally filtered by status
    pub async fn list_schedules(
        &mut self,
        status_filter: Option<ScheduleStatus>,
    ) -> Result<Vec<Schedule>> {
        let req = Request::ListSchedules { status_filter };

        match self.request(req).await? {
            Response::Schedules(schedules) => Ok(schedules),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Get a specific schedule by ID
    pub async fn get_schedule(&mut self, id: &str) -> Result<Schedule> {
        let req = Request::GetSchedule {
            id: ScheduleId::from_string(id.to_string()),
        };

        match self.request(req).await? {
            Response::Schedule(schedule) => Ok(schedule),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Cancel a scheduled commit
    pub async fn cancel_schedule(&mut self, id: &str) -> Result<()> {
        let req = Request::CancelSchedule {
            id: ScheduleId::from_string(id.to_string()),
        };

        match self.request(req).await? {
            Response::Ok => Ok(()),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Update a schedule
    pub async fn update_schedule(
        &mut self,
        id: &str,
        message: Option<String>,
        scheduled_at: Option<DateTime<Utc>>,
    ) -> Result<Schedule> {
        let req = Request::UpdateSchedule {
            id: ScheduleId::from_string(id.to_string()),
            message,
            scheduled_at,
        };

        match self.request(req).await? {
            Response::Schedule(schedule) => Ok(schedule),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Get daemon status
    pub async fn get_status(&mut self) -> Result<DaemonStatus> {
        let req = Request::GetStatus;

        match self.request(req).await? {
            Response::Status(status) => Ok(status),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Retry a failed schedule
    pub async fn retry_failed(&mut self, id: &str) -> Result<Schedule> {
        let req = Request::RetryFailed {
            id: ScheduleId::from_string(id.to_string()),
        };

        match self.request(req).await? {
            Response::Schedule(schedule) => Ok(schedule),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Request daemon shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        let req = Request::Shutdown;

        match self.request(req).await? {
            Response::Ok => Ok(()),
            Response::Error { message } => Err(anyhow!(message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Ping the daemon
    pub async fn ping(&mut self) -> Result<()> {
        let req = Request::Ping;

        match self.request(req).await? {
            Response::Pong => Ok(()),
            _ => Err(anyhow!("Unexpected response")),
        }
    }
}

/// Check if daemon is running
pub async fn is_daemon_running() -> bool {
    if let Ok(mut client) = Client::connect().await {
        client.ping().await.is_ok()
    } else {
        false
    }
}

/// Ensure daemon is running, start if not
pub async fn ensure_daemon_running() -> Result<()> {
    if is_daemon_running().await {
        return Ok(());
    }

    // Start daemon
    crate::commands::daemon::start_daemon_process()?;

    // Wait for it to be ready
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if is_daemon_running().await {
            return Ok(());
        }
    }

    Err(anyhow!("Failed to start daemon"))
}
