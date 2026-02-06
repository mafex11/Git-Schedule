use anyhow::{Context, Result};
use chrono::Utc;
use git_schedule_shared::{config, Schedule, ScheduleId, ScheduleStatus};
use std::collections::HashMap;
use std::path::PathBuf;

/// Storage for schedules, backed by JSON file
pub struct Storage {
    schedules: HashMap<String, Schedule>,
    file_path: PathBuf,
}

impl Storage {
    /// Create a new storage instance, loading existing schedules
    pub fn new() -> Result<Self> {
        let file_path = config::schedules_file()?;

        let schedules = if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;

            if content.trim().is_empty() {
                HashMap::new()
            } else {
                let list: Vec<Schedule> = serde_json::from_str(&content)
                    .with_context(|| "Failed to parse schedules.json")?;

                list.into_iter()
                    .map(|s| (s.id.as_str().to_string(), s))
                    .collect()
            }
        } else {
            HashMap::new()
        };

        Ok(Self {
            schedules,
            file_path,
        })
    }

    /// Save schedules to disk
    fn save(&self) -> Result<()> {
        let schedules: Vec<&Schedule> = self.schedules.values().collect();
        let content = serde_json::to_string_pretty(&schedules)?;

        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.file_path, content)
            .with_context(|| format!("Failed to write {}", self.file_path.display()))?;

        Ok(())
    }

    /// Add a new schedule
    pub fn add(&mut self, schedule: Schedule) -> Result<()> {
        let id = schedule.id.as_str().to_string();
        self.schedules.insert(id, schedule);
        self.save()
    }

    /// Get a schedule by ID
    pub fn get(&self, id: &ScheduleId) -> Option<&Schedule> {
        self.schedules.get(id.as_str())
    }

    /// Get a mutable reference to a schedule
    pub fn get_mut(&mut self, id: &ScheduleId) -> Option<&mut Schedule> {
        self.schedules.get_mut(id.as_str())
    }

    /// Remove a schedule
    pub fn remove(&mut self, id: &ScheduleId) -> Result<Option<Schedule>> {
        let removed = self.schedules.remove(id.as_str());
        self.save()?;
        Ok(removed)
    }

    /// Update a schedule
    pub fn update(&mut self, schedule: Schedule) -> Result<()> {
        let id = schedule.id.as_str().to_string();
        self.schedules.insert(id, schedule);
        self.save()
    }

    /// Get all pending schedules
    pub fn get_pending(&self) -> Vec<&Schedule> {
        self.schedules
            .values()
            .filter(|s| s.status == ScheduleStatus::Pending)
            .collect()
    }

    /// Get all schedules with a specific status
    pub fn get_by_status(&self, status: ScheduleStatus) -> Vec<&Schedule> {
        self.schedules
            .values()
            .filter(|s| s.status == status)
            .collect()
    }

    /// Get all schedules (optionally filtered by status)
    pub fn get_all(&self, status_filter: Option<ScheduleStatus>) -> Vec<&Schedule> {
        match status_filter {
            Some(status) => self.get_by_status(status),
            None => self.schedules.values().collect(),
        }
    }

    /// Count schedules by status
    pub fn count_by_status(&self, status: ScheduleStatus) -> usize {
        self.schedules
            .values()
            .filter(|s| s.status == status)
            .count()
    }

    /// Count pending schedules
    pub fn count_pending(&self) -> usize {
        self.count_by_status(ScheduleStatus::Pending)
    }

    /// Count failed/missed schedules
    pub fn count_failed(&self) -> usize {
        self.schedules
            .values()
            .filter(|s| matches!(s.status, ScheduleStatus::Failed | ScheduleStatus::Missed))
            .count()
    }

    /// Get the next scheduled commit (earliest pending)
    pub fn get_next(&self) -> Option<&Schedule> {
        self.get_pending()
            .into_iter()
            .min_by_key(|s| s.scheduled_at)
    }

    /// Mark schedules that missed their time as Missed
    /// Returns count of marked schedules
    pub fn mark_missed_schedules(&mut self) -> Result<usize> {
        let now = Utc::now();
        let mut count = 0;

        let ids_to_mark: Vec<String> = self
            .schedules
            .values()
            .filter(|s| s.status == ScheduleStatus::Pending && s.scheduled_at < now)
            .map(|s| s.id.as_str().to_string())
            .collect();

        for id in ids_to_mark {
            if let Some(schedule) = self.schedules.get_mut(&id) {
                schedule.status = ScheduleStatus::Missed;
                schedule.error = Some("Missed: machine was asleep or daemon not running".to_string());
                count += 1;
            }
        }

        if count > 0 {
            self.save()?;
        }

        Ok(count)
    }

    /// Clean up old completed schedules (older than 24 hours)
    pub fn cleanup_old(&mut self) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        let mut count = 0;

        let ids_to_remove: Vec<String> = self
            .schedules
            .values()
            .filter(|s| s.status == ScheduleStatus::Completed && s.scheduled_at < cutoff)
            .map(|s| s.id.as_str().to_string())
            .collect();

        for id in ids_to_remove {
            if let Some(schedule) = self.schedules.remove(&id) {
                // Also delete the patch file
                if schedule.patch_file.exists() {
                    let _ = std::fs::remove_file(&schedule.patch_file);
                }
                count += 1;
            }
        }

        if count > 0 {
            self.save()?;
        }

        Ok(count)
    }

    /// Clear all pending schedules, returns count of cleared
    pub fn clear_all_pending(&mut self) -> Result<usize> {
        let ids_to_remove: Vec<String> = self
            .schedules
            .values()
            .filter(|s| s.status == ScheduleStatus::Pending)
            .map(|s| s.id.as_str().to_string())
            .collect();

        let count = ids_to_remove.len();

        for id in ids_to_remove {
            if let Some(schedule) = self.schedules.remove(&id) {
                // Delete the patch file
                if schedule.patch_file.exists() {
                    let _ = std::fs::remove_file(&schedule.patch_file);
                }
            }
        }

        if count > 0 {
            self.save()?;
        }

        Ok(count)
    }

    /// Get the most recently created pending schedule
    pub fn get_most_recent_pending(&self) -> Option<&Schedule> {
        self.schedules
            .values()
            .filter(|s| s.status == ScheduleStatus::Pending)
            .max_by_key(|s| s.created_at)
    }

    /// Get completed schedules, sorted by completion time (most recent first)
    pub fn get_completed(&self, limit: usize) -> Vec<&Schedule> {
        let mut completed: Vec<&Schedule> = self
            .schedules
            .values()
            .filter(|s| s.status == ScheduleStatus::Completed)
            .collect();

        // Sort by scheduled_at descending (most recent first)
        completed.sort_by(|a, b| b.scheduled_at.cmp(&a.scheduled_at));

        completed.into_iter().take(limit).collect()
    }

    /// Count completed schedules
    pub fn count_completed(&self) -> usize {
        self.count_by_status(ScheduleStatus::Completed)
    }
}
