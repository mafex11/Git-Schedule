use anyhow::Result;
use console::style;

use crate::client::{ensure_daemon_running, Client};

pub async fn run() -> Result<()> {
    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;
    let count = client.clear_all().await?;

    if count == 0 {
        println!("{} No pending schedules to clear", style("●").dim());
    } else {
        println!(
            "{} Cleared {} scheduled commit{}",
            style("✓").green(),
            count,
            if count == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
