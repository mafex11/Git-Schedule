use notify_rust::{Notification, Timeout};
use std::path::Path;
use tracing::warn;

const APP_NAME: &str = "git-schedule";

/// Send a success notification
pub fn send_success(message: &str, repo_path: &Path) {
    let repo_name = repo_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "repository".to_string());

    send(
        "Commit Successful",
        &format!("{}\n{}", truncate(message, 50), repo_name),
        false,
    );
}

/// Send a failure notification
pub fn send_failure(message: &str, error: &str) {
    send(
        "Commit Failed",
        &format!("{}\n{}", truncate(message, 40), truncate(error, 60)),
        true,
    );
}

/// Send a notification about missed schedules
pub fn send_missed(count: usize) {
    send(
        "Missed Schedules",
        &format!(
            "{} commit(s) missed while away.\nRun 'git-schedule failed' to retry.",
            count
        ),
        true,
    );
}

/// Send a system notification
fn send(title: &str, body: &str, _is_error: bool) {
    let result = Notification::new()
        .summary(title)
        .body(body)
        .appname(APP_NAME)
        .timeout(Timeout::Milliseconds(5000))
        .show();

    if let Err(e) = result {
        warn!("Failed to send notification: {}", e);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
