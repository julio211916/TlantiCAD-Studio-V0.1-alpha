//! AI Model Registry
//!
//! Version, catalog, and manage AI/heuristic models used throughout
//! the dental CAD pipeline.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    Segmentation,
    Classification,
    FeatureExtraction,
    AnomalyDetection,
    MarginDetection,
    ToothPlacement,
    MeshRepair,
    Custom,
}

/// Model status in the registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelStatus {
    Draft,
    Validated,
    Production,
    Deprecated,
    Archived,
}

/// A registered model with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredModel {
    pub id: String,
    pub name: String,
    pub version: String,
    pub model_type: ModelType,
    pub status: ModelStatus,
    pub description: String,
    pub accuracy: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub parameters: HashMap<String, String>,
}

impl RegisteredModel {
    pub fn is_production(&self) -> bool {
        self.status == ModelStatus::Production
    }

    pub fn is_usable(&self) -> bool {
        matches!(self.status, ModelStatus::Validated | ModelStatus::Production)
    }
}

/// Model registry that catalogs all available models
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelRegistry {
    models: Vec<RegisteredModel>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    /// Register a new model
    pub fn register(&mut self, model: RegisteredModel) -> Result<(), String> {
        if self.models.iter().any(|m| m.id == model.id) {
            return Err(format!("Model with id '{}' already exists", model.id));
        }
        self.models.push(model);
        Ok(())
    }

    /// Get a model by ID
    pub fn get(&self, id: &str) -> Option<&RegisteredModel> {
        self.models.iter().find(|m| m.id == id)
    }

    /// Get all models of a specific type
    pub fn by_type(&self, model_type: ModelType) -> Vec<&RegisteredModel> {
        self.models.iter().filter(|m| m.model_type == model_type).collect()
    }

    /// Get all production models
    pub fn production_models(&self) -> Vec<&RegisteredModel> {
        self.models.iter().filter(|m| m.is_production()).collect()
    }

    /// Get the best model for a type (highest accuracy among production models)
    pub fn best_for(&self, model_type: ModelType) -> Option<&RegisteredModel> {
        self.models.iter()
            .filter(|m| m.model_type == model_type && m.is_usable())
            .max_by(|a, b| {
                a.accuracy.unwrap_or(0.0)
                    .partial_cmp(&b.accuracy.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Update model status
    pub fn set_status(&mut self, id: &str, status: ModelStatus) -> bool {
        if let Some(model) = self.models.iter_mut().find(|m| m.id == id) {
            model.status = status;
            model.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// List all models
    pub fn list(&self) -> &[RegisteredModel] {
        &self.models
    }

    /// Count models by status
    pub fn count_by_status(&self) -> HashMap<ModelStatus, usize> {
        let mut counts = HashMap::new();
        for m in &self.models {
            *counts.entry(m.status).or_insert(0) += 1;
        }
        counts
    }

    /// Remove deprecated models
    pub fn prune_deprecated(&mut self) -> usize {
        let before = self.models.len();
        self.models.retain(|m| m.status != ModelStatus::Deprecated);
        before - self.models.len()
    }

    /// Search models by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&RegisteredModel> {
        self.models.iter()
            .filter(|m| m.tags.iter().any(|t| t == tag))
            .collect()
    }
}

/// Create a standard dental model entry
pub fn create_dental_model(
    id: &str,
    name: &str,
    version: &str,
    model_type: ModelType,
) -> RegisteredModel {
    let now = Utc::now();
    RegisteredModel {
        id: id.into(),
        name: name.into(),
        version: version.into(),
        model_type,
        status: ModelStatus::Draft,
        description: String::new(),
        accuracy: None,
        created_at: now,
        updated_at: now,
        tags: Vec::new(),
        parameters: HashMap::new(),
    }
}

/// Register standard dental AI models
pub fn register_standard_models(registry: &mut ModelRegistry) {
    let models = vec![
        ("seg-v1", "Tooth Segmentation", "1.0.0", ModelType::Segmentation),
        ("cls-v1", "Tooth Classification", "1.0.0", ModelType::Classification),
        ("feat-v1", "Feature Extraction", "1.0.0", ModelType::FeatureExtraction),
        ("anom-v1", "Scan Anomaly Detector", "1.0.0", ModelType::AnomalyDetection),
        ("margin-v1", "Margin Line Detector", "1.0.0", ModelType::MarginDetection),
        ("place-v1", "Tooth Placement", "1.0.0", ModelType::ToothPlacement),
        ("repair-v1", "Mesh Repair", "1.0.0", ModelType::MeshRepair),
    ];

    for (id, name, ver, mtype) in models {
        let _ = registry.register(create_dental_model(id, name, ver, mtype));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model(id: &str, mtype: ModelType, status: ModelStatus, acc: Option<f64>) -> RegisteredModel {
        let mut m = create_dental_model(id, id, "1.0", mtype);
        m.status = status;
        m.accuracy = acc;
        m
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = ModelRegistry::new();
        let m = make_model("seg-1", ModelType::Segmentation, ModelStatus::Draft, None);
        reg.register(m).unwrap();
        assert!(reg.get("seg-1").is_some());
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn test_duplicate_register_error() {
        let mut reg = ModelRegistry::new();
        let m = make_model("seg-1", ModelType::Segmentation, ModelStatus::Draft, None);
        reg.register(m.clone()).unwrap();
        assert!(reg.register(m).is_err());
    }

    #[test]
    fn test_by_type() {
        let mut reg = ModelRegistry::new();
        reg.register(make_model("s1", ModelType::Segmentation, ModelStatus::Production, Some(0.95))).unwrap();
        reg.register(make_model("c1", ModelType::Classification, ModelStatus::Production, Some(0.90))).unwrap();
        assert_eq!(reg.by_type(ModelType::Segmentation).len(), 1);
        assert_eq!(reg.by_type(ModelType::Classification).len(), 1);
    }

    #[test]
    fn test_best_for() {
        let mut reg = ModelRegistry::new();
        reg.register(make_model("s1", ModelType::Segmentation, ModelStatus::Production, Some(0.92))).unwrap();
        reg.register(make_model("s2", ModelType::Segmentation, ModelStatus::Validated, Some(0.96))).unwrap();
        reg.register(make_model("s3", ModelType::Segmentation, ModelStatus::Deprecated, Some(0.99))).unwrap();
        let best = reg.best_for(ModelType::Segmentation).unwrap();
        assert_eq!(best.id, "s2"); // highest accuracy among usable
    }

    #[test]
    fn test_set_status() {
        let mut reg = ModelRegistry::new();
        reg.register(make_model("m1", ModelType::MeshRepair, ModelStatus::Draft, None)).unwrap();
        assert!(reg.set_status("m1", ModelStatus::Production));
        assert!(reg.get("m1").unwrap().is_production());
        assert!(!reg.set_status("nonexistent", ModelStatus::Archived));
    }

    #[test]
    fn test_prune_deprecated() {
        let mut reg = ModelRegistry::new();
        reg.register(make_model("a", ModelType::Segmentation, ModelStatus::Production, None)).unwrap();
        reg.register(make_model("b", ModelType::Segmentation, ModelStatus::Deprecated, None)).unwrap();
        let pruned = reg.prune_deprecated();
        assert_eq!(pruned, 1);
        assert_eq!(reg.list().len(), 1);
    }

    #[test]
    fn test_search_by_tag() {
        let mut reg = ModelRegistry::new();
        let mut m = make_model("t1", ModelType::Classification, ModelStatus::Draft, None);
        m.tags = vec!["dental".into(), "molar".into()];
        reg.register(m).unwrap();
        assert_eq!(reg.search_by_tag("dental").len(), 1);
        assert_eq!(reg.search_by_tag("unknown").len(), 0);
    }

    #[test]
    fn test_register_standard_models() {
        let mut reg = ModelRegistry::new();
        register_standard_models(&mut reg);
        assert_eq!(reg.list().len(), 7);
    }

    #[test]
    fn test_count_by_status() {
        let mut reg = ModelRegistry::new();
        reg.register(make_model("a", ModelType::Segmentation, ModelStatus::Production, None)).unwrap();
        reg.register(make_model("b", ModelType::Segmentation, ModelStatus::Draft, None)).unwrap();
        let counts = reg.count_by_status();
        assert_eq!(counts[&ModelStatus::Production], 1);
        assert_eq!(counts[&ModelStatus::Draft], 1);
    }

    #[test]
    fn test_model_is_usable() {
        let m1 = make_model("x", ModelType::Segmentation, ModelStatus::Production, None);
        let m2 = make_model("y", ModelType::Segmentation, ModelStatus::Draft, None);
        let m3 = make_model("z", ModelType::Segmentation, ModelStatus::Validated, None);
        assert!(m1.is_usable());
        assert!(!m2.is_usable());
        assert!(m3.is_usable());
    }
}
