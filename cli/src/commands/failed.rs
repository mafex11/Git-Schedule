use anyhow::Result;
use console::style;
use git_schedule_shared::ScheduleStatus;

use crate::client::{ensure_daemon_running, Client};
use crate::time_parser::format_absolute;

pub async fn run() -> Result<()> {
    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;

    // Get both failed and missed schedules
    let mut failed_schedules = client.list_schedules(Some(ScheduleStatus::Failed)).await?;
    let missed_schedules = client.list_schedules(Some(ScheduleStatus::Missed)).await?;
    failed_schedules.extend(missed_schedules);

    if failed_schedules.is_empty() {
        println!("{}", style("No failed commits.").dim());
        return Ok(());
    }

    println!("{}", style("Failed/Missed Commits").bold());
    println!("{}", style("─".repeat(60)).dim());

    for schedule in failed_schedules {
        let status_str = match schedule.status {
            ScheduleStatus::Failed => style("FAILED").red(),
            ScheduleStatus::Missed => style("MISSED").yellow(),
            _ => style("UNKNOWN").dim(),
        };

        println!(
            "{} {} {}",
            status_str,
            style(&schedule.id.short()).dim(),
            truncate(&schedule.message, 40)
        );

        let repo_name = schedule
            .repo_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!(
            "    {} {} @ {}",
            style("Repo:").dim(),
            repo_name,
            schedule.branch
        );
        println!(
            "    {} {}",
            style("Was scheduled for:").dim(),
            format_absolute(schedule.scheduled_at)
        );

        if let Some(error) = &schedule.error {
            println!("    {} {}", style("Error:").dim(), style(error).red());
        }

        println!();
    }

    println!(
        "{}",
        style("Use 'git-schedule retry <id>' to re-stage files and try again").dim()
    );

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
