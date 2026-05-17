//! Project management: undo/redo command history, snapshots, persistence

pub mod command;
pub mod history;
pub mod snapshot;
pub mod persistence;

// AR-V406 — restoration / reconstruction-type changes
pub mod reconstruction_types;

pub use command::*;
pub use history::*;
pub use snapshot::*;
pub use persistence::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("No hay más operaciones para deshacer")]
    NothingToUndo,
    #[error("No hay más operaciones para rehacer")]
    NothingToRedo,
    #[error("Snapshot no encontrado: {0}")]
    SnapshotNotFound(String),
    #[error("Error de serialización: {0}")]
    Serialization(String),
    #[error("Error de IO: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Snapshot ────────────────────────────────────────────────
    #[test]
    fn test_snapshot_new() {
        let s = Snapshot::new("test", vec![1, 2, 3], false);
        assert_eq!(s.name, "test");
        assert_eq!(s.size_bytes(), 3);
        assert!(!s.auto_save);
    }

    // ── SnapshotManager ────────────────────────────────────────
    #[test]
    fn test_snapshot_manager_create() {
        let mut mgr = SnapshotManager::new(5);
        let id = mgr.create("snap1", vec![1, 2, 3]);
        assert!(mgr.get(&id).is_some());
    }

    #[test]
    fn test_snapshot_manager_list_delete() {
        let mut mgr = SnapshotManager::new(5);
        let id = mgr.create("s1", vec![1]);
        mgr.create("s2", vec![2]);
        assert_eq!(mgr.list().len(), 2);
        mgr.delete(&id);
        assert_eq!(mgr.list().len(), 1);
    }

    #[test]
    fn test_snapshot_manager_auto_save() {
        let mut mgr = SnapshotManager::new(2);
        mgr.auto_save(vec![1]);
        mgr.auto_save(vec![2]);
        mgr.auto_save(vec![3]);
        let auto_count = mgr.list().iter().filter(|s| s.auto_save).count();
        assert!(auto_count <= 2);
    }

    #[test]
    fn test_snapshot_manager_total_size() {
        let mut mgr = SnapshotManager::new(5);
        mgr.create("a", vec![1, 2, 3]);
        mgr.create("b", vec![4, 5]);
        assert_eq!(mgr.total_size(), 5);
    }

    // ── History ────────────────────────────────────────────────
    #[test]
    fn test_history_execute_undo_redo() {
        let mut h = History::new(10);
        let cmd = EditCommand::new(
            CommandKind::MoveVertices { vertex_ids: vec![0, 1], delta: [1.0, 0.0, 0.0] },
            "move",
        );
        h.execute(cmd);
        assert_eq!(h.undo_count(), 1);
        assert!(h.can_undo());
        assert!(!h.can_redo());

        h.undo().unwrap();
        assert_eq!(h.undo_count(), 0);
        assert_eq!(h.redo_count(), 1);

        h.redo().unwrap();
        assert_eq!(h.undo_count(), 1);
        assert_eq!(h.redo_count(), 0);
    }

    #[test]
    fn test_history_nothing_to_undo() {
        let mut h = History::new(10);
        assert!(h.undo().is_err());
    }

    #[test]
    fn test_history_nothing_to_redo() {
        let mut h = History::new(10);
        assert!(h.redo().is_err());
    }

    #[test]
    fn test_history_clear() {
        let mut h = History::new(10);
        let cmd = EditCommand::new(
            CommandKind::SetParameter {
                path: "crown.thickness".into(),
                old_value: serde_json::json!(0.5),
                new_value: serde_json::json!(0.6),
            },
            "param change",
        );
        h.execute(cmd);
        h.clear();
        assert_eq!(h.undo_count(), 0);
        assert_eq!(h.memory_usage(), 0);
    }

    // ── EditCommand ────────────────────────────────────────────
    #[test]
    fn test_edit_command_new() {
        let cmd = EditCommand::new(
            CommandKind::MoveVertices { vertex_ids: vec![0], delta: [0.0, 0.0, 1.0] },
            "move up",
        );
        assert_eq!(cmd.description, "move up");
        assert!(cmd.estimated_size() > 0);
    }

    // ── ProjectError ───────────────────────────────────────────
    #[test]
    fn test_error_display() {
        let e = ProjectError::NothingToUndo;
        assert!(format!("{}", e).contains("deshacer"));
    }
}
