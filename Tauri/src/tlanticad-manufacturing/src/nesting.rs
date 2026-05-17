//! S319-S322: Part Nesting & Build Plate Optimization
//!
//! Arrange multiple dental units on a milling disc or print platform.

use serde::{Deserialize, Serialize};

/// 2D bounding box for nesting
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BBox2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BBox2D {
    pub fn width(&self) -> f64 { self.max_x - self.min_x }
    pub fn height(&self) -> f64 { self.max_y - self.min_y }
    pub fn area(&self) -> f64 { self.width() * self.height() }
    pub fn center(&self) -> [f64; 2] {
        [(self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0]
    }

    pub fn overlaps(&self, other: &BBox2D) -> bool {
        self.min_x < other.max_x && self.max_x > other.min_x &&
        self.min_y < other.max_y && self.max_y > other.min_y
    }

    pub fn translate(&self, dx: f64, dy: f64) -> Self {
        Self {
            min_x: self.min_x + dx, min_y: self.min_y + dy,
            max_x: self.max_x + dx, max_y: self.max_y + dy,
        }
    }
}

/// A part to be nested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestingPart {
    pub id: String,
    pub bbox: BBox2D,
    pub height_mm: f64,
    pub priority: u8,
}

/// Platform shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformShape {
    Disc { diameter_mm: f64 },
    Rectangle { width_mm: f64, depth_mm: f64 },
}

impl PlatformShape {
    pub fn area(&self) -> f64 {
        match self {
            Self::Disc { diameter_mm } =>
                std::f64::consts::PI * (diameter_mm / 2.0).powi(2),
            Self::Rectangle { width_mm, depth_mm } =>
                width_mm * depth_mm,
        }
    }

    pub fn fits(&self, x: f64, y: f64, bbox: &BBox2D) -> bool {
        match self {
            Self::Disc { diameter_mm } => {
                let r = diameter_mm / 2.0;
                let corners = [
                    (x + bbox.min_x, y + bbox.min_y),
                    (x + bbox.max_x, y + bbox.min_y),
                    (x + bbox.min_x, y + bbox.max_y),
                    (x + bbox.max_x, y + bbox.max_y),
                ];
                corners.iter().all(|(cx, cy)| cx * cx + cy * cy <= r * r)
            }
            Self::Rectangle { width_mm, depth_mm } => {
                x + bbox.min_x >= -width_mm / 2.0 && x + bbox.max_x <= width_mm / 2.0 &&
                y + bbox.min_y >= -depth_mm / 2.0 && y + bbox.max_y <= depth_mm / 2.0
            }
        }
    }
}

/// Nesting result for one part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedPart {
    pub part_id: String,
    pub position_x: f64,
    pub position_y: f64,
    pub rotation_deg: f64,
}

/// Nesting result for the entire platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestingResult {
    pub placed: Vec<PlacedPart>,
    pub unplaced: Vec<String>,
    pub utilization_pct: f64,
    pub platform: PlatformShape,
}

/// Simple grid-based nesting algorithm
pub fn nest_parts(
    parts: &[NestingPart],
    platform: &PlatformShape,
    gap_mm: f64,
) -> NestingResult {
    let mut placed = Vec::new();
    let mut unplaced = Vec::new();
    let mut occupied: Vec<BBox2D> = Vec::new();
    let mut total_part_area = 0.0;

    // Sort by area descending (larger parts first)
    let mut sorted_parts = parts.to_vec();
    sorted_parts.sort_by(|a, b| b.bbox.area().partial_cmp(&a.bbox.area()).unwrap());

    for part in &sorted_parts {
        let mut best_pos: Option<(f64, f64)> = None;
        let step = 1.0;
        let range = match platform {
            PlatformShape::Disc { diameter_mm } => *diameter_mm / 2.0,
            PlatformShape::Rectangle { width_mm, .. } => *width_mm / 2.0,
        };

        'search: for ix in 0..(range * 2.0 / step) as i32 {
            for iy in 0..(range * 2.0 / step) as i32 {
                let x = -range + ix as f64 * step;
                let y = -range + iy as f64 * step;

                let candidate = part.bbox.translate(x, y);
                if !platform.fits(x, y, &part.bbox) { continue; }

                let expanded = BBox2D {
                    min_x: candidate.min_x - gap_mm,
                    min_y: candidate.min_y - gap_mm,
                    max_x: candidate.max_x + gap_mm,
                    max_y: candidate.max_y + gap_mm,
                };

                if !occupied.iter().any(|o| o.overlaps(&expanded)) {
                    best_pos = Some((x, y));
                    break 'search;
                }
            }
        }

        if let Some((x, y)) = best_pos {
            occupied.push(part.bbox.translate(x, y));
            total_part_area += part.bbox.area();
            placed.push(PlacedPart {
                part_id: part.id.clone(),
                position_x: x,
                position_y: y,
                rotation_deg: 0.0,
            });
        } else {
            unplaced.push(part.id.clone());
        }
    }

    let utilization = if platform.area() > 0.0 {
        (total_part_area / platform.area() * 100.0).min(100.0)
    } else { 0.0 };

    NestingResult { placed, unplaced, utilization_pct: utilization, platform: platform.clone() }
}

/// Nest parts across multiple platforms (discs/plates) when they don't fit in one
pub fn multi_platform_nesting(
    parts: &[NestingPart],
    platform: &PlatformShape,
    spacing_mm: f64,
) -> Vec<NestingResult> {
    let mut remaining = parts.to_vec();
    let mut batches = Vec::new();

    while !remaining.is_empty() {
        let result = nest_parts(&remaining, platform, spacing_mm);
        let placed_ids: Vec<_> = result.placed.iter().map(|p| p.part_id.clone()).collect();
        if placed_ids.is_empty() {
            break; // No more parts can fit
        }
        remaining.retain(|p| !placed_ids.contains(&p.id));
        batches.push(result);
    }
    batches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_operations() {
        let a = BBox2D { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        let b = BBox2D { min_x: 5.0, min_y: 5.0, max_x: 15.0, max_y: 15.0 };
        assert!(a.overlaps(&b));
        assert_eq!(a.area(), 100.0);

        let c = a.translate(20.0, 0.0);
        assert!(!c.overlaps(&b));
    }

    #[test]
    fn test_disc_platform() {
        let disc = PlatformShape::Disc { diameter_mm: 98.0 };
        assert!(disc.area() > 7000.0);
    }

    #[test]
    fn test_nesting_small_parts() {
        let parts = vec![
            NestingPart {
                id: "crown1".into(),
                bbox: BBox2D { min_x: 0.0, min_y: 0.0, max_x: 12.0, max_y: 12.0 },
                height_mm: 8.0, priority: 1,
            },
            NestingPart {
                id: "crown2".into(),
                bbox: BBox2D { min_x: 0.0, min_y: 0.0, max_x: 12.0, max_y: 12.0 },
                height_mm: 8.0, priority: 1,
            },
        ];
        let platform = PlatformShape::Rectangle { width_mm: 80.0, depth_mm: 80.0 };
        let result = nest_parts(&parts, &platform, 2.0);
        assert_eq!(result.placed.len(), 2);
        assert!(result.unplaced.is_empty());
        assert!(result.utilization_pct > 0.0);
    }

    #[test]
    fn test_platform_fits() {
        let rect = PlatformShape::Rectangle { width_mm: 40.0, depth_mm: 30.0 };
        let small = BBox2D { min_x: 0.0, min_y: 0.0, max_x: 10.0, max_y: 10.0 };
        assert!(rect.fits(-10.0, -10.0, &small));
    }

    #[test]
    fn test_nesting_utilization_bounded() {
        let parts = vec![
            NestingPart { id: "a".into(), bbox: BBox2D { min_x: 0.0, min_y: 0.0, max_x: 5.0, max_y: 5.0 }, height_mm: 5.0, priority: 1 },
        ];
        let platform = PlatformShape::Rectangle { width_mm: 100.0, depth_mm: 100.0 };
        let result = nest_parts(&parts, &platform, 1.0);
        assert!(result.utilization_pct <= 100.0);
        assert!(result.utilization_pct > 0.0);
    }

    #[test]
    fn test_multi_disc_nesting() {
        let parts: Vec<NestingPart> = (0..20).map(|i| NestingPart {
            id: format!("unit_{}", i),
            bbox: BBox2D { min_x: 0.0, min_y: 0.0, max_x: 12.0, max_y: 12.0 },
            height_mm: 8.0,
            priority: 1,
        }).collect();
        let disc = PlatformShape::Disc { diameter_mm: 98.0 };
        let batches = multi_platform_nesting(&parts, &disc, 2.0);
        assert!(!batches.is_empty());
        let total_placed: usize = batches.iter().map(|b| b.placed.len()).sum();
        assert!(total_placed <= 20);
    }

    #[test]
    fn test_bbox_center() {
        let b = BBox2D { min_x: 0.0, min_y: 10.0, max_x: 20.0, max_y: 30.0 };
        let c = b.center();
        assert!((c[0] - 10.0).abs() < 1e-10);
        assert!((c[1] - 20.0).abs() < 1e-10);
    }
}
