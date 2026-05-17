//! Async task queue for background AI processing

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// Status of an AI task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// An AI processing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTask {
    pub id: Uuid,
    pub pipeline_name: String,
    pub status: TaskStatus,
    pub progress: f32,  // 0.0 .. 1.0
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl AiTask {
    pub fn new(pipeline_name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            pipeline_name: pipeline_name.into(),
            status: TaskStatus::Queued,
            progress: 0.0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
}

/// Simple in-memory task queue
pub struct TaskQueue {
    pending: VecDeque<AiTask>,
    running: Vec<AiTask>,
    completed: Vec<AiTask>,
    max_concurrent: usize,
}

impl TaskQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            pending: VecDeque::new(),
            running: Vec::new(),
            completed: Vec::new(),
            max_concurrent,
        }
    }

    /// Submit a new task
    pub fn submit(&mut self, pipeline_name: impl Into<String>) -> Uuid {
        let task = AiTask::new(pipeline_name);
        let id = task.id;
        self.pending.push_back(task);
        id
    }

    /// Get the next task to run (if capacity allows)
    pub fn dequeue(&mut self) -> Option<&AiTask> {
        if self.running.len() >= self.max_concurrent {
            return None;
        }
        if let Some(mut task) = self.pending.pop_front() {
            task.status = TaskStatus::Running;
            task.started_at = Some(Utc::now());
            self.running.push(task);
            self.running.last()
        } else {
            None
        }
    }

    /// Mark a task as completed
    pub fn complete(&mut self, task_id: &Uuid) {
        if let Some(pos) = self.running.iter().position(|t| &t.id == task_id) {
            let mut task = self.running.remove(pos);
            task.status = TaskStatus::Completed;
            task.progress = 1.0;
            task.completed_at = Some(Utc::now());
            self.completed.push(task);
        }
    }

    /// Mark a task as failed
    pub fn fail(&mut self, task_id: &Uuid, error: impl Into<String>) {
        if let Some(pos) = self.running.iter().position(|t| &t.id == task_id) {
            let mut task = self.running.remove(pos);
            task.status = TaskStatus::Failed;
            task.error = Some(error.into());
            task.completed_at = Some(Utc::now());
            self.completed.push(task);
        }
    }

    /// Cancel a pending task
    pub fn cancel(&mut self, task_id: &Uuid) -> bool {
        if let Some(pos) = self.pending.iter().position(|t| &t.id == task_id) {
            let mut task = self.pending.remove(pos).unwrap();
            task.status = TaskStatus::Cancelled;
            task.completed_at = Some(Utc::now());
            self.completed.push(task);
            true
        } else {
            false
        }
    }

    /// Update progress of a running task
    pub fn update_progress(&mut self, task_id: &Uuid, progress: f32) {
        if let Some(task) = self.running.iter_mut().find(|t| &t.id == task_id) {
            task.progress = progress.clamp(0.0, 1.0);
        }
    }

    /// Get task by ID
    pub fn get(&self, task_id: &Uuid) -> Option<&AiTask> {
        self.running.iter()
            .chain(self.pending.iter())
            .chain(self.completed.iter())
            .find(|t| &t.id == task_id)
    }

    pub fn pending_count(&self) -> usize { self.pending.len() }
    pub fn running_count(&self) -> usize { self.running.len() }
    pub fn completed_count(&self) -> usize { self.completed.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_and_dequeue() {
        let mut q = TaskQueue::new(2);
        let id = q.submit("segmentation");
        assert_eq!(q.pending_count(), 1);
        assert_eq!(q.running_count(), 0);

        let task = q.dequeue().unwrap();
        assert_eq!(task.id, id);
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(q.pending_count(), 0);
        assert_eq!(q.running_count(), 1);
    }

    #[test]
    fn test_max_concurrent() {
        let mut q = TaskQueue::new(1);
        q.submit("a");
        q.submit("b");
        q.dequeue(); // first task starts
        assert!(q.dequeue().is_none()); // can't start second, at capacity
    }

    #[test]
    fn test_complete() {
        let mut q = TaskQueue::new(2);
        let id = q.submit("test");
        q.dequeue();
        q.complete(&id);
        assert_eq!(q.running_count(), 0);
        assert_eq!(q.completed_count(), 1);
        let task = q.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!((task.progress - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fail() {
        let mut q = TaskQueue::new(2);
        let id = q.submit("test");
        q.dequeue();
        q.fail(&id, "out of memory");
        let task = q.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error.as_deref(), Some("out of memory"));
    }

    #[test]
    fn test_cancel_pending() {
        let mut q = TaskQueue::new(2);
        let id = q.submit("test");
        assert!(q.cancel(&id));
        assert_eq!(q.pending_count(), 0);
        let task = q.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[test]
    fn test_cancel_running_fails() {
        let mut q = TaskQueue::new(2);
        let id = q.submit("test");
        q.dequeue(); // now running
        assert!(!q.cancel(&id)); // can't cancel running task
    }

    #[test]
    fn test_update_progress() {
        let mut q = TaskQueue::new(2);
        let id = q.submit("test");
        q.dequeue();
        q.update_progress(&id, 0.5);
        let task = q.get(&id).unwrap();
        assert!((task.progress - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_progress_clamp() {
        let mut q = TaskQueue::new(2);
        let id = q.submit("test");
        q.dequeue();
        q.update_progress(&id, 5.0);
        let task = q.get(&id).unwrap();
        assert!((task.progress - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_task_new() {
        let task = AiTask::new("my_pipeline");
        assert_eq!(task.pipeline_name, "my_pipeline");
        assert_eq!(task.status, TaskStatus::Queued);
        assert!((task.progress - 0.0).abs() < 1e-5);
        assert!(task.started_at.is_none());
    }
}
