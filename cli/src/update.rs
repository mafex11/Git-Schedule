use chrono::{DateTime, Utc};
use console::style;
use dialoguer::Confirm;
use serde::{Deserialize, Serialize};
use std::fs;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL_SECS: i64 = 24 * 60 * 60;
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/mafex11/Git-Schedule/releases/latest";

#[derive(Serialize, Deserialize)]
struct UpdateCache {
    last_checked: DateTime<Utc>,
    latest_version: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
}

/// Check for updates. All errors are silently swallowed.
pub fn check_for_update() {
    let _ = check_for_update_inner();
}

fn check_for_update_inner() -> Option<()> {
    let cache_path = git_schedule_shared::config::update_check_file().ok()?;
    let cached = fs::read_to_string(&cache_path)
        .ok()
        .and_then(|s| serde_json::from_str::<UpdateCache>(&s).ok());

    match cached {
        Some(cache) if is_fresh(&cache) => {
            // Cache is fresh — show one-line notice if newer
            let latest = semver::Version::parse(&cache.latest_version).ok()?;
            let current = semver::Version::parse(CURRENT_VERSION).ok()?;
            if latest > current {
                show_update_notice(&cache.latest_version);
            }
        }
        _ => {
            // Cache stale or missing — fetch from GitHub
            let latest_version = fetch_latest_version()?;
            let cache = UpdateCache {
                last_checked: Utc::now(),
                latest_version: latest_version.clone(),
            };
            let _ = fs::write(&cache_path, serde_json::to_string(&cache).ok()?);

            let latest = semver::Version::parse(&latest_version).ok()?;
            let current = semver::Version::parse(CURRENT_VERSION).ok()?;
            if latest > current {
                show_update_prompt(&latest_version);
            }
        }
    }

    Some(())
}

fn is_fresh(cache: &UpdateCache) -> bool {
    let elapsed = Utc::now().signed_duration_since(cache.last_checked);
    elapsed.num_seconds() < CHECK_INTERVAL_SECS
}

fn fetch_latest_version() -> Option<String> {
    let body = ureq::agent()
        .get(GITHUB_API_URL)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .ok()?
        .into_string()
        .ok()?;

    let release: GithubRelease = serde_json::from_str(&body).ok()?;
    let version = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
    Some(version.to_string())
}

fn show_update_prompt(latest: &str) {
    println!(
        "\n{}",
        style(format!(
            "  A new version of git-schedule is available: v{} (current: v{})",
            latest, CURRENT_VERSION
        ))
        .yellow()
        .bold()
    );

    let show_instructions = Confirm::new()
        .with_prompt("  Would you like to see update instructions?")
        .default(false)
        .interact()
        .unwrap_or(false);

    if show_instructions {
        print_update_instructions();
    }
    println!();
}

fn show_update_notice(latest: &str) {
    println!(
        "{}",
        style(format!(
            "  Update available: v{} → v{} (run your package manager to update)",
            CURRENT_VERSION, latest
        ))
        .yellow()
    );
}

fn print_update_instructions() {
    #[cfg(target_os = "macos")]
    {
        println!("\n  Update via Homebrew:");
        println!(
            "    {}",
            style("brew upgrade mafex11/tap/git-schedule").green()
        );
    }

    #[cfg(target_os = "linux")]
    {
        println!("\n  Download the latest release:");
        println!(
            "    {}",
            style("https://github.com/mafex11/Git-Schedule/releases/latest").green()
        );
    }

    #[cfg(target_os = "windows")]
    {
        println!("\n  Update via PowerShell:");
        println!(
            "    {}",
            style("irm https://raw.githubusercontent.com/mafex11/git-schedule/main/install.ps1 | iex")
                .green()
        );
    }
}
