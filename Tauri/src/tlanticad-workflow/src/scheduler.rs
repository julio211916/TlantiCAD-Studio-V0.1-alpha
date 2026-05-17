//! S371-S375: Production Scheduler
//!
//! Machine scheduling, load balancing, and due-date optimization.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Job priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum JobPriority { Critical = 0, High = 1, Normal = 2, Low = 3 }

/// Scheduled job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub case_id: String,
    pub machine_id: String,
    pub priority: JobPriority,
    pub estimated_duration_min: f64,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
}

impl ScheduledJob {
    pub fn new(
        case_id: impl Into<String>,
        machine_id: impl Into<String>,
        duration_min: f64,
        start: DateTime<Utc>,
    ) -> Self {
        let end = start + Duration::minutes(duration_min as i64);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            case_id: case_id.into(),
            machine_id: machine_id.into(),
            priority: JobPriority::Normal,
            estimated_duration_min: duration_min,
            scheduled_start: start,
            scheduled_end: end,
            actual_start: None,
            actual_end: None,
        }
    }

    pub fn is_overdue(&self) -> bool {
        self.actual_end.is_none() && self.scheduled_end < Utc::now()
    }

    pub fn delay_min(&self) -> f64 {
        if let Some(actual) = self.actual_end {
            let diff = actual.signed_duration_since(self.scheduled_end);
            diff.num_minutes() as f64
        } else { 0.0 }
    }
}

/// Machine capacity slot
#[derive(Debug, Clone)]
pub struct MachineSlot {
    pub machine_id: String,
    pub available_from: DateTime<Utc>,
    pub capacity_hours_per_day: f64,
}

/// Production scheduler
#[derive(Debug, Clone, Default)]
pub struct Scheduler {
    pub jobs: Vec<ScheduledJob>,
    pub machines: Vec<MachineSlot>,
}

impl Scheduler {
    pub fn new() -> Self { Self::default() }

    pub fn add_machine(&mut self, id: impl Into<String>, capacity_hours: f64) {
        self.machines.push(MachineSlot {
            machine_id: id.into(),
            available_from: Utc::now(),
            capacity_hours_per_day: capacity_hours,
        });
    }

    /// Schedule a job on the least-loaded machine
    pub fn schedule_job(&mut self, case_id: impl Into<String>, duration_min: f64) -> Option<String> {
        // Find machine with earliest availability
        let slot = self.machines.iter_mut()
            .min_by_key(|m| m.available_from)?;

        let job = ScheduledJob::new(
            case_id,
            &slot.machine_id,
            duration_min,
            slot.available_from,
        );
        let job_id = job.id.clone();

        // Advance machine availability
        slot.available_from = job.scheduled_end;

        self.jobs.push(job);
        Some(job_id)
    }

    pub fn jobs_for_machine(&self, machine_id: &str) -> Vec<&ScheduledJob> {
        self.jobs.iter().filter(|j| j.machine_id == machine_id).collect()
    }

    pub fn pending_jobs(&self) -> Vec<&ScheduledJob> {
        self.jobs.iter().filter(|j| j.actual_end.is_none()).collect()
    }

    pub fn utilization_pct(&self, machine_id: &str, hours: f64) -> f64 {
        let total_min: f64 = self.jobs_for_machine(machine_id)
            .iter()
            .map(|j| j.estimated_duration_min)
            .sum();
        if hours > 0.0 { (total_min / (hours * 60.0) * 100.0).min(100.0) } else { 0.0 }
    }

    pub fn total_jobs(&self) -> usize { self.jobs.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_job() {
        let mut sched = Scheduler::new();
        sched.add_machine("mill-1", 8.0);
        let id = sched.schedule_job("C-001", 60.0);
        assert!(id.is_some());
        assert_eq!(sched.total_jobs(), 1);
    }

    #[test]
    fn test_multi_machine() {
        let mut sched = Scheduler::new();
        sched.add_machine("mill-1", 8.0);
        sched.add_machine("printer-1", 12.0);
        sched.schedule_job("C-001", 120.0);
        sched.schedule_job("C-002", 60.0);
        assert_eq!(sched.total_jobs(), 2);
    }

    #[test]
    fn test_utilization() {
        let mut sched = Scheduler::new();
        sched.add_machine("m1", 8.0);
        sched.schedule_job("C-1", 240.0); // 4 hours
        let util = sched.utilization_pct("m1", 8.0);
        assert!((util - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_pending_jobs() {
        let mut sched = Scheduler::new();
        sched.add_machine("m1", 8.0);
        sched.schedule_job("C-1", 30.0);
        sched.schedule_job("C-2", 30.0);
        assert_eq!(sched.pending_jobs().len(), 2);
    }
}
