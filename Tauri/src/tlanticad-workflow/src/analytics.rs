//! S396-S400: Production Analytics & KPIs
//!
//! Dashboards, trend analysis, and key performance indicators for dental labs.

use serde::{Deserialize, Serialize};

/// KPI definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kpi {
    pub name: String,
    pub value: f64,
    pub target: f64,
    pub unit: String,
    pub trend: Trend,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trend { Up, Down, Flat }

impl Kpi {
    pub fn new(name: impl Into<String>, value: f64, target: f64, unit: impl Into<String>) -> Self {
        let trend = if (value - target).abs() < f64::EPSILON { Trend::Flat }
            else if value >= target { Trend::Up }
            else { Trend::Down };
        Self { name: name.into(), value, target, unit: unit.into(), trend }
    }

    pub fn is_on_target(&self) -> bool {
        self.value >= self.target
    }

    pub fn pct_of_target(&self) -> f64 {
        if self.target == 0.0 { 0.0 } else { self.value / self.target * 100.0 }
    }
}

/// Production analytics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSnapshot {
    pub period: String,
    pub kpis: Vec<Kpi>,
}

impl AnalyticsSnapshot {
    pub fn new(period: impl Into<String>) -> Self {
        Self { period: period.into(), kpis: Vec::new() }
    }

    pub fn add_kpi(&mut self, kpi: Kpi) { self.kpis.push(kpi); }

    pub fn on_target_count(&self) -> usize {
        self.kpis.iter().filter(|k| k.is_on_target()).count()
    }

    pub fn overall_score(&self) -> f64 {
        if self.kpis.is_empty() { return 0.0; }
        let sum: f64 = self.kpis.iter().map(|k| k.pct_of_target().min(100.0)).sum();
        sum / self.kpis.len() as f64
    }
}

/// Build a standard dental lab KPI dashboard
pub fn lab_dashboard(
    cases_completed: usize,
    cases_target: usize,
    avg_turnaround_days: f64,
    turnaround_target: f64,
    qc_pass_rate: f64,
    first_time_fit_rate: f64,
    machine_utilization_pct: f64,
    on_time_delivery_pct: f64,
) -> AnalyticsSnapshot {
    let mut snap = AnalyticsSnapshot::new("current");
    snap.add_kpi(Kpi::new("Cases Completed", cases_completed as f64, cases_target as f64, "cases"));
    snap.add_kpi(Kpi::new("Avg Turnaround", avg_turnaround_days, turnaround_target, "days"));
    snap.add_kpi(Kpi::new("QC Pass Rate", qc_pass_rate, 95.0, "%"));
    snap.add_kpi(Kpi::new("First-Time Fit", first_time_fit_rate, 90.0, "%"));
    snap.add_kpi(Kpi::new("Machine Utilization", machine_utilization_pct, 80.0, "%"));
    snap.add_kpi(Kpi::new("On-Time Delivery", on_time_delivery_pct, 95.0, "%"));
    snap
}

/// Simple moving average
pub fn moving_average(values: &[f64], window: usize) -> Vec<f64> {
    if window == 0 || values.len() < window { return Vec::new(); }
    values.windows(window)
        .map(|w| w.iter().sum::<f64>() / w.len() as f64)
        .collect()
}

/// Detect outlier values (simple z-score approach)
pub fn detect_outliers(values: &[f64], z_threshold: f64) -> Vec<usize> {
    if values.len() < 3 { return Vec::new(); }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();
    if std_dev < f64::EPSILON { return Vec::new(); }
    values.iter().enumerate()
        .filter(|(_, v)| ((*v - mean) / std_dev).abs() > z_threshold)
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kpi_on_target() {
        let kpi = Kpi::new("QC", 96.0, 95.0, "%");
        assert!(kpi.is_on_target());
        assert!(kpi.pct_of_target() > 100.0);
    }

    #[test]
    fn test_kpi_below_target() {
        let kpi = Kpi::new("Delivery", 88.0, 95.0, "%");
        assert!(!kpi.is_on_target());
        assert_eq!(kpi.trend, Trend::Down);
    }

    #[test]
    fn test_analytics_snapshot() {
        let snap = lab_dashboard(50, 45, 3.2, 4.0, 97.0, 92.0, 85.0, 96.0);
        assert_eq!(snap.kpis.len(), 6);
        assert!(snap.overall_score() > 0.0);
    }

    #[test]
    fn test_on_target_count() {
        let snap = lab_dashboard(50, 45, 3.0, 4.0, 97.0, 92.0, 85.0, 96.0);
        assert!(snap.on_target_count() >= 4);
    }

    #[test]
    fn test_moving_average() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ma = moving_average(&data, 3);
        assert_eq!(ma.len(), 3);
        assert!((ma[0] - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_outliers() {
        let data = vec![10.0, 11.0, 10.5, 10.2, 50.0, 10.8];
        let outliers = detect_outliers(&data, 2.0);
        assert!(outliers.contains(&4));
    }
}
