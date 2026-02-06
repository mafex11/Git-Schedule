use anyhow::{Context, Result};
use git_schedule_shared::{
    config, deserialize_request, serialize_response, DaemonStatus, Request, Response, Schedule,
    ScheduleStatus,
};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
#[cfg(windows)]
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

use crate::SharedState;

/// Run the IPC server (Unix socket on Unix, TCP on Windows)
pub async fn run(state: SharedState) -> Result<()> {
    #[cfg(unix)]
    {
        let socket_path = config::socket_path()?;

        // Remove existing socket file
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        // Create listener
        let listener = UnixListener::bind(&socket_path)
            .with_context(|| format!("Failed to bind to {:?}", socket_path))?;

        // Set socket permissions (owner only)
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;

        info!("Server listening on {:?}", socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_unix_client(stream, state).await {
                            warn!("Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    #[cfg(windows)]
    {
        let addr = config::daemon_address();

        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind to {}", addr))?;

        info!("Server listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_tcp_client(stream, state).await {
                            warn!("Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }
}

/// Handle a single Unix socket client connection
#[cfg(unix)]
async fn handle_unix_client(stream: UnixStream, state: SharedState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read request
    reader.read_line(&mut line).await?;

    if line.is_empty() {
        return Ok(());
    }

    let request = deserialize_request(line.as_bytes())?;
    let response = handle_request(request, state).await;

    // Send response
    let bytes = serialize_response(&response)?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;

    Ok(())
}

/// Handle a single TCP client connection (Windows)
#[cfg(windows)]
async fn handle_tcp_client(stream: TcpStream, state: SharedState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read request
    reader.read_line(&mut line).await?;

    if line.is_empty() {
        return Ok(());
    }

    let request = deserialize_request(line.as_bytes())?;
    let response = handle_request(request, state).await;

    // Send response
    let bytes = serialize_response(&response)?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;

    Ok(())
}

/// Handle a request and return a response
async fn handle_request(request: Request, state: SharedState) -> Response {
    match request {
        Request::Ping => Response::Pong,

        Request::CreateSchedule {
            message,
            scheduled_at,
            repo_path,
            branch,
            patch_file,
            push_after,
        } => {
            let schedule = Schedule::new(
                message,
                scheduled_at,
                repo_path,
                branch,
                patch_file,
                push_after,
            );

            let mut state = state.write().await;

            // Check queue limit
            if state.storage.count_pending() >= config::MAX_SCHEDULES {
                return Response::error(format!(
                    "Queue full ({} schedules). Cancel some first.",
                    config::MAX_SCHEDULES
                ));
            }

            if let Err(e) = state.storage.add(schedule.clone()) {
                return Response::error(format!("Failed to save schedule: {}", e));
            }

            info!("Created schedule {}: {}", schedule.id, schedule.message);
            Response::Created(schedule)
        }

        Request::ListSchedules { status_filter } => {
            let state = state.read().await;
            let schedules: Vec<Schedule> = state
                .storage
                .get_all(status_filter)
                .into_iter()
                .cloned()
                .collect();
            Response::Schedules(schedules)
        }

        Request::GetSchedule { id } => {
            let state = state.read().await;
            match state.storage.get(&id) {
                Some(schedule) => Response::Schedule(schedule.clone()),
                None => Response::error(format!("Schedule not found: {}", id)),
            }
        }

        Request::CancelSchedule { id } => {
            let mut state = state.write().await;

            // Check if schedule exists and is pending
            if let Some(schedule) = state.storage.get(&id) {
                if schedule.status != ScheduleStatus::Pending {
                    return Response::error(format!(
                        "Cannot cancel schedule with status: {}",
                        schedule.status
                    ));
                }
            } else {
                return Response::error(format!("Schedule not found: {}", id));
            }

            match state.storage.remove(&id) {
                Ok(Some(_)) => {
                    info!("Cancelled schedule {}", id);
                    Response::Ok
                }
                Ok(None) => Response::error(format!("Schedule not found: {}", id)),
                Err(e) => Response::error(format!("Failed to cancel: {}", e)),
            }
        }

        Request::UpdateSchedule {
            id,
            message,
            scheduled_at,
        } => {
            let mut state = state.write().await;

            match state.storage.get_mut(&id) {
                Some(schedule) => {
                    if schedule.status != ScheduleStatus::Pending {
                        return Response::error(format!(
                            "Cannot update schedule with status: {}",
                            schedule.status
                        ));
                    }

                    if let Some(msg) = message {
                        schedule.message = msg;
                    }
                    if let Some(time) = scheduled_at {
                        schedule.scheduled_at = time;
                    }

                    let updated = schedule.clone();

                    if let Err(e) = state.storage.update(updated.clone()) {
                        return Response::error(format!("Failed to update: {}", e));
                    }

                    info!("Updated schedule {}", id);
                    Response::Schedule(updated)
                }
                None => Response::error(format!("Schedule not found: {}", id)),
            }
        }

        Request::GetStatus => {
            let state = state.read().await;
            let status = DaemonStatus {
                running: true,
                pid: Some(std::process::id()),
                uptime_seconds: Some(state.uptime_seconds()),
                pending_count: state.storage.count_pending(),
                failed_count: state.storage.count_failed(),
                next_schedule: state.storage.get_next().cloned(),
            };
            Response::Status(status)
        }

        Request::RetryFailed { id } => {
            let mut state = state.write().await;

            match state.storage.get(&id) {
                Some(schedule) => {
                    if !matches!(
                        schedule.status,
                        ScheduleStatus::Failed | ScheduleStatus::Missed
                    ) {
                        return Response::error(format!(
                            "Schedule {} is not failed or missed",
                            id
                        ));
                    }

                    let schedule_clone = schedule.clone();

                    // Remove from storage (patch file kept for CLI to apply)
                    if let Err(e) = state.storage.remove(&id) {
                        return Response::error(format!("Failed to remove: {}", e));
                    }

                    info!("Retry requested for schedule {}", id);
                    Response::Schedule(schedule_clone)
                }
                None => Response::error(format!("Schedule not found: {}", id)),
            }
        }

        Request::Shutdown => {
            info!("Shutdown requested");
            // Signal shutdown - the main loop will handle this
            std::process::exit(0);
        }
    }
}
