//! S349-S350: Manufacturing Integration & Workflow
//!
//! End-to-end manufacturing workflow orchestration.

use serde::{Deserialize, Serialize};
use crate::milling::MillingAxes;
use crate::printing::PrintTechnology;

/// Manufacturing method choice
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManufMethod {
    CncMilling(MillingAxes),
    Print3D(PrintTechnology),
    Casting,
    Pressing,
}

/// Workflow stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowStage {
    DesignReceived,
    NestingQueued,
    CamPrepared,
    Manufacturing,
    PostProcessing,
    QualityCheck,
    Shipping,
    Delivered,
}

/// Manufacturing order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufOrder {
    pub id: String,
    pub method: ManufMethod,
    pub stage: WorkflowStage,
    pub parts: Vec<String>,
    pub priority: u8,
}

impl ManufOrder {
    pub fn new(method: ManufMethod, parts: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            method,
            stage: WorkflowStage::DesignReceived,
            parts,
            priority: 5,
        }
    }

    pub fn advance(&mut self) {
        self.stage = match self.stage {
            WorkflowStage::DesignReceived => WorkflowStage::NestingQueued,
            WorkflowStage::NestingQueued => WorkflowStage::CamPrepared,
            WorkflowStage::CamPrepared => WorkflowStage::Manufacturing,
            WorkflowStage::Manufacturing => WorkflowStage::PostProcessing,
            WorkflowStage::PostProcessing => WorkflowStage::QualityCheck,
            WorkflowStage::QualityCheck => WorkflowStage::Shipping,
            WorkflowStage::Shipping => WorkflowStage::Delivered,
            WorkflowStage::Delivered => WorkflowStage::Delivered,
        };
    }

    pub fn is_complete(&self) -> bool {
        self.stage == WorkflowStage::Delivered
    }
}

/// Queue for manufacturing orders
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManufQueue {
    pub orders: Vec<ManufOrder>,
}

impl ManufQueue {
    pub fn new() -> Self { Self { orders: Vec::new() } }

    pub fn enqueue(&mut self, order: ManufOrder) {
        self.orders.push(order);
        self.orders.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    pub fn pending(&self) -> Vec<&ManufOrder> {
        self.orders.iter().filter(|o| !o.is_complete()).collect()
    }

    pub fn total(&self) -> usize { self.orders.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_advance() {
        let mut order = ManufOrder::new(
            ManufMethod::CncMilling(MillingAxes::FiveAxis),
            vec!["Crown14".into()],
        );
        assert_eq!(order.stage, WorkflowStage::DesignReceived);
        order.advance();
        assert_eq!(order.stage, WorkflowStage::NestingQueued);
        for _ in 0..6 { order.advance(); }
        assert!(order.is_complete());
    }

    #[test]
    fn test_queue() {
        let mut q = ManufQueue::new();
        q.enqueue(ManufOrder::new(ManufMethod::Print3D(PrintTechnology::DLP), vec!["Guide".into()]));
        q.enqueue(ManufOrder::new(ManufMethod::CncMilling(MillingAxes::FiveAxis), vec!["Crown".into()]));
        assert_eq!(q.total(), 2);
        assert_eq!(q.pending().len(), 2);
    }

    #[test]
    fn test_delivered_stays() {
        let mut order = ManufOrder::new(ManufMethod::Casting, vec!["Frame".into()]);
        for _ in 0..10 { order.advance(); }
        assert_eq!(order.stage, WorkflowStage::Delivered);
    }
}
