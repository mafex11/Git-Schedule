use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local, NaiveTime, Utc};
use regex::Regex;

/// Maximum allowed schedule time (24 hours)
const MAX_HOURS: i64 = 24;

/// Parse a relative time string like "2h", "30m", "1h30m"
pub fn parse_relative(s: &str) -> Result<DateTime<Utc>> {
    let s = s.trim().to_lowercase();

    // Match patterns like "2h", "30m", "1h30m", "1h 30m"
    let re = Regex::new(r"^(?:(\d+)\s*h)?\s*(?:(\d+)\s*m)?$")?;

    let caps = re
        .captures(&s)
        .ok_or_else(|| anyhow!("Invalid time format. Use formats like '2h', '30m', '1h30m'"))?;

    let hours: i64 = caps
        .get(1)
        .map(|m| m.as_str().parse().unwrap_or(0))
        .unwrap_or(0);
    let minutes: i64 = caps
        .get(2)
        .map(|m| m.as_str().parse().unwrap_or(0))
        .unwrap_or(0);

    if hours == 0 && minutes == 0 {
        return Err(anyhow!(
            "Invalid time format. Use formats like '2h', '30m', '1h30m'"
        ));
    }

    let total_hours = hours as f64 + (minutes as f64 / 60.0);
    if total_hours > MAX_HOURS as f64 {
        return Err(anyhow!("Schedule time cannot exceed {} hours", MAX_HOURS));
    }

    let duration = Duration::hours(hours) + Duration::minutes(minutes);
    Ok(Utc::now() + duration)
}

/// Parse an absolute time string like "9:30am", "14:00", "9:30 PM"
pub fn parse_absolute(s: &str) -> Result<DateTime<Utc>> {
    let s = s.trim();

    // Try various time formats
    let time = parse_time_string(s)?;

    // Get today's date in local timezone
    let now = Local::now();
    let today = now.date_naive();

    // Create datetime for today
    let local_datetime = today
        .and_time(time)
        .and_local_timezone(Local)
        .single()
        .ok_or_else(|| anyhow!("Invalid time"))?;

    // If the time has already passed today, schedule for tomorrow
    let scheduled = if local_datetime <= now {
        let tomorrow = today + Duration::days(1);
        tomorrow
            .and_time(time)
            .and_local_timezone(Local)
            .single()
            .ok_or_else(|| anyhow!("Invalid time"))?
    } else {
        local_datetime
    };

    // Check it's within 24 hours
    let diff = scheduled - now;
    if diff > Duration::hours(MAX_HOURS) {
        return Err(anyhow!("Schedule time cannot exceed {} hours", MAX_HOURS));
    }

    Ok(scheduled.with_timezone(&Utc))
}

/// Parse a time string into NaiveTime
fn parse_time_string(s: &str) -> Result<NaiveTime> {
    let s = s.trim().to_lowercase();

    // Try 12-hour format: "9:30am", "9:30 am", "9am", "9 am"
    let re_12h = Regex::new(r"^(\d{1,2})(?::(\d{2}))?\s*(am|pm)$")?;
    if let Some(caps) = re_12h.captures(&s) {
        let mut hour: u32 = caps.get(1).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(2).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
        let period = caps.get(3).unwrap().as_str();

        if hour > 12 || minute > 59 {
            return Err(anyhow!("Invalid time"));
        }

        // Convert to 24-hour
        if period == "am" {
            if hour == 12 {
                hour = 0;
            }
        } else {
            // pm
            if hour != 12 {
                hour += 12;
            }
        }

        return NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| anyhow!("Invalid time"));
    }

    // Try 24-hour format: "14:00", "9:30"
    let re_24h = Regex::new(r"^(\d{1,2}):(\d{2})$")?;
    if let Some(caps) = re_24h.captures(&s) {
        let hour: u32 = caps.get(1).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(2).unwrap().as_str().parse()?;

        if hour > 23 || minute > 59 {
            return Err(anyhow!("Invalid time"));
        }

        return NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| anyhow!("Invalid time"));
    }

    Err(anyhow!(
        "Invalid time format. Use formats like '9:30am', '14:00', '9am'"
    ))
}

/// Format a DateTime for display (relative to now)
pub fn format_relative(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = dt - now;

    if diff.num_seconds() < 0 {
        return "now".to_string();
    }

    let hours = diff.num_hours();
    let minutes = diff.num_minutes() % 60;

    if hours > 0 {
        if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        let seconds = diff.num_seconds();
        format!("{}s", seconds)
    }
}

/// Format a DateTime for display (absolute local time)
pub fn format_absolute(dt: DateTime<Utc>) -> String {
    let local = dt.with_timezone(&Local);
    local.format("%I:%M %p").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_relative_hours() {
        let result = parse_relative("2h").unwrap();
        let diff = result - Utc::now();
        assert!(diff.num_minutes() >= 119 && diff.num_minutes() <= 121);
    }

    #[test]
    fn test_parse_relative_minutes() {
        let result = parse_relative("30m").unwrap();
        let diff = result - Utc::now();
        assert!(diff.num_minutes() >= 29 && diff.num_minutes() <= 31);
    }

    #[test]
    fn test_parse_relative_mixed() {
        let result = parse_relative("1h30m").unwrap();
        let diff = result - Utc::now();
        assert!(diff.num_minutes() >= 89 && diff.num_minutes() <= 91);
    }

    #[test]
    fn test_parse_relative_over_24h_rejected() {
        let result = parse_relative("25h");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_absolute_am() {
        let result = parse_time_string("9:30am");
        assert!(result.is_ok());
        let time = result.unwrap();
        assert_eq!(time.hour(), 9);
        assert_eq!(time.minute(), 30);
    }

    #[test]
    fn test_parse_absolute_pm() {
        let result = parse_time_string("9:30pm");
        assert!(result.is_ok());
        let time = result.unwrap();
        assert_eq!(time.hour(), 21);
        assert_eq!(time.minute(), 30);
    }

    #[test]
    fn test_parse_absolute_24h() {
        let result = parse_time_string("14:00");
        assert!(result.is_ok());
        let time = result.unwrap();
        assert_eq!(time.hour(), 14);
        assert_eq!(time.minute(), 0);
    }
}
