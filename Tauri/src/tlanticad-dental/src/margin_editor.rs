//! S127-S130: Margin editor — interactive margin refinement and classification.
//!
//! Extends the margin module with advanced editing, multi-type detection, and
//! finish-line validation.

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

use crate::margin::MarginLine;

/// A control point on the margin that the user can drag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginControlPoint {
    pub index: usize,
    pub position: [f64; 3],
    pub is_locked: bool,
}

/// Margin editing session.
#[derive(Debug, Clone)]
pub struct MarginEditSession {
    pub margin: MarginLine,
    pub control_points: Vec<MarginControlPoint>,
    history: Vec<MarginLine>,
}

impl MarginEditSession {
    /// Start an editing session from a detected margin line.
    pub fn new(margin: MarginLine) -> Self {
        let control_points = margin
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| MarginControlPoint {
                index: i,
                position: [p.x, p.y, p.z],
                is_locked: false,
            })
            .collect();
        Self {
            margin,
            control_points,
            history: Vec::new(),
        }
    }

    /// Move a control point and update the margin.
    pub fn move_point(&mut self, index: usize, new_pos: Point3<f64>) {
        if index >= self.margin.points.len() {
            return;
        }
        if let Some(cp) = self.control_points.get(index) {
            if cp.is_locked {
                return;
            }
        }
        // Save history
        self.history.push(self.margin.clone());
        self.margin.points[index] = new_pos;
        self.control_points[index].position = [new_pos.x, new_pos.y, new_pos.z];
    }

    /// Lock a control point so it cannot be moved.
    pub fn lock_point(&mut self, index: usize) {
        if let Some(cp) = self.control_points.get_mut(index) {
            cp.is_locked = true;
        }
    }

    /// Unlock a control point.
    pub fn unlock_point(&mut self, index: usize) {
        if let Some(cp) = self.control_points.get_mut(index) {
            cp.is_locked = false;
        }
    }

    /// Undo last edit.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.margin = prev;
            // Rebuild control points
            self.control_points = self.margin
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| MarginControlPoint {
                    index: i,
                    position: [p.x, p.y, p.z],
                    is_locked: false,
                })
                .collect();
            true
        } else {
            false
        }
    }

    /// Get number of undo steps available.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Finalize the edited margin.
    pub fn finish(self) -> MarginLine {
        self.margin
    }
}

/// Margin finish-line quality check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginQuality {
    pub is_closed: bool,
    pub smoothness_score: f64,    // 0.0–1.0
    pub min_curvature_radius: f64, // mm
    pub sharp_corners: usize,
}

/// Evaluate margin quality.
pub fn evaluate_margin_quality(margin: &MarginLine) -> MarginQuality {
    let n = margin.points.len();
    if n < 3 {
        return MarginQuality {
            is_closed: margin.is_closed,
            smoothness_score: 0.0,
            min_curvature_radius: 0.0,
            sharp_corners: 0,
        };
    }

    let mut min_radius = f64::MAX;
    let mut sharp_corners = 0usize;
    let mut angle_sum = 0.0;
    let mut total_angles = 0usize;

    for i in 0..n {
        let prev = if i == 0 { n - 1 } else { i - 1 };
        let next = if i == n - 1 { 0 } else { i + 1 };

        let v1 = (margin.points[prev] - margin.points[i]).normalize();
        let v2 = (margin.points[next] - margin.points[i]).normalize();
        let dot = v1.dot(&v2).clamp(-1.0, 1.0);
        let angle = dot.acos();

        angle_sum += angle;
        total_angles += 1;

        // Curvature radius approximation
        let chord = (margin.points[next] - margin.points[prev]).norm();
        if angle.sin().abs() > 1e-6 {
            let radius = chord / (2.0 * angle.sin());
            min_radius = min_radius.min(radius);
        }

        if angle < 0.5 {
            sharp_corners += 1;
        }
    }

    let avg_angle = if total_angles > 0 {
        angle_sum / total_angles as f64
    } else {
        0.0
    };
    let smoothness = (avg_angle / std::f64::consts::PI).clamp(0.0, 1.0);

    if min_radius == f64::MAX {
        min_radius = 0.0;
    }

    MarginQuality {
        is_closed: margin.is_closed,
        smoothness_score: smoothness,
        min_curvature_radius: min_radius,
        sharp_corners,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    fn make_margin() -> MarginLine {
        MarginLine {
            points: vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(1.0, 1.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
            ],
            normals: vec![Vector3::z(); 4],
            margin_type: MarginType::Chamfer,
            confidence: 0.9,
            is_closed: true,
        }
    }

    #[test]
    fn edit_session_move() {
        let mut session = MarginEditSession::new(make_margin());
        session.move_point(1, Point3::new(2.0, 0.0, 0.0));
        assert_eq!(session.margin.points[1], Point3::new(2.0, 0.0, 0.0));
        assert_eq!(session.history_len(), 1);
    }

    #[test]
    fn edit_session_undo() {
        let mut session = MarginEditSession::new(make_margin());
        let original = session.margin.points[1];
        session.move_point(1, Point3::new(2.0, 0.0, 0.0));
        assert!(session.undo());
        assert_eq!(session.margin.points[1], original);
    }

    #[test]
    fn lock_prevents_move() {
        let mut session = MarginEditSession::new(make_margin());
        session.lock_point(1);
        let before = session.margin.points[1];
        session.move_point(1, Point3::new(99.0, 99.0, 99.0));
        assert_eq!(session.margin.points[1], before);
    }

    #[test]
    fn quality_check() {
        let margin = make_margin();
        let quality = evaluate_margin_quality(&margin);
        assert!(quality.is_closed);
        assert!(quality.smoothness_score >= 0.0 && quality.smoothness_score <= 1.0);
    }
}
