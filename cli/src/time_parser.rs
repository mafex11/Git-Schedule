use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local, NaiveTime, Utc};
use regex::Regex;

/// Maximum allowed schedule time (7 days)
const MAX_DAYS: i64 = 7;

/// Parse a relative time string like "2h", "30m", "1h30m", "2d", "1d12h", "3d6h30m"
pub fn parse_relative(s: &str) -> Result<DateTime<Utc>> {
    let s = s.trim().to_lowercase();

    let re = Regex::new(r"^(?:(\d+)\s*d)?\s*(?:(\d+)\s*h)?\s*(?:(\d+)\s*m)?$")?;

    let caps = re
        .captures(&s)
        .ok_or_else(|| anyhow!("Invalid time format. Use formats like '2h', '30m', '1d', '2d12h'"))?;

    let days: i64 = caps
        .get(1)
        .map(|m| m.as_str().parse().unwrap_or(0))
        .unwrap_or(0);
    let hours: i64 = caps
        .get(2)
        .map(|m| m.as_str().parse().unwrap_or(0))
        .unwrap_or(0);
    let minutes: i64 = caps
        .get(3)
        .map(|m| m.as_str().parse().unwrap_or(0))
        .unwrap_or(0);

    if days == 0 && hours == 0 && minutes == 0 {
        return Err(anyhow!(
            "Invalid time format. Use formats like '2h', '30m', '1d', '2d12h'"
        ));
    }

    let total_days = days as f64 + (hours as f64 / 24.0) + (minutes as f64 / 1440.0);
    if total_days > MAX_DAYS as f64 {
        return Err(anyhow!("Schedule time cannot exceed {} days", MAX_DAYS));
    }

    let duration = Duration::days(days) + Duration::hours(hours) + Duration::minutes(minutes);
    Ok(Utc::now() + duration)
}

/// Parse an absolute time string like "9:30am", "14:00", "monday 9am", "feb 10 9am"
pub fn parse_absolute(s: &str) -> Result<DateTime<Utc>> {
    let s = s.trim();
    let now = Local::now();

    // Try "weekday time" format: "monday 9am", "mon 9:30am", "thu 14:00"
    if let Ok(dt) = parse_weekday_time(s, &now) {
        return validate_max(dt, &now);
    }

    // Try plain time (today/tomorrow): "9:30am", "14:00"
    let time = parse_time_string(s)?;
    let today = now.date_naive();

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

    validate_max(scheduled, &now)
}

/// Parse "weekday time" like "monday 9am", "tue 14:00"
fn parse_weekday_time(s: &str, now: &DateTime<Local>) -> Result<DateTime<Local>> {
    use chrono::Datelike;

    let s_lower = s.to_lowercase();
    let parts: Vec<&str> = s_lower.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Not a weekday format"));
    }

    let target_weekday = match parts[0] {
        "mon" | "monday" => chrono::Weekday::Mon,
        "tue" | "tuesday" => chrono::Weekday::Tue,
        "wed" | "wednesday" => chrono::Weekday::Wed,
        "thu" | "thursday" => chrono::Weekday::Thu,
        "fri" | "friday" => chrono::Weekday::Fri,
        "sat" | "saturday" => chrono::Weekday::Sat,
        "sun" | "sunday" => chrono::Weekday::Sun,
        _ => return Err(anyhow!("Not a weekday")),
    };

    let time = parse_time_string(parts[1])?;
    let today = now.date_naive();
    let current_weekday = today.weekday();

    // Calculate days until target weekday
    let days_ahead = (target_weekday.num_days_from_monday() as i64
        - current_weekday.num_days_from_monday() as i64
        + 7) % 7;

    let target_date = today + Duration::days(days_ahead);
    let candidate = target_date
        .and_time(time)
        .and_local_timezone(Local)
        .single()
        .ok_or_else(|| anyhow!("Invalid time"))?;

    // If it's the same day but time has passed, push to next week
    if candidate <= *now {
        let next_week = target_date + Duration::days(7);
        return next_week
            .and_time(time)
            .and_local_timezone(Local)
            .single()
            .ok_or_else(|| anyhow!("Invalid time"));
    }

    Ok(candidate)
}

fn validate_max(scheduled: DateTime<Local>, now: &DateTime<Local>) -> Result<DateTime<Utc>> {
    let diff = scheduled - *now;
    if diff > Duration::days(MAX_DAYS) {
        return Err(anyhow!("Schedule time cannot exceed {} days", MAX_DAYS));
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

    let days = diff.num_days();
    let hours = diff.num_hours() % 24;
    let minutes = diff.num_minutes() % 60;

    if days > 0 {
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    } else if hours > 0 {
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

/// Format a DateTime for display (absolute local time, with date if not today)
pub fn format_absolute(dt: DateTime<Utc>) -> String {
    let local = dt.with_timezone(&Local);
    let now = Local::now();

    if local.date_naive() == now.date_naive() {
        local.format("%I:%M %p").to_string()
    } else {
        local.format("%a %b %d, %I:%M %p").to_string()
    }
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
    fn test_parse_relative_days() {
        let result = parse_relative("2d").unwrap();
        let diff = result - Utc::now();
        assert!(diff.num_hours() >= 47 && diff.num_hours() <= 49);
    }

    #[test]
    fn test_parse_relative_over_7d_rejected() {
        let result = parse_relative("8d");
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
