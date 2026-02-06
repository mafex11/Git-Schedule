use anyhow::Result;
use console::style;
use git_schedule_shared::ScheduleStatus;

use crate::client::{ensure_daemon_running, Client};
use crate::time_parser::{format_absolute, format_relative};

pub async fn run() -> Result<()> {
    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;
    let schedules = client.list_schedules(Some(ScheduleStatus::Pending)).await?;

    if schedules.is_empty() {
        println!("{}", style("No scheduled commits.").dim());
        println!();
        println!(
            "{}",
            style("Schedule one with: git-schedule \"message\" --in 2h").dim()
        );
        return Ok(());
    }

    println!("{}", style("Scheduled Commits").bold());
    println!("{}", style("─".repeat(60)).dim());

    for schedule in schedules {
        let status_icon = match schedule.status {
            ScheduleStatus::Pending => style("○").cyan(),
            ScheduleStatus::InProgress => style("◐").yellow(),
            _ => style("●").dim(),
        };

        let time_str = format!(
            "{} ({})",
            format_absolute(schedule.scheduled_at),
            format_relative(schedule.scheduled_at)
        );

        println!(
            "{} {} {} {}",
            status_icon,
            style(&schedule.id.short()).dim(),
            style(&time_str).green(),
            truncate(&schedule.message, 35)
        );

        let repo_name = schedule
            .repo_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let push_indicator = if schedule.push_after { " [push]" } else { "" };

        println!(
            "    {} @ {}{}",
            style(&repo_name).dim(),
            style(&schedule.branch).dim(),
            style(push_indicator).yellow()
        );
    }

    println!();
    println!(
        "{}",
        style("Use 'git-schedule show <id>' to view diff, 'git-schedule cancel <id>' to cancel")
            .dim()
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
