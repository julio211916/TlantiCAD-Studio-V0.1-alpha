//! Project snapshots for save/restore points

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A snapshot captures the full project state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    /// Serialized project state (meshes, parameters, etc.)
    pub data: Vec<u8>,
    /// Whether this is an auto-save snapshot
    pub auto_save: bool,
}

impl Snapshot {
    pub fn new(name: impl Into<String>, data: Vec<u8>, auto_save: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            created_at: Utc::now(),
            data,
            auto_save,
        }
    }

    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }
}

/// Manages a list of project snapshots
pub struct SnapshotManager {
    snapshots: Vec<Snapshot>,
    max_auto_saves: usize,
}

impl SnapshotManager {
    pub fn new(max_auto_saves: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_auto_saves,
        }
    }

    /// Create a named snapshot
    pub fn create(&mut self, name: impl Into<String>, data: Vec<u8>) -> Uuid {
        let snap = Snapshot::new(name, data, false);
        let id = snap.id;
        self.snapshots.push(snap);
        id
    }

    /// Create an auto-save snapshot, evicting oldest auto-saves if over limit
    pub fn auto_save(&mut self, data: Vec<u8>) -> Uuid {
        let snap = Snapshot::new("Auto-save", data, true);
        let id = snap.id;
        self.snapshots.push(snap);

        // Evict oldest auto-saves
        let auto_count = self.snapshots.iter().filter(|s| s.auto_save).count();
        if auto_count > self.max_auto_saves {
            let remove_count = auto_count - self.max_auto_saves;
            let mut removed = 0;
            self.snapshots.retain(|s| {
                if s.auto_save && removed < remove_count {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
        id
    }

    /// Get a snapshot by ID
    pub fn get(&self, id: &Uuid) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| &s.id == id)
    }

    /// List all snapshots (newest first)
    pub fn list(&self) -> Vec<&Snapshot> {
        let mut snaps: Vec<_> = self.snapshots.iter().collect();
        snaps.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        snaps
    }

    /// Delete a snapshot
    pub fn delete(&mut self, id: &Uuid) -> bool {
        let len_before = self.snapshots.len();
        self.snapshots.retain(|s| &s.id != id);
        self.snapshots.len() < len_before
    }

    /// Total size of all snapshots
    pub fn total_size(&self) -> usize {
        self.snapshots.iter().map(|s| s.size_bytes()).sum()
    }
}
