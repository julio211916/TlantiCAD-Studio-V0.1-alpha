//! AI Model Training Utilities
//!
//! Manage training datasets, augmentation, evaluation metrics, and
//! model validation for dental AI models (heuristic + ML).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A labeled training sample (3D dental data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub id: String,
    pub features: Vec<f64>,
    pub label: String,
    pub split: DataSplit,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSplit {
    Train,
    Validation,
    Test,
}

/// Training dataset container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub name: String,
    pub samples: Vec<TrainingSample>,
    pub feature_names: Vec<String>,
    pub label_set: Vec<String>,
}

impl Dataset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            samples: Vec::new(),
            feature_names: Vec::new(),
            label_set: Vec::new(),
        }
    }

    pub fn add_sample(&mut self, sample: TrainingSample) {
        if !self.label_set.contains(&sample.label) {
            self.label_set.push(sample.label.clone());
        }
        self.samples.push(sample);
    }

    pub fn train_set(&self) -> Vec<&TrainingSample> {
        self.samples.iter().filter(|s| s.split == DataSplit::Train).collect()
    }

    pub fn validation_set(&self) -> Vec<&TrainingSample> {
        self.samples.iter().filter(|s| s.split == DataSplit::Validation).collect()
    }

    pub fn test_set(&self) -> Vec<&TrainingSample> {
        self.samples.iter().filter(|s| s.split == DataSplit::Test).collect()
    }

    pub fn class_distribution(&self) -> HashMap<String, usize> {
        let mut dist = HashMap::new();
        for s in &self.samples {
            *dist.entry(s.label.clone()).or_insert(0) += 1;
        }
        dist
    }

    pub fn split_counts(&self) -> HashMap<DataSplit, usize> {
        let mut counts = HashMap::new();
        for s in &self.samples {
            *counts.entry(s.split).or_insert(0) += 1;
        }
        counts
    }
}

/// Confusion matrix for classification evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    pub labels: Vec<String>,
    pub matrix: Vec<Vec<usize>>, // [actual][predicted]
}

impl ConfusionMatrix {
    pub fn new(labels: Vec<String>) -> Self {
        let n = labels.len();
        Self {
            labels,
            matrix: vec![vec![0; n]; n],
        }
    }

    pub fn record(&mut self, actual: &str, predicted: &str) {
        if let Some(a) = self.labels.iter().position(|l| l == actual) {
            if let Some(p) = self.labels.iter().position(|l| l == predicted) {
                self.matrix[a][p] += 1;
            }
        }
    }

    pub fn accuracy(&self) -> f64 {
        let correct: usize = (0..self.labels.len()).map(|i| self.matrix[i][i]).sum();
        let total: usize = self.matrix.iter().flat_map(|row| row.iter()).sum();
        if total == 0 { 0.0 } else { correct as f64 / total as f64 }
    }

    pub fn precision(&self, label: &str) -> f64 {
        if let Some(p) = self.labels.iter().position(|l| l == label) {
            let tp = self.matrix[p][p];
            let total_predicted: usize = (0..self.labels.len()).map(|i| self.matrix[i][p]).sum();
            if total_predicted == 0 { 0.0 } else { tp as f64 / total_predicted as f64 }
        } else {
            0.0
        }
    }

    pub fn recall(&self, label: &str) -> f64 {
        if let Some(a) = self.labels.iter().position(|l| l == label) {
            let tp = self.matrix[a][a];
            let total_actual: usize = self.matrix[a].iter().sum();
            if total_actual == 0 { 0.0 } else { tp as f64 / total_actual as f64 }
        } else {
            0.0
        }
    }

    pub fn f1_score(&self, label: &str) -> f64 {
        let p = self.precision(label);
        let r = self.recall(label);
        if p + r == 0.0 { 0.0 } else { 2.0 * p * r / (p + r) }
    }
}

/// Model evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub accuracy: f64,
    pub mean_precision: f64,
    pub mean_recall: f64,
    pub mean_f1: f64,
    pub per_class: HashMap<String, ClassMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub support: usize,
}

/// Compute evaluation metrics from a confusion matrix
pub fn evaluate(cm: &ConfusionMatrix) -> EvalMetrics {
    let accuracy = cm.accuracy();
    let mut per_class = HashMap::new();
    let mut sum_p = 0.0;
    let mut sum_r = 0.0;
    let mut sum_f1 = 0.0;

    for label in &cm.labels {
        let p = cm.precision(label);
        let r = cm.recall(label);
        let f1 = cm.f1_score(label);
        let support: usize = if let Some(a) = cm.labels.iter().position(|l| l == label) {
            cm.matrix[a].iter().sum()
        } else {
            0
        };

        per_class.insert(label.clone(), ClassMetrics { precision: p, recall: r, f1, support });
        sum_p += p;
        sum_r += r;
        sum_f1 += f1;
    }

    let n = cm.labels.len().max(1) as f64;
    EvalMetrics {
        accuracy,
        mean_precision: sum_p / n,
        mean_recall: sum_r / n,
        mean_f1: sum_f1 / n,
        per_class,
    }
}

/// Data augmentation: add Gaussian noise to features
pub fn augment_noise(sample: &TrainingSample, std_dev: f64) -> TrainingSample {
    // Deterministic pseudo-noise based on feature index (no RNG dependency)
    let features: Vec<f64> = sample.features.iter().enumerate()
        .map(|(i, f)| {
            let noise = ((i as f64 * 0.31415) % 1.0 - 0.5) * 2.0 * std_dev;
            f + noise
        })
        .collect();

    TrainingSample {
        id: format!("{}_aug", sample.id),
        features,
        label: sample.label.clone(),
        split: sample.split,
        metadata: sample.metadata.clone(),
    }
}

/// K-fold cross-validation split indices
pub fn kfold_split(n_samples: usize, k: usize) -> Vec<(Vec<usize>, Vec<usize>)> {
    if k == 0 || n_samples == 0 {
        return Vec::new();
    }
    let fold_size = n_samples / k;
    let mut result = Vec::with_capacity(k);

    for fold in 0..k {
        let test_start = fold * fold_size;
        let test_end = if fold == k - 1 { n_samples } else { test_start + fold_size };
        let test_indices: Vec<usize> = (test_start..test_end).collect();
        let train_indices: Vec<usize> = (0..n_samples).filter(|i| !test_indices.contains(i)).collect();
        result.push((train_indices, test_indices));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample(id: &str, label: &str, split: DataSplit) -> TrainingSample {
        TrainingSample {
            id: id.into(),
            features: vec![1.0, 2.0, 3.0],
            label: label.into(),
            split,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_dataset_add_and_split() {
        let mut ds = Dataset::new("teeth");
        ds.add_sample(make_sample("s1", "molar", DataSplit::Train));
        ds.add_sample(make_sample("s2", "incisor", DataSplit::Train));
        ds.add_sample(make_sample("s3", "molar", DataSplit::Test));
        assert_eq!(ds.train_set().len(), 2);
        assert_eq!(ds.test_set().len(), 1);
        assert_eq!(ds.label_set.len(), 2);
    }

    #[test]
    fn test_class_distribution() {
        let mut ds = Dataset::new("test");
        ds.add_sample(make_sample("a", "molar", DataSplit::Train));
        ds.add_sample(make_sample("b", "molar", DataSplit::Train));
        ds.add_sample(make_sample("c", "incisor", DataSplit::Train));
        let dist = ds.class_distribution();
        assert_eq!(dist["molar"], 2);
        assert_eq!(dist["incisor"], 1);
    }

    #[test]
    fn test_confusion_matrix_accuracy() {
        let mut cm = ConfusionMatrix::new(vec!["A".into(), "B".into()]);
        cm.record("A", "A");
        cm.record("A", "A");
        cm.record("B", "B");
        cm.record("B", "A"); // misclassification
        assert!((cm.accuracy() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_precision_recall_f1() {
        let mut cm = ConfusionMatrix::new(vec!["A".into(), "B".into()]);
        cm.record("A", "A"); // TP for A
        cm.record("A", "B"); // FN for A, FP for B
        cm.record("B", "B"); // TP for B
        cm.record("B", "B"); // TP for B

        // A precision: 1/1 = 1.0, recall: 1/2 = 0.5
        assert!((cm.precision("A") - 1.0).abs() < 1e-10);
        assert!((cm.recall("A") - 0.5).abs() < 1e-10);
        // F1 for A: 2*1.0*0.5/(1.0+0.5) = 2/3
        assert!((cm.f1_score("A") - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_metrics() {
        let mut cm = ConfusionMatrix::new(vec!["A".into(), "B".into()]);
        cm.record("A", "A");
        cm.record("B", "B");
        let metrics = evaluate(&cm);
        assert!((metrics.accuracy - 1.0).abs() < 1e-10);
        assert!((metrics.mean_f1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_augment_noise() {
        let sample = make_sample("s1", "molar", DataSplit::Train);
        let aug = augment_noise(&sample, 0.1);
        assert_eq!(aug.id, "s1_aug");
        assert_eq!(aug.label, "molar");
        assert_eq!(aug.features.len(), sample.features.len());
        // Should be different from original
        assert_ne!(aug.features, sample.features);
    }

    #[test]
    fn test_kfold_split() {
        let folds = kfold_split(10, 5);
        assert_eq!(folds.len(), 5);
        for (train, test) in &folds {
            assert_eq!(train.len() + test.len(), 10);
        }
    }

    #[test]
    fn test_kfold_empty() {
        let folds = kfold_split(0, 5);
        assert!(folds.is_empty());
    }

    #[test]
    fn test_split_counts() {
        let mut ds = Dataset::new("test");
        ds.add_sample(make_sample("a", "x", DataSplit::Train));
        ds.add_sample(make_sample("b", "x", DataSplit::Validation));
        let counts = ds.split_counts();
        assert_eq!(counts[&DataSplit::Train], 1);
        assert_eq!(counts[&DataSplit::Validation], 1);
    }
}
