//! S236-S245: Advanced visualization — cross-section, exploded view,
//! 3D annotations, dimension lines, callouts

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ────────────────────────────────────────────────────────────────────
//  Cross-section
// ────────────────────────────────────────────────────────────────────

/// Cross-section plane definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSection {
    pub id: Uuid,
    pub plane_normal: [f64; 3],
    pub plane_distance: f64,
    pub enabled: bool,
    pub fill_color: [f32; 4],
    pub outline_color: [f32; 4],
    pub outline_width: f32,
    pub show_fill: bool,
}

impl CrossSection {
    pub fn sagittal(offset: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            plane_normal: [1.0, 0.0, 0.0],
            plane_distance: offset,
            enabled: true,
            fill_color: [0.8, 0.6, 0.5, 0.3],
            outline_color: [1.0, 0.2, 0.2, 1.0],
            outline_width: 2.0,
            show_fill: true,
        }
    }

    pub fn coronal(offset: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            plane_normal: [0.0, 1.0, 0.0],
            plane_distance: offset,
            enabled: true,
            fill_color: [0.5, 0.8, 0.6, 0.3],
            outline_color: [0.2, 1.0, 0.2, 1.0],
            outline_width: 2.0,
            show_fill: true,
        }
    }

    pub fn axial(offset: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            plane_normal: [0.0, 0.0, 1.0],
            plane_distance: offset,
            enabled: true,
            fill_color: [0.5, 0.6, 0.8, 0.3],
            outline_color: [0.2, 0.2, 1.0, 1.0],
            outline_width: 2.0,
            show_fill: true,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Exploded view
// ────────────────────────────────────────────────────────────────────

/// Explosion direction per node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplosionComponent {
    pub node_id: Uuid,
    pub direction: [f64; 3],
    pub distance: f64,
    pub order: u32,
}

/// Exploded view state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplodedView {
    pub components: Vec<ExplosionComponent>,
    pub factor: f64,  // 0.0 = assembled, 1.0 = fully exploded
    pub center: [f64; 3],
    pub auto_center: bool,
}

impl ExplodedView {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            factor: 0.0,
            center: [0.0; 3],
            auto_center: true,
        }
    }

    /// Get the displacement for a component at the current factor
    pub fn displacement(&self, node_id: &Uuid) -> [f64; 3] {
        if let Some(comp) = self.components.iter().find(|c| &c.node_id == node_id) {
            let d = comp.distance * self.factor;
            [comp.direction[0] * d, comp.direction[1] * d, comp.direction[2] * d]
        } else {
            [0.0; 3]
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  3D Annotations
// ────────────────────────────────────────────────────────────────────

/// Annotation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnnotationType {
    Text,
    Arrow,
    Callout,
    DimensionLine,
    RadiusDimension,
    AngleDimension,
    Leader,
    Note,
}

/// A 3D annotation in the scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    pub annotation_type: AnnotationType,
    pub text: String,
    pub anchor_point: [f64; 3],
    pub end_point: Option<[f64; 3]>,
    pub third_point: Option<[f64; 3]>,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub font_size: f32,
    pub color: [f32; 4],
    pub line_width: f32,
    pub visible: bool,
    pub layer: String,
}

impl Annotation {
    /// Linear dimension between two points
    pub fn dimension_line(p1: [f64; 3], p2: [f64; 3]) -> Self {
        let dx = p2[0] - p1[0];
        let dy = p2[1] - p1[1];
        let dz = p2[2] - p1[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        Self {
            id: Uuid::new_v4(),
            annotation_type: AnnotationType::DimensionLine,
            text: format!("{:.2} mm", dist),
            anchor_point: p1,
            end_point: Some(p2),
            third_point: None,
            value: Some(dist),
            unit: Some("mm".into()),
            font_size: 12.0,
            color: [1.0, 1.0, 1.0, 1.0],
            line_width: 1.5,
            visible: true,
            layer: "dimensions".into(),
        }
    }

    /// Angle dimension between three points (vertex at anchor)
    pub fn angle_dimension(a: [f64; 3], vertex: [f64; 3], b: [f64; 3]) -> Self {
        let v1 = Vector3::new(a[0] - vertex[0], a[1] - vertex[1], a[2] - vertex[2]).normalize();
        let v2 = Vector3::new(b[0] - vertex[0], b[1] - vertex[1], b[2] - vertex[2]).normalize();
        let angle = v1.dot(&v2).clamp(-1.0, 1.0).acos().to_degrees();
        Self {
            id: Uuid::new_v4(),
            annotation_type: AnnotationType::AngleDimension,
            text: format!("{:.1}°", angle),
            anchor_point: vertex,
            end_point: Some(a),
            third_point: Some(b),
            value: Some(angle),
            unit: Some("°".into()),
            font_size: 12.0,
            color: [1.0, 1.0, 0.0, 1.0],
            line_width: 1.5,
            visible: true,
            layer: "dimensions".into(),
        }
    }

    /// Text callout with leader line
    pub fn callout(text: impl Into<String>, anchor: [f64; 3], label_pos: [f64; 3]) -> Self {
        Self {
            id: Uuid::new_v4(),
            annotation_type: AnnotationType::Callout,
            text: text.into(),
            anchor_point: anchor,
            end_point: Some(label_pos),
            third_point: None,
            value: None,
            unit: None,
            font_size: 14.0,
            color: [0.9, 0.9, 0.9, 1.0],
            line_width: 1.0,
            visible: true,
            layer: "annotations".into(),
        }
    }
}

/// Annotation layer manager
#[derive(Debug, Clone, Default)]
pub struct AnnotationLayer {
    pub annotations: Vec<Annotation>,
}

impl AnnotationLayer {
    pub fn add(&mut self, ann: Annotation) -> Uuid {
        let id = ann.id;
        self.annotations.push(ann);
        id
    }

    pub fn remove(&mut self, id: &Uuid) -> Option<Annotation> {
        if let Some(pos) = self.annotations.iter().position(|a| &a.id == id) {
            Some(self.annotations.remove(pos))
        } else {
            None
        }
    }

    pub fn get_visible(&self) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.visible).collect()
    }

    pub fn set_layer_visibility(&mut self, layer: &str, visible: bool) {
        for ann in &mut self.annotations {
            if ann.layer == layer {
                ann.visible = visible;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_section_presets() {
        let sag = CrossSection::sagittal(0.0);
        assert_eq!(sag.plane_normal, [1.0, 0.0, 0.0]);
        let cor = CrossSection::coronal(5.0);
        assert_eq!(cor.plane_distance, 5.0);
        let ax = CrossSection::axial(-2.0);
        assert_eq!(ax.plane_normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_exploded_view() {
        let node_id = Uuid::new_v4();
        let mut ev = ExplodedView::new();
        ev.components.push(ExplosionComponent {
            node_id,
            direction: [1.0, 0.0, 0.0],
            distance: 10.0,
            order: 0,
        });
        ev.factor = 0.5;
        let d = ev.displacement(&node_id);
        assert!((d[0] - 5.0).abs() < 0.001);
        assert_eq!(d[1], 0.0);
    }

    #[test]
    fn test_dimension_line() {
        let ann = Annotation::dimension_line(
            [0.0, 0.0, 0.0],
            [3.0, 4.0, 0.0],
        );
        assert!((ann.value.unwrap() - 5.0).abs() < 0.001);
        assert_eq!(ann.annotation_type, AnnotationType::DimensionLine);
    }

    #[test]
    fn test_angle_dimension() {
        let ann = Annotation::angle_dimension(
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((ann.value.unwrap() - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_annotation_layer() {
        let mut layer = AnnotationLayer::default();
        let id = layer.add(Annotation::callout("Test", [0.0; 3], [5.0, 5.0, 0.0]));
        assert_eq!(layer.get_visible().len(), 1);
        layer.remove(&id);
        assert!(layer.get_visible().is_empty());
    }

    #[test]
    fn test_layer_visibility() {
        let mut layer = AnnotationLayer::default();
        layer.add(Annotation::dimension_line([0.0; 3], [1.0, 0.0, 0.0]));
        layer.add(Annotation::callout("Note", [0.0; 3], [1.0; 3]));
        layer.set_layer_visibility("dimensions", false);
        let visible = layer.get_visible();
        assert_eq!(visible.len(), 1); // only callout visible
        assert_eq!(visible[0].annotation_type, AnnotationType::Callout);
    }
}
