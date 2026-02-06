use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

use crate::git::GitRepo;

/// Prompt user to select files to stage
pub fn select_files_to_stage(repo: &GitRepo) -> Result<Vec<String>> {
    let unstaged = repo.get_unstaged_files()?;

    if unstaged.is_empty() {
        return Err(anyhow!(
            "No modified or untracked files to stage.\n\
             Make some changes first, then run git-schedule."
        ));
    }

    println!(
        "{}",
        style("No files staged. Select files to stage:").yellow()
    );

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&unstaged)
        .interact()?;

    if selections.is_empty() {
        return Err(anyhow!("No files selected"));
    }

    let selected: Vec<String> = selections.iter().map(|&i| unstaged[i].clone()).collect();

    Ok(selected)
}
