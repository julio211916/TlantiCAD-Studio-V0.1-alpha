//! S216-S220: Selection system — multi-select, gizmo, measurement, clipping planes

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ────────────────────────────────────────────────────────────────────
//  Selection
// ────────────────────────────────────────────────────────────────────

/// What kind of element is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionKind {
    Node,
    Face,
    Edge,
    Vertex,
}

/// An item in the current selection set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionItem {
    pub node_id: Uuid,
    pub kind: SelectionKind,
    /// Sub-element index (face/edge/vertex index, 0 for Node)
    pub sub_index: u32,
}

/// Multi-selection manager
#[derive(Debug, Clone, Default)]
pub struct SelectionSet {
    pub items: Vec<SelectionItem>,
}

impl SelectionSet {
    pub fn clear(&mut self) { self.items.clear(); }

    pub fn select_single(&mut self, item: SelectionItem) {
        self.items.clear();
        self.items.push(item);
    }

    pub fn toggle(&mut self, item: SelectionItem) {
        if let Some(pos) = self.items.iter().position(|i|
            i.node_id == item.node_id && i.sub_index == item.sub_index && i.kind == item.kind
        ) {
            self.items.remove(pos);
        } else {
            self.items.push(item);
        }
    }

    pub fn add(&mut self, item: SelectionItem) {
        if !self.items.iter().any(|i|
            i.node_id == item.node_id && i.sub_index == item.sub_index && i.kind == item.kind
        ) {
            self.items.push(item);
        }
    }

    pub fn is_selected(&self, node_id: &Uuid) -> bool {
        self.items.iter().any(|i| &i.node_id == node_id)
    }

    pub fn count(&self) -> usize { self.items.len() }
}

// ────────────────────────────────────────────────────────────────────
//  Gizmo
// ────────────────────────────────────────────────────────────────────

/// Gizmo operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

/// Gizmo coordinate space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GizmoSpace {
    Local,
    World,
}

/// Which axis/plane the gizmo is operating on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GizmoAxis {
    X, Y, Z,
    XY, XZ, YZ,
    All,
    None,
}

/// Transform gizmo state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gizmo {
    pub mode: GizmoMode,
    pub space: GizmoSpace,
    pub active_axis: GizmoAxis,
    pub visible: bool,
    pub size: f32,
    pub snap_translate: f32,
    pub snap_rotate_deg: f32,
    pub snap_scale: f32,
}

impl Default for Gizmo {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Translate,
            space: GizmoSpace::World,
            active_axis: GizmoAxis::None,
            visible: true,
            size: 1.0,
            snap_translate: 0.0,
            snap_rotate_deg: 0.0,
            snap_scale: 0.0,
        }
    }
}

impl Gizmo {
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            GizmoMode::Translate => GizmoMode::Rotate,
            GizmoMode::Rotate => GizmoMode::Scale,
            GizmoMode::Scale => GizmoMode::Translate,
        };
    }

    pub fn toggle_space(&mut self) {
        self.space = match self.space {
            GizmoSpace::Local => GizmoSpace::World,
            GizmoSpace::World => GizmoSpace::Local,
        };
    }
}

// ────────────────────────────────────────────────────────────────────
//  Measurement
// ────────────────────────────────────────────────────────────────────

/// Measurement types supported by the viewport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MeasurementType {
    PointToPoint,
    Angle,
    Radius,
    MinimumDistance,
    SurfaceArea,
    Volume,
}

/// A measurement in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub id: Uuid,
    pub measurement_type: MeasurementType,
    pub points: Vec<Point3<f64>>,
    pub value: f64,
    pub unit: String,
    pub label: String,
    pub visible: bool,
}

impl Measurement {
    /// Create a point-to-point distance measurement
    pub fn distance(p1: Point3<f64>, p2: Point3<f64>) -> Self {
        let dist = nalgebra::distance(&p1, &p2);
        Self {
            id: Uuid::new_v4(),
            measurement_type: MeasurementType::PointToPoint,
            points: vec![p1, p2],
            value: dist,
            unit: "mm".into(),
            label: format!("{:.2} mm", dist),
            visible: true,
        }
    }

    /// Create an angle measurement between three points
    pub fn angle(a: Point3<f64>, vertex: Point3<f64>, b: Point3<f64>) -> Self {
        let v1 = (a - vertex).normalize();
        let v2 = (b - vertex).normalize();
        let angle_rad = v1.dot(&v2).clamp(-1.0, 1.0).acos();
        let angle_deg = angle_rad.to_degrees();
        Self {
            id: Uuid::new_v4(),
            measurement_type: MeasurementType::Angle,
            points: vec![a, vertex, b],
            value: angle_deg,
            unit: "°".into(),
            label: format!("{:.1}°", angle_deg),
            visible: true,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Clipping planes
// ────────────────────────────────────────────────────────────────────

/// A clipping plane that hides geometry on one side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClippingPlane {
    pub id: Uuid,
    pub normal: [f64; 3],
    pub distance: f64,
    pub enabled: bool,
    pub show_cap: bool,
    pub cap_color: [f32; 4],
}

impl ClippingPlane {
    /// Create XY clipping plane at height z
    pub fn xy(z: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            normal: [0.0, 0.0, 1.0],
            distance: z,
            enabled: true,
            show_cap: true,
            cap_color: [0.8, 0.2, 0.2, 0.5],
        }
    }

    /// Plane equation coefficients [a, b, c, d] where ax+by+cz+d=0
    pub fn equation(&self) -> [f64; 4] {
        [self.normal[0], self.normal[1], self.normal[2], -self.distance]
    }

    /// Test if a point is on the visible side of the plane
    pub fn is_visible(&self, point: &Point3<f64>) -> bool {
        let n = Vector3::new(self.normal[0], self.normal[1], self.normal[2]);
        n.dot(&point.coords) - self.distance >= 0.0
    }
}

// ────────────────────────────────────────────────────────────────────
//  Tests
// ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_set() {
        let mut sel = SelectionSet::default();
        let id = Uuid::new_v4();
        let item = SelectionItem { node_id: id, kind: SelectionKind::Node, sub_index: 0 };
        sel.select_single(item.clone());
        assert_eq!(sel.count(), 1);
        assert!(sel.is_selected(&id));

        sel.toggle(SelectionItem { node_id: id, kind: SelectionKind::Node, sub_index: 0 });
        assert_eq!(sel.count(), 0);
    }

    #[test]
    fn test_multi_select() {
        let mut sel = SelectionSet::default();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        sel.add(SelectionItem { node_id: id1, kind: SelectionKind::Node, sub_index: 0 });
        sel.add(SelectionItem { node_id: id2, kind: SelectionKind::Face, sub_index: 5 });
        assert_eq!(sel.count(), 2);
        // Dedup: adding same item again should not increase count
        sel.add(SelectionItem { node_id: id1, kind: SelectionKind::Node, sub_index: 0 });
        assert_eq!(sel.count(), 2);
    }

    #[test]
    fn test_gizmo_cycle() {
        let mut g = Gizmo::default();
        assert_eq!(g.mode, GizmoMode::Translate);
        g.cycle_mode();
        assert_eq!(g.mode, GizmoMode::Rotate);
        g.cycle_mode();
        assert_eq!(g.mode, GizmoMode::Scale);
        g.cycle_mode();
        assert_eq!(g.mode, GizmoMode::Translate);
    }

    #[test]
    fn test_measurement_distance() {
        let m = Measurement::distance(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 4.0, 0.0),
        );
        assert!((m.value - 5.0).abs() < 0.001);
        assert_eq!(m.unit, "mm");
    }

    #[test]
    fn test_measurement_angle() {
        let m = Measurement::angle(
            Point3::new(1.0, 0.0, 0.0),
            Point3::origin(),
            Point3::new(0.0, 1.0, 0.0),
        );
        assert!((m.value - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_clipping_plane() {
        let cp = ClippingPlane::xy(5.0);
        assert!(cp.is_visible(&Point3::new(0.0, 0.0, 10.0)));
        assert!(!cp.is_visible(&Point3::new(0.0, 0.0, 3.0)));
        let eq = cp.equation();
        assert_eq!(eq[2], 1.0);
        assert_eq!(eq[3], -5.0);
    }
}
