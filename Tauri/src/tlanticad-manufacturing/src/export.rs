//! S348-S350: Manufacturing Export Formats
//!
//! Export restoration data to STL, 3MF, and manufacturing reports.

use serde::{Deserialize, Serialize};

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    StlBinary,
    StlAscii,
    ThreeMF,
    Obj,
    Ply,
    Step,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::StlBinary | Self::StlAscii => "stl",
            Self::ThreeMF => "3mf",
            Self::Obj => "obj",
            Self::Ply => "ply",
            Self::Step => "step",
        }
    }

    pub fn is_mesh_format(&self) -> bool {
        !matches!(self, Self::Step)
    }
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub include_supports: bool,
    pub include_base: bool,
    pub scale_factor: f64,
    pub units: ExportUnits,
    pub coordinate_system: CoordinateSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportUnits { Millimeters, Inches, Microns }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinateSystem { RightHandedZUp, RightHandedYUp }

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::StlBinary,
            include_supports: false,
            include_base: false,
            scale_factor: 1.0,
            units: ExportUnits::Millimeters,
            coordinate_system: CoordinateSystem::RightHandedZUp,
        }
    }
}

/// Manufacturing ticket (work order)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingTicket {
    pub order_id: String,
    pub patient_id: String,
    pub items: Vec<ManufacturingItem>,
    pub priority: Priority,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority { Urgent, Normal, Low }

/// Single item to manufacture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingItem {
    pub name: String,
    pub material: String,
    pub method: ManufacturingMethod,
    pub tooth_positions: Vec<u8>,
    pub shade: Option<String>,
    pub file_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManufacturingMethod {
    Milling,
    Printing,
    Pressing,
    Casting,
    Sintering,
}

impl std::fmt::Display for ManufacturingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Milling => write!(f, "Milling"),
            Self::Printing => write!(f, "Printing"),
            Self::Pressing => write!(f, "Pressing"),
            Self::Casting => write!(f, "Casting"),
            Self::Sintering => write!(f, "Sintering"),
        }
    }
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub size_bytes: usize,
    pub triangle_count: usize,
    pub vertex_count: usize,
    pub data: Vec<u8>,
}

/// Generate a binary STL header + placeholder for a mesh
pub fn export_stl_binary(vertices: &[[f64; 3]], triangles: &[[usize; 3]]) -> ExportResult {
    let mut data = Vec::with_capacity(84 + triangles.len() * 50);

    // 80-byte header
    let header = b"TlantiCAD Manufacturing Export - Binary STL";
    data.extend_from_slice(header);
    data.resize(80, 0u8);

    // Triangle count (u32 LE)
    let tri_count = triangles.len() as u32;
    data.extend_from_slice(&tri_count.to_le_bytes());

    for tri in triangles {
        let v0 = vertices.get(tri[0]).copied().unwrap_or([0.0; 3]);
        let v1 = vertices.get(tri[1]).copied().unwrap_or([0.0; 3]);
        let v2 = vertices.get(tri[2]).copied().unwrap_or([0.0; 3]);

        // Compute face normal
        let u = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let v = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let nn = if len > 1e-12 { [n[0] / len, n[1] / len, n[2] / len] } else { [0.0; 3] };

        // Normal (3 x f32)
        for c in nn { data.extend_from_slice(&(c as f32).to_le_bytes()); }
        // Vertices (3 x 3 x f32)
        for vert in [v0, v1, v2] {
            for c in vert { data.extend_from_slice(&(c as f32).to_le_bytes()); }
        }
        // Attribute byte count
        data.extend_from_slice(&0u16.to_le_bytes());
    }

    ExportResult {
        format: ExportFormat::StlBinary,
        size_bytes: data.len(),
        triangle_count: triangles.len(),
        vertex_count: vertices.len(),
        data,
    }
}

/// Generate manufacturing ticket
pub fn create_ticket(
    order_id: impl Into<String>,
    patient_id: impl Into<String>,
    items: Vec<ManufacturingItem>,
    priority: Priority,
) -> ManufacturingTicket {
    ManufacturingTicket {
        order_id: order_id.into(),
        patient_id: patient_id.into(),
        items,
        priority,
        notes: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::StlBinary.extension(), "stl");
        assert_eq!(ExportFormat::ThreeMF.extension(), "3mf");
        assert_eq!(ExportFormat::Step.extension(), "step");
    }

    #[test]
    fn test_stl_binary_export() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let result = export_stl_binary(&verts, &tris);
        assert_eq!(result.triangle_count, 1);
        assert_eq!(result.vertex_count, 3);
        // 80 header + 4 count + 50 per triangle
        assert_eq!(result.size_bytes, 84 + 50);
    }

    #[test]
    fn test_manufacturing_ticket() {
        let item = ManufacturingItem {
            name: "Crown #14".into(),
            material: "Zirconia HT".into(),
            method: ManufacturingMethod::Milling,
            tooth_positions: vec![14],
            shade: Some("A2".into()),
            file_path: "crown_14.stl".into(),
        };
        let ticket = create_ticket("ORD-001", "PAT-123", vec![item], Priority::Normal);
        assert_eq!(ticket.items.len(), 1);
        assert_eq!(ticket.priority, Priority::Normal);
    }

    #[test]
    fn test_export_config_default() {
        let cfg = ExportConfig::default();
        assert_eq!(cfg.format, ExportFormat::StlBinary);
        assert!((cfg.scale_factor - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mesh_format_check() {
        assert!(ExportFormat::StlBinary.is_mesh_format());
        assert!(!ExportFormat::Step.is_mesh_format());
    }

    #[test]
    fn test_stl_empty() {
        let result = export_stl_binary(&[], &[]);
        assert_eq!(result.triangle_count, 0);
        assert_eq!(result.size_bytes, 84);
    }

    #[test]
    fn test_ticket_priority_ordering() {
        assert!((Priority::Urgent as u8) < (Priority::Normal as u8));
    }

    #[test]
    fn test_export_format_extensions() {
        assert_eq!(ExportFormat::StlBinary.extension(), "stl");
        assert_eq!(ExportFormat::Obj.extension(), "obj");
        assert_eq!(ExportFormat::Step.extension(), "step");
    }

    #[test]
    fn test_manufacturing_method_display() {
        assert_eq!(ManufacturingMethod::Milling.to_string(), "Milling");
        assert_eq!(ManufacturingMethod::Printing.to_string(), "Printing");
    }
}
