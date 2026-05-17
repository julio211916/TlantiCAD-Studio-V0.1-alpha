//! S301-S303: CAM Toolpath Generation
//!
//! Core toolpath computation for dental milling and 3D printing.

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// Toolpath computation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolpathStrategy {
    Roughing,
    SemiFinishing,
    Finishing,
    RestMilling,
    Contouring,
    Drilling,
    Engraving,
}

/// Motion type along a toolpath
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotionType {
    Rapid,
    Linear,
    ArcCW,
    ArcCCW,
    Retract,
    Plunge,
}

/// Single toolpath segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathSegment {
    pub start: Point3<f64>,
    pub end: Point3<f64>,
    pub motion: MotionType,
    pub feed_rate_mm_min: f64,
    pub spindle_rpm: f64,
}

impl ToolpathSegment {
    pub fn length(&self) -> f64 {
        (self.end - self.start).norm()
    }

    pub fn duration_seconds(&self) -> f64 {
        if self.feed_rate_mm_min > 0.0 {
            self.length() / self.feed_rate_mm_min * 60.0
        } else {
            0.0
        }
    }
}

/// Complete toolpath for one operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toolpath {
    pub id: String,
    pub strategy: ToolpathStrategy,
    pub tool_diameter_mm: f64,
    pub tool_length_mm: f64,
    pub stepover_mm: f64,
    pub stepdown_mm: f64,
    pub segments: Vec<ToolpathSegment>,
    pub safety_height_mm: f64,
}

impl Toolpath {
    pub fn new(strategy: ToolpathStrategy, tool_diameter: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            strategy,
            tool_diameter_mm: tool_diameter,
            tool_length_mm: tool_diameter * 5.0,
            stepover_mm: tool_diameter * 0.4,
            stepdown_mm: tool_diameter * 0.3,
            segments: Vec::new(),
            safety_height_mm: 5.0,
        }
    }

    pub fn total_length(&self) -> f64 {
        self.segments.iter().map(|s| s.length()).sum()
    }

    pub fn estimated_time_minutes(&self) -> f64 {
        self.segments.iter().map(|s| s.duration_seconds()).sum::<f64>() / 60.0
    }

    pub fn cutting_segments(&self) -> Vec<&ToolpathSegment> {
        self.segments.iter()
            .filter(|s| matches!(s.motion, MotionType::Linear | MotionType::ArcCW | MotionType::ArcCCW))
            .collect()
    }
}

/// Generate zigzag roughing toolpath over bounding box
pub fn generate_roughing_toolpath(
    bbox_min: Point3<f64>,
    bbox_max: Point3<f64>,
    tool_diameter: f64,
    stepover: f64,
    stepdown: f64,
    feed_rate: f64,
    spindle_rpm: f64,
) -> Toolpath {
    let mut tp = Toolpath::new(ToolpathStrategy::Roughing, tool_diameter);
    tp.stepover_mm = stepover;
    tp.stepdown_mm = stepdown;

    let mut z = bbox_max.z;
    let mut forward = true;

    while z > bbox_min.z {
        z = (z - stepdown).max(bbox_min.z);
        let mut y = bbox_min.y + tool_diameter / 2.0;

        while y < bbox_max.y - tool_diameter / 2.0 {
            let (x_start, x_end) = if forward {
                (bbox_min.x + tool_diameter / 2.0, bbox_max.x - tool_diameter / 2.0)
            } else {
                (bbox_max.x - tool_diameter / 2.0, bbox_min.x + tool_diameter / 2.0)
            };

            tp.segments.push(ToolpathSegment {
                start: Point3::new(x_start, y, z),
                end: Point3::new(x_end, y, z),
                motion: MotionType::Linear,
                feed_rate_mm_min: feed_rate,
                spindle_rpm,
            });

            forward = !forward;
            y += stepover;
        }
    }

    tp
}

/// Generate spiral finishing toolpath
pub fn generate_spiral_finishing(
    center: Point3<f64>,
    radius: f64,
    depth: f64,
    tool_diameter: f64,
    feed_rate: f64,
    spindle_rpm: f64,
) -> Toolpath {
    let mut tp = Toolpath::new(ToolpathStrategy::Finishing, tool_diameter);
    let steps = 72; // 5-degree increments
    let layers = (depth / 0.1).ceil() as usize;

    for layer in 0..layers {
        let z = center.z - layer as f64 * 0.1;
        let r = radius - tool_diameter / 2.0;

        for i in 0..steps {
            let a0 = (i as f64) * std::f64::consts::TAU / steps as f64;
            let a1 = ((i + 1) as f64) * std::f64::consts::TAU / steps as f64;

            tp.segments.push(ToolpathSegment {
                start: Point3::new(center.x + r * a0.cos(), center.y + r * a0.sin(), z),
                end: Point3::new(center.x + r * a1.cos(), center.y + r * a1.sin(), z),
                motion: MotionType::ArcCW,
                feed_rate_mm_min: feed_rate,
                spindle_rpm,
            });
        }
    }

    tp
}

/// Toolpath verification: check for gouging
pub fn verify_no_gouging(toolpath: &Toolpath, min_z: f64) -> Vec<usize> {
    toolpath.segments.iter().enumerate()
        .filter(|(_, s)| s.end.z < min_z || s.start.z < min_z)
        .map(|(i, _)| i)
        .collect()
}

/// Merge multiple toolpaths into a single operation
pub fn merge_toolpaths(paths: &[Toolpath]) -> Toolpath {
    let mut merged = Toolpath::new(
        paths.first().map(|p| p.strategy).unwrap_or(ToolpathStrategy::Finishing),
        paths.first().map(|p| p.tool_diameter_mm).unwrap_or(1.0),
    );
    for tp in paths {
        merged.segments.extend(tp.segments.iter().cloned());
    }
    merged
}

/// Reverse toolpath direction (for climb vs conventional milling)
pub fn reverse_toolpath(tp: &Toolpath) -> Toolpath {
    let mut rev = Toolpath::new(tp.strategy, tp.tool_diameter_mm);
    rev.stepover_mm = tp.stepover_mm;
    rev.stepdown_mm = tp.stepdown_mm;
    rev.safety_height_mm = tp.safety_height_mm;
    rev.segments = tp.segments.iter().rev().map(|s| ToolpathSegment {
        start: s.end,
        end: s.start,
        motion: match s.motion {
            MotionType::ArcCW => MotionType::ArcCCW,
            MotionType::ArcCCW => MotionType::ArcCW,
            other => other,
        },
        feed_rate_mm_min: s.feed_rate_mm_min,
        spindle_rpm: s.spindle_rpm,
    }).collect();
    rev
}

/// Compute bounding box of a toolpath
pub fn toolpath_bounding_box(tp: &Toolpath) -> (Point3<f64>, Point3<f64>) {
    let mut bmin = Point3::new(f64::MAX, f64::MAX, f64::MAX);
    let mut bmax = Point3::new(f64::MIN, f64::MIN, f64::MIN);
    for s in &tp.segments {
        for p in [&s.start, &s.end] {
            bmin.x = bmin.x.min(p.x);
            bmin.y = bmin.y.min(p.y);
            bmin.z = bmin.z.min(p.z);
            bmax.x = bmax.x.max(p.x);
            bmax.y = bmax.y.max(p.y);
            bmax.z = bmax.z.max(p.z);
        }
    }
    (bmin, bmax)
}

/// Insert retract/rapid moves between non-contiguous segments
pub fn insert_retracts(tp: &Toolpath, retract_height: f64, gap_threshold: f64) -> Toolpath {
    let mut result = Toolpath::new(tp.strategy, tp.tool_diameter_mm);
    result.stepover_mm = tp.stepover_mm;
    result.stepdown_mm = tp.stepdown_mm;
    result.safety_height_mm = tp.safety_height_mm;

    for (i, seg) in tp.segments.iter().enumerate() {
        if i > 0 {
            let prev_end = tp.segments[i - 1].end;
            let gap = (seg.start - prev_end).norm();
            if gap > gap_threshold {
                // Retract
                result.segments.push(ToolpathSegment {
                    start: prev_end,
                    end: Point3::new(prev_end.x, prev_end.y, retract_height),
                    motion: MotionType::Retract,
                    feed_rate_mm_min: 3000.0,
                    spindle_rpm: seg.spindle_rpm,
                });
                // Rapid to new position
                result.segments.push(ToolpathSegment {
                    start: Point3::new(prev_end.x, prev_end.y, retract_height),
                    end: Point3::new(seg.start.x, seg.start.y, retract_height),
                    motion: MotionType::Rapid,
                    feed_rate_mm_min: 5000.0,
                    spindle_rpm: seg.spindle_rpm,
                });
                // Plunge
                result.segments.push(ToolpathSegment {
                    start: Point3::new(seg.start.x, seg.start.y, retract_height),
                    end: seg.start,
                    motion: MotionType::Plunge,
                    feed_rate_mm_min: 200.0,
                    spindle_rpm: seg.spindle_rpm,
                });
            }
        }
        result.segments.push(seg.clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roughing_toolpath() {
        let tp = generate_roughing_toolpath(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(20.0, 20.0, 10.0),
            3.0, 1.2, 1.0, 1000.0, 15000.0,
        );
        assert_eq!(tp.strategy, ToolpathStrategy::Roughing);
        assert!(!tp.segments.is_empty());
        assert!(tp.total_length() > 0.0);
        assert!(tp.estimated_time_minutes() > 0.0);
    }

    #[test]
    fn test_spiral_finishing() {
        let tp = generate_spiral_finishing(
            Point3::new(10.0, 10.0, 5.0), 8.0, 3.0, 1.0, 800.0, 20000.0,
        );
        assert_eq!(tp.strategy, ToolpathStrategy::Finishing);
        assert!(!tp.segments.is_empty());
    }

    #[test]
    fn test_segment_properties() {
        let seg = ToolpathSegment {
            start: Point3::new(0.0, 0.0, 0.0),
            end: Point3::new(10.0, 0.0, 0.0),
            motion: MotionType::Linear,
            feed_rate_mm_min: 600.0,
            spindle_rpm: 15000.0,
        };
        assert!((seg.length() - 10.0).abs() < 1e-10);
        assert!((seg.duration_seconds() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_no_gouging() {
        let mut tp = Toolpath::new(ToolpathStrategy::Finishing, 1.0);
        tp.segments.push(ToolpathSegment {
            start: Point3::new(0.0, 0.0, 1.0),
            end: Point3::new(10.0, 0.0, -0.5),
            motion: MotionType::Linear, feed_rate_mm_min: 500.0, spindle_rpm: 15000.0,
        });
        let gouges = verify_no_gouging(&tp, 0.0);
        assert_eq!(gouges.len(), 1);
    }

    #[test]
    fn test_cutting_segments_filter() {
        let mut tp = Toolpath::new(ToolpathStrategy::Roughing, 2.0);
        tp.segments.push(ToolpathSegment {
            start: Point3::origin(), end: Point3::new(0.0, 0.0, 10.0),
            motion: MotionType::Rapid, feed_rate_mm_min: 5000.0, spindle_rpm: 0.0,
        });
        tp.segments.push(ToolpathSegment {
            start: Point3::new(0.0, 0.0, 10.0), end: Point3::new(10.0, 0.0, 10.0),
            motion: MotionType::Linear, feed_rate_mm_min: 500.0, spindle_rpm: 15000.0,
        });
        assert_eq!(tp.cutting_segments().len(), 1);
    }

    #[test]
    fn test_toolpath_merge() {
        let tp1 = generate_roughing_toolpath(
            Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 5.0),
            2.0, 1.0, 0.5, 800.0, 15000.0,
        );
        let tp2 = generate_spiral_finishing(
            Point3::new(5.0, 5.0, 5.0), 4.0, 2.0, 1.0, 600.0, 18000.0,
        );
        let merged = merge_toolpaths(&[tp1.clone(), tp2.clone()]);
        assert_eq!(merged.segments.len(), tp1.segments.len() + tp2.segments.len());
    }

    #[test]
    fn test_toolpath_reverse() {
        let tp = generate_roughing_toolpath(
            Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 5.0),
            2.0, 1.0, 0.5, 800.0, 15000.0,
        );
        let rev = reverse_toolpath(&tp);
        assert_eq!(rev.segments.len(), tp.segments.len());
        if !tp.segments.is_empty() {
            assert!((rev.segments[0].start - tp.segments.last().unwrap().end).norm() < 1e-10);
        }
    }

    #[test]
    fn test_toolpath_bbox() {
        let tp = generate_roughing_toolpath(
            Point3::new(-5.0, -5.0, 0.0), Point3::new(5.0, 5.0, 3.0),
            1.5, 0.7, 0.5, 1000.0, 15000.0,
        );
        let (bmin, bmax) = toolpath_bounding_box(&tp);
        assert!(bmin.x <= bmax.x);
        assert!(bmin.y <= bmax.y);
        assert!(bmin.z <= bmax.z);
    }

    #[test]
    fn test_retract_insert() {
        let mut tp = Toolpath::new(ToolpathStrategy::Finishing, 1.0);
        tp.segments.push(ToolpathSegment {
            start: Point3::new(0.0, 0.0, 1.0), end: Point3::new(5.0, 0.0, 1.0),
            motion: MotionType::Linear, feed_rate_mm_min: 500.0, spindle_rpm: 15000.0,
        });
        tp.segments.push(ToolpathSegment {
            start: Point3::new(20.0, 0.0, 1.0), end: Point3::new(25.0, 0.0, 1.0),
            motion: MotionType::Linear, feed_rate_mm_min: 500.0, spindle_rpm: 15000.0,
        });
        let with_retracts = insert_retracts(&tp, 5.0, 10.0);
        assert!(with_retracts.segments.len() > tp.segments.len());
    }
}
