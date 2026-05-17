//! TlantiCAD Performance Monitor
//! Cross-platform CPU, memory, GPU and disk monitoring using sysinfo

pub mod cpu;
pub mod memory;
pub mod gpu;
pub mod metrics;

pub use cpu::*;
pub use memory::*;
pub use gpu::*;
pub use metrics::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_snapshot_collect() {
        let snap = PerfSnapshot::collect();
        assert!(!snap.summary().is_empty());
    }

    #[test]
    fn test_perf_snapshot_healthy() {
        let snap = PerfSnapshot::collect();
        // just ensure it returns a bool without panic
        let _ = snap.is_system_healthy();
    }

    #[test]
    fn test_cpu_load_category_idle() {
        let c = CpuLoadCategory::from_usage(10.0);
        assert!(matches!(c, CpuLoadCategory::Idle));
    }

    #[test]
    fn test_cpu_load_category_critical() {
        let c = CpuLoadCategory::from_usage(95.0);
        assert!(matches!(c, CpuLoadCategory::Critical));
    }

    #[test]
    fn test_metrics_collector() {
        let mut mc = MetricsCollector::new(10);
        mc.tick();
        assert!(mc.latest().is_some());
    }

    #[test]
    fn test_metrics_avg_cpu() {
        let mut mc = MetricsCollector::new(10);
        mc.tick();
        mc.tick();
        let avg = mc.avg_cpu_last_n(2);
        assert!(avg >= 0.0);
    }

    #[test]
    fn test_memory_stats_collect() {
        let ms = collect_memory_stats();
        assert!(ms.total_gb() > 0.0);
        assert!(!ms.summary().is_empty());
    }

    #[test]
    fn test_required_ram() {
        let r = required_ram_gb("cbct");
        assert!(r > 0.0);
    }
}
