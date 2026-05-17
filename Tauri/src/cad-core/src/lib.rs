pub mod math;
pub mod mesh;
pub mod geometry;
pub mod error;

pub use error::{CadError, Result};

// Re-export frequently used types
pub use math::{Point3f, Vec3f, Mat4f, Aabb, Ray};
pub use mesh::{HeMesh, MeshId, Vertex, Face};
pub use geometry::{Transform3D, ToothNumber, Millimeters, Degrees};
