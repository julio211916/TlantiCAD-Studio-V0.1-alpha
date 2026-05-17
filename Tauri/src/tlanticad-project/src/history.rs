//! Undo/redo history stack with memory limit

use crate::command::EditCommand;
use crate::ProjectError;

/// Manages the undo/redo stacks
pub struct History {
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    max_memory_bytes: usize,
    current_memory: usize,
}

impl History {
    pub fn new(max_memory_mb: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory: 0,
        }
    }

    /// Execute a command and push it onto the undo stack
    pub fn execute(&mut self, cmd: EditCommand) {
        let size = cmd.estimated_size();
        self.current_memory += size;
        self.undo_stack.push(cmd);
        self.redo_stack.clear();

        // Evict old commands if over memory limit
        while self.current_memory > self.max_memory_bytes && self.undo_stack.len() > 1 {
            let old = self.undo_stack.remove(0);
            self.current_memory -= old.estimated_size();
        }
    }

    /// Undo the last command, returning it
    pub fn undo(&mut self) -> Result<&EditCommand, ProjectError> {
        let cmd = self.undo_stack.pop().ok_or(ProjectError::NothingToUndo)?;
        self.redo_stack.push(cmd);
        Ok(self.redo_stack.last().unwrap())
    }

    /// Redo the last undone command, returning it
    pub fn redo(&mut self) -> Result<&EditCommand, ProjectError> {
        let cmd = self.redo_stack.pop().ok_or(ProjectError::NothingToRedo)?;
        self.undo_stack.push(cmd);
        Ok(self.undo_stack.last().unwrap())
    }

    /// Number of undoable steps
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of redoable steps
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_memory = 0;
    }

    /// Current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Get all commands in the undo stack (oldest first)
    pub fn commands(&self) -> &[EditCommand] {
        &self.undo_stack
    }
}
