//! Event system for inter-component communication

use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::{EntityId, ProcessingStatus};

/// Application events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    // Project events
    ProjectCreated { id: EntityId, name: String },
    ProjectOpened { id: EntityId },
    ProjectSaved { id: EntityId },
    ProjectClosed { id: EntityId },

    // Mesh events
    MeshLoaded { id: EntityId, name: String },
    MeshModified { id: EntityId },
    MeshDeleted { id: EntityId },

    // ML events
    ModelLoaded { id: EntityId, name: String },
    InferenceStarted { model_id: EntityId },
    InferenceCompleted { model_id: EntityId, duration_ms: u64 },
    InferenceFailed { model_id: EntityId, error: String },

    // Processing events
    ProcessingStatusChanged { status: ProcessingStatus },

    // Python sidecar events
    PythonScriptStarted { script: String },
    PythonScriptCompleted { script: String, output: String },
    PythonScriptFailed { script: String, error: String },

    // System events
    Error { message: String },
    Warning { message: String },
    Info { message: String },
}

/// Event bus for broadcasting events
pub struct EventBus {
    sender: Sender<AppEvent>,
    #[allow(dead_code)]
    receiver: Receiver<AppEvent>,
    subscribers: Arc<RwLock<Vec<Sender<AppEvent>>>>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self {
            sender,
            receiver,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to events
    pub async fn subscribe(&self) -> Receiver<AppEvent> {
        let (tx, rx) = bounded(100);
        self.subscribers.write().await.push(tx);
        rx
    }

    /// Publish an event
    pub fn publish(&self, event: AppEvent) -> Result<(), crossbeam_channel::SendError<AppEvent>> {
        self.sender.send(event)
    }

    /// Get the sender for publishing events
    pub fn sender(&self) -> Sender<AppEvent> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000)
    }
}
