use anyhow::Result;
use chrono::Utc;
use git_schedule_shared::ScheduleStatus;
use std::time::Duration;
use tracing::{error, info};

use crate::executor;
use crate::notification;
use crate::SharedState;

/// Run the scheduler loop
pub async fn run(state: SharedState) -> Result<()> {
    info!("Scheduler started");

    loop {
        // Check for due schedules
        let due_schedules = {
            let state = state.read().await;
            let now = Utc::now();

            state
                .storage
                .get_pending()
                .into_iter()
                .filter(|s| s.scheduled_at <= now)
                .cloned()
                .collect::<Vec<_>>()
        };

        // Execute due schedules
        for schedule in due_schedules {
            info!("Executing schedule {}: {}", schedule.id, schedule.message);

            // Mark as in progress
            {
                let mut state = state.write().await;
                let updated = state.storage.get(&schedule.id).map(|s| {
                    let mut updated = s.clone();
                    updated.status = ScheduleStatus::InProgress;
                    updated
                });
                if let Some(updated) = updated {
                    let _ = state.storage.update(updated);
                }
            }

            // Execute the commit
            let result = executor::execute_schedule(&schedule).await;

            // Update status based on result
            {
                let mut state = state.write().await;
                let updated = state.storage.get(&schedule.id).map(|s| {
                    let mut updated = s.clone();
                    match &result {
                        Ok(_) => {
                            updated.status = ScheduleStatus::Completed;
                            updated.error = None;
                        }
                        Err(e) => {
                            updated.status = ScheduleStatus::Failed;
                            updated.error = Some(e.to_string());
                        }
                    }
                    updated
                });
                if let Some(updated) = updated {
                    let _ = state.storage.update(updated);
                }
            }

            // Send notification outside of lock
            match &result {
                Ok(_) => {
                    info!("Schedule {} completed successfully", schedule.id);
                    notification::send_success(&schedule.message, &schedule.repo_path);
                }
                Err(e) => {
                    error!("Schedule {} failed: {}", schedule.id, e);
                    notification::send_failure(&schedule.message, &e.to_string());
                }
            }
        }

        // Periodic cleanup of old completed schedules
        {
            let mut state = state.write().await;
            if let Ok(count) = state.storage.cleanup_old() {
                if count > 0 {
                    info!("Cleaned up {} old completed schedules", count);
                }
            }
        }

        // Calculate sleep duration until next schedule
        let sleep_duration = {
            let state = state.read().await;
            calculate_next_wake(&state.storage)
        };

        tokio::time::sleep(sleep_duration).await;
    }
}

/// Calculate how long to sleep until the next schedule
fn calculate_next_wake(storage: &crate::storage::Storage) -> Duration {
    const MAX_SLEEP: Duration = Duration::from_secs(60);
    const MIN_SLEEP: Duration = Duration::from_secs(1);

    if let Some(next) = storage.get_next() {
        let now = Utc::now();
        let until = next.scheduled_at - now;

        if until.num_seconds() <= 0 {
            MIN_SLEEP
        } else {
            let secs = until.num_seconds() as u64;
            Duration::from_secs(secs.min(MAX_SLEEP.as_secs()))
        }
    } else {
        // No pending schedules, sleep for max duration
        MAX_SLEEP
    }
}
