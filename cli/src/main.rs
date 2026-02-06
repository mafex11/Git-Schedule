mod client;
mod commands;
mod git;
mod interactive;
mod time_parser;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "git-schedule")]
#[command(author, version, about = "Schedule git commits for later")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Commit message (when scheduling directly)
    #[arg(value_name = "MESSAGE")]
    message: Option<String>,

    /// Schedule commit in relative time (e.g., "2h", "30m", "1h30m")
    #[arg(long = "in", value_name = "TIME")]
    in_time: Option<String>,

    /// Schedule commit at absolute time (e.g., "9:30am", "14:00")
    #[arg(long = "at", value_name = "TIME")]
    at_time: Option<String>,

    /// Push to remote after commit
    #[arg(long, short)]
    push: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all scheduled commits
    List {
        /// Show only failed/missed commits
        #[arg(long)]
        failed: bool,
    },

    /// Show daemon status and next scheduled commit
    Status,

    /// Cancel a scheduled commit
    Cancel {
        /// Schedule ID to cancel
        id: String,
    },

    /// Edit a scheduled commit's message or time
    Edit {
        /// Schedule ID to edit
        id: String,

        /// New commit message
        #[arg(long, short)]
        message: Option<String>,

        /// New relative time (e.g., "2h", "30m")
        #[arg(long = "in")]
        in_time: Option<String>,

        /// New absolute time (e.g., "9:30am", "14:00")
        #[arg(long = "at")]
        at_time: Option<String>,
    },

    /// Show the diff that will be committed
    Show {
        /// Schedule ID to show
        id: String,
    },

    /// List failed/missed commits
    Failed,

    /// Retry a failed commit (re-stages files for rescheduling)
    Retry {
        /// Schedule ID to retry
        id: String,
    },

    /// Manage the background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Restart the daemon
    Restart,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // Subcommands
        Some(Commands::List { failed }) => {
            if failed {
                commands::failed::run().await
            } else {
                commands::list::run().await
            }
        }
        Some(Commands::Status) => commands::status::run().await,
        Some(Commands::Cancel { id }) => commands::cancel::run(&id).await,
        Some(Commands::Edit {
            id,
            message,
            in_time,
            at_time,
        }) => commands::edit::run(&id, message, in_time, at_time).await,
        Some(Commands::Show { id }) => commands::show::run(&id).await,
        Some(Commands::Failed) => commands::failed::run().await,
        Some(Commands::Retry { id }) => commands::retry::run(&id).await,
        Some(Commands::Daemon { action }) => match action {
            DaemonAction::Start => commands::daemon::start().await,
            DaemonAction::Stop => commands::daemon::stop().await,
            DaemonAction::Restart => commands::daemon::restart().await,
        },

        // Direct schedule command: git-schedule "message" --in 2h
        None => {
            if let Some(message) = cli.message {
                commands::schedule::run(message, cli.in_time, cli.at_time, cli.push).await
            } else {
                // No message provided, show help
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
