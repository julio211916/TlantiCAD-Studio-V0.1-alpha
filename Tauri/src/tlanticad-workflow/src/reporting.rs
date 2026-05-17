//! S376-S380: Reporting & Metrics
//!
//! Case reports, production summaries, and workflow metrics.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportType {
    CaseSummary,
    ProductionDaily,
    ProductionWeekly,
    QcSummary,
    TurnaroundTime,
    MaterialUsage,
    MachineUtilization,
    RevenueBreakdown,
}

/// A single report record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub report_type: ReportType,
    pub title: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub metrics: Vec<ReportMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
}

impl Report {
    pub fn new(report_type: ReportType, title: impl Into<String>, days_back: i64) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            report_type,
            title: title.into(),
            generated_at: now,
            period_start: now - Duration::days(days_back),
            period_end: now,
            metrics: Vec::new(),
        }
    }

    pub fn add_metric(&mut self, name: impl Into<String>, value: f64, unit: impl Into<String>) {
        self.metrics.push(ReportMetric { name: name.into(), value, unit: unit.into() });
    }

    pub fn metric_value(&self, name: &str) -> Option<f64> {
        self.metrics.iter().find(|m| m.name == name).map(|m| m.value)
    }
}

/// Production turnaround analyser
pub fn avg_turnaround_days(cases: &[(DateTime<Utc>, DateTime<Utc>)]) -> f64 {
    if cases.is_empty() { return 0.0; }
    let total: f64 = cases.iter()
        .map(|(start, end)| end.signed_duration_since(*start).num_hours() as f64 / 24.0)
        .sum();
    total / cases.len() as f64
}

/// On-time delivery rate
pub fn on_time_rate(delivered: &[(DateTime<Utc>, DateTime<Utc>)]) -> f64 {
    if delivered.is_empty() { return 100.0; }
    let on_time = delivered.iter()
        .filter(|(actual, deadline)| actual <= deadline)
        .count();
    on_time as f64 / delivered.len() as f64 * 100.0
}

/// Daily production summary
pub fn daily_production_report(
    cases_completed: usize,
    units_milled: usize,
    units_printed: usize,
    qc_pass_rate: f64,
) -> Report {
    let mut report = Report::new(ReportType::ProductionDaily, "Daily Production", 1);
    report.add_metric("Cases Completed", cases_completed as f64, "cases");
    report.add_metric("Units Milled", units_milled as f64, "units");
    report.add_metric("Units Printed", units_printed as f64, "units");
    report.add_metric("QC Pass Rate", qc_pass_rate, "%");
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_create() {
        let r = Report::new(ReportType::CaseSummary, "Monthly Cases", 30);
        assert_eq!(r.report_type, ReportType::CaseSummary);
        assert!(r.metrics.is_empty());
    }

    #[test]
    fn test_report_metrics() {
        let mut r = Report::new(ReportType::ProductionDaily, "Daily", 1);
        r.add_metric("Cases", 42.0, "count");
        r.add_metric("Revenue", 12500.0, "USD");
        assert_eq!(r.metric_value("Cases"), Some(42.0));
        assert_eq!(r.metric_value("Revenue"), Some(12500.0));
        assert_eq!(r.metric_value("Missing"), None);
    }

    #[test]
    fn test_avg_turnaround() {
        let now = Utc::now();
        let cases = vec![
            (now - Duration::days(5), now - Duration::days(2)),
            (now - Duration::days(3), now),
        ];
        let avg = avg_turnaround_days(&cases);
        assert!(avg > 0.0 && avg < 5.0);
    }

    #[test]
    fn test_on_time_rate() {
        let now = Utc::now();
        let cases = vec![
            (now - Duration::hours(1), now),
            (now + Duration::hours(1), now),
        ];
        let rate = on_time_rate(&cases);
        assert!((rate - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_daily_production_report() {
        let r = daily_production_report(15, 10, 5, 95.5);
        assert_eq!(r.metrics.len(), 4);
        assert_eq!(r.metric_value("QC Pass Rate"), Some(95.5));
    }
}
