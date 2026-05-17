//! S106-S110: Tooth library — anatomy presets, morphology catalog, and matching.
//!
//! Provides a local catalog of tooth anatomy templates used for wax-up,
//! crown design, and prosthetic tooth selection.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use crate::notation::ToothId;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Category of library tooth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToothCategory {
    Anterior,
    Premolar,
    Molar,
}

/// Gender-based morphology hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Morphology {
    Male,
    Female,
    Average,
}

/// Ethnicity-based morphology hint (broad categories per ISO 1942).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ethnicity {
    Caucasian,
    Asian,
    African,
    Universal,
}

/// A tooth anatomy template from the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothTemplate {
    pub id: String,
    pub tooth_id: ToothId,
    pub category: ToothCategory,
    pub morphology: Morphology,
    pub ethnicity: Ethnicity,
    /// Buccal-lingual width (mm).
    pub width_bl: f64,
    /// Mesial-distal width (mm).
    pub width_md: f64,
    /// Height (mm) from CEJ to cusp tip.
    pub height: f64,
    /// Optional mesh vertex positions.
    pub mesh_vertices: Vec<Point3<f64>>,
    /// Optional mesh triangle indices.
    pub mesh_indices: Vec<[u32; 3]>,
}

/// Criteria for searching the library.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchCriteria {
    pub fdi_number: Option<u8>,
    pub category: Option<ToothCategory>,
    pub morphology: Option<Morphology>,
    pub ethnicity: Option<Ethnicity>,
    pub min_width_md: Option<f64>,
    pub max_width_md: Option<f64>,
}

/// Collection of tooth templates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToothCatalog {
    pub templates: Vec<ToothTemplate>,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl ToothTemplate {
    /// Compute the bounding box diagonal of the template mesh.
    pub fn bounding_size(&self) -> f64 {
        if self.mesh_vertices.is_empty() {
            return 0.0;
        }
        let mut min = self.mesh_vertices[0].coords;
        let mut max = min;
        for v in &self.mesh_vertices {
            min = min.inf(&v.coords);
            max = max.sup(&v.coords);
        }
        (max - min).norm()
    }

    /// Scale the template mesh to match a target MD width.
    pub fn scale_to_width(&mut self, target_md: f64) {
        if self.width_md <= 0.0 {
            return;
        }
        let factor = target_md / self.width_md;
        for v in &mut self.mesh_vertices {
            v.coords *= factor;
        }
        self.width_md = target_md;
        self.width_bl *= factor;
        self.height *= factor;
    }
}

impl ToothCatalog {
    /// Add a template to the catalog.
    pub fn add(&mut self, template: ToothTemplate) {
        self.templates.push(template);
    }

    /// Search by criteria; returns matching templates sorted by relevance.
    pub fn search(&self, criteria: &SearchCriteria) -> Vec<&ToothTemplate> {
        let mut results: Vec<&ToothTemplate> = self
            .templates
            .iter()
            .filter(|t| {
                if let Some(fdi) = criteria.fdi_number {
                    if t.tooth_id.fdi != fdi {
                        return false;
                    }
                }
                if let Some(cat) = criteria.category {
                    if t.category != cat {
                        return false;
                    }
                }
                if let Some(morph) = criteria.morphology {
                    if t.morphology != morph && t.morphology != Morphology::Average {
                        return false;
                    }
                }
                if let Some(eth) = criteria.ethnicity {
                    if t.ethnicity != eth && t.ethnicity != Ethnicity::Universal {
                        return false;
                    }
                }
                if let Some(min) = criteria.min_width_md {
                    if t.width_md < min {
                        return false;
                    }
                }
                if let Some(max) = criteria.max_width_md {
                    if t.width_md > max {
                        return false;
                    }
                }
                true
            })
            .collect();
        // Sort by bounding_size descending (larger = more detail = more relevant)
        results.sort_by(|a, b| b.bounding_size().partial_cmp(&a.bounding_size()).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Find the best match for a given FDI number and target dimensions.
    pub fn best_match(
        &self,
        fdi: u8,
        target_md: f64,
        target_bl: f64,
    ) -> Option<&ToothTemplate> {
        self.templates
            .iter()
            .filter(|t| t.tooth_id.fdi == fdi)
            .min_by(|a, b| {
                let da = (a.width_md - target_md).abs() + (a.width_bl - target_bl).abs();
                let db = (b.width_md - target_md).abs() + (b.width_bl - target_bl).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get all templates for a specific tooth number.
    pub fn by_fdi(&self, fdi: u8) -> Vec<&ToothTemplate> {
        self.templates.iter().filter(|t| t.tooth_id.fdi == fdi).collect()
    }

    /// Categories present in the catalog.
    pub fn categories(&self) -> Vec<ToothCategory> {
        let mut cats: Vec<ToothCategory> = self
            .templates
            .iter()
            .map(|t| t.category)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        cats.sort_by_key(|c| *c as u8);
        cats
    }
}

/// Build a default catalog with common anatomical averages (no meshes).
pub fn default_catalog() -> ToothCatalog {
    let mut catalog = ToothCatalog::default();
    // Average adult permanent teeth dimensions (ISO approximate)
    let specs = [
        (11, ToothCategory::Anterior, 8.5, 7.0, 10.5),
        (12, ToothCategory::Anterior, 6.5, 6.0, 9.0),
        (13, ToothCategory::Anterior, 7.5, 8.0, 10.0),
        (14, ToothCategory::Premolar, 7.0, 9.0, 8.5),
        (15, ToothCategory::Premolar, 7.0, 9.0, 8.0),
        (16, ToothCategory::Molar, 10.0, 11.0, 7.5),
        (17, ToothCategory::Molar, 9.0, 10.5, 7.0),
        (18, ToothCategory::Molar, 8.5, 10.0, 6.5),
    ];
    for &(fdi_base, cat, md, bl, h) in &specs {
        for quadrant_tens in [10u8, 20, 30, 40] {
            let fdi = quadrant_tens + (fdi_base % 10);
            if let Some(tooth_id) = ToothId::from_fdi(fdi) {
                catalog.add(ToothTemplate {
                    id: format!("avg-{fdi}"),
                    tooth_id,
                    category: cat,
                    morphology: Morphology::Average,
                    ethnicity: Ethnicity::Universal,
                    width_bl: bl,
                    width_md: md,
                    height: h,
                    mesh_vertices: vec![],
                    mesh_indices: vec![],
                });
            }
        }
    }
    catalog
}

// ---------------------------------------------------------------------------
// Alignment helpers (S108-S110)
// ---------------------------------------------------------------------------

/// Align a template mesh centroid to a target position.
pub fn align_template_to_position(
    template: &mut ToothTemplate,
    target_center: &Point3<f64>,
) {
    if template.mesh_vertices.is_empty() {
        return;
    }
    let sum: Vector3<f64> = template.mesh_vertices.iter().map(|p| p.coords).sum();
    let centroid = sum / template.mesh_vertices.len() as f64;
    let offset = target_center.coords - centroid;
    for v in &mut template.mesh_vertices {
        v.coords += offset;
    }
}

/// Measure similarity between two tooth templates (lower = more similar).
pub fn template_similarity(a: &ToothTemplate, b: &ToothTemplate) -> f64 {
    let md_diff = (a.width_md - b.width_md).abs();
    let bl_diff = (a.width_bl - b.width_bl).abs();
    let h_diff = (a.height - b.height).abs();
    md_diff + bl_diff + h_diff
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_populated() {
        let cat = default_catalog();
        // 8 tooth types × 4 quadrants = 32 templates
        assert_eq!(cat.templates.len(), 32);
    }

    #[test]
    fn search_by_fdi() {
        let cat = default_catalog();
        let results = cat.by_fdi(11);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tooth_id.fdi, 11);
    }

    #[test]
    fn search_by_category() {
        let cat = default_catalog();
        let criteria = SearchCriteria {
            category: Some(ToothCategory::Molar),
            ..Default::default()
        };
        let results = cat.search(&criteria);
        assert!(results.len() >= 12); // 3 molar types × 4 quadrants
    }

    #[test]
    fn best_match_finds_closest() {
        let cat = default_catalog();
        let best = cat.best_match(11, 8.0, 7.0);
        assert!(best.is_some());
        assert_eq!(best.unwrap().tooth_id.fdi, 11);
    }

    #[test]
    fn scale_template() {
        let cat = default_catalog();
        let mut template = cat.templates[0].clone();
        let original_md = template.width_md;
        template.scale_to_width(12.0);
        assert!((template.width_md - 12.0).abs() < 1e-9);
        assert!(template.width_bl != cat.templates[0].width_bl || original_md == 12.0);
    }

    #[test]
    fn similarity_same_is_zero() {
        let cat = default_catalog();
        let s = template_similarity(&cat.templates[0], &cat.templates[0]);
        assert!((s - 0.0).abs() < 1e-9);
    }

    #[test]
    fn align_empty_noop() {
        let mut t = ToothTemplate {
            id: "test".into(),
            tooth_id: ToothId::from_fdi(11).unwrap(),
            category: ToothCategory::Anterior,
            morphology: Morphology::Average,
            ethnicity: Ethnicity::Universal,
            width_bl: 7.0,
            width_md: 8.5,
            height: 10.5,
            mesh_vertices: vec![],
            mesh_indices: vec![],
        };
        align_template_to_position(&mut t, &Point3::new(10.0, 10.0, 10.0));
        // No crash, no change
        assert!(t.mesh_vertices.is_empty());
    }
}
