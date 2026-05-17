//! S201-S205: Render pipeline configuration and GPU resource types
//!
//! Abstractions for the render pipeline (vertex layout, shader stages, depth/stencil).
//! Actual GPU creation (wgpu) is done in the `gpu` crate; here we define the
//! portable configuration structs consumed by both the Rust backend and R3F frontend.

use serde::{Deserialize, Serialize};

/// Vertex attribute format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
}

impl VertexFormat {
    pub fn byte_size(self) -> u32 {
        match self {
            Self::Float32 => 4,
            Self::Float32x2 => 8,
            Self::Float32x3 => 12,
            Self::Float32x4 => 16,
            Self::Uint32 => 4,
        }
    }
}

/// A single vertex attribute in a vertex layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAttribute {
    pub name: String,
    pub format: VertexFormat,
    pub offset: u32,
    pub location: u32,
}

/// Describes the vertex layout for a mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexLayout {
    pub attributes: Vec<VertexAttribute>,
    pub stride: u32,
}

impl VertexLayout {
    /// Standard dental mesh layout: position (vec3) + normal (vec3) + uv (vec2)
    pub fn standard_dental() -> Self {
        Self {
            attributes: vec![
                VertexAttribute { name: "position".into(), format: VertexFormat::Float32x3, offset: 0, location: 0 },
                VertexAttribute { name: "normal".into(), format: VertexFormat::Float32x3, offset: 12, location: 1 },
                VertexAttribute { name: "uv".into(), format: VertexFormat::Float32x2, offset: 24, location: 2 },
            ],
            stride: 32,
        }
    }

    /// Position-only layout for depth/shadow passes
    pub fn position_only() -> Self {
        Self {
            attributes: vec![
                VertexAttribute { name: "position".into(), format: VertexFormat::Float32x3, offset: 0, location: 0 },
            ],
            stride: 12,
        }
    }
}

/// Shader stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// Render pipeline descriptor — portable definition of a GPU pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderPipelineDesc {
    pub label: String,
    pub vertex_layout: VertexLayout,
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub depth_test: bool,
    pub depth_write: bool,
    pub cull_back_face: bool,
    pub blend_enabled: bool,
    pub wireframe: bool,
    pub sample_count: u32,
}

impl Default for RenderPipelineDesc {
    fn default() -> Self {
        Self {
            label: "default".into(),
            vertex_layout: VertexLayout::standard_dental(),
            vertex_shader: "dental_pbr.vert".into(),
            fragment_shader: "dental_pbr.frag".into(),
            depth_test: true,
            depth_write: true,
            cull_back_face: true,
            blend_enabled: false,
            wireframe: false,
            sample_count: 4,
        }
    }
}

impl RenderPipelineDesc {
    /// Transparent material pipeline (for X-ray mode, ghost views)
    pub fn transparent() -> Self {
        Self {
            label: "transparent".into(),
            blend_enabled: true,
            depth_write: false,
            ..Default::default()
        }
    }

    /// Wireframe overlay pipeline
    pub fn wireframe() -> Self {
        Self {
            label: "wireframe".into(),
            wireframe: true,
            cull_back_face: false,
            fragment_shader: "solid_color.frag".into(),
            ..Default::default()
        }
    }

    /// Depth-only pipeline for shadow maps
    pub fn depth_only() -> Self {
        Self {
            label: "depth_only".into(),
            vertex_layout: VertexLayout::position_only(),
            fragment_shader: "".into(),
            ..Default::default()
        }
    }
}

/// Texture descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureDesc {
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub mip_levels: u32,
}

/// Supported texture formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8Srgb,
    Rgba16Float,
    Depth32Float,
    Depth24Stencil8,
    R8Unorm,
    Rg8Unorm,
}

/// Texture usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureUsage {
    Sampled,
    RenderTarget,
    DepthStencil,
    Storage,
}

/// Sampler descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplerDesc {
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mip_filter: FilterMode,
    pub address_u: AddressMode,
    pub address_v: AddressMode,
    pub max_anisotropy: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterMode { Nearest, Linear }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressMode { Repeat, ClampToEdge, MirrorRepeat }

impl Default for SamplerDesc {
    fn default() -> Self {
        Self {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mip_filter: FilterMode::Linear,
            address_u: AddressMode::Repeat,
            address_v: AddressMode::Repeat,
            max_anisotropy: 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_dental_layout() {
        let layout = VertexLayout::standard_dental();
        assert_eq!(layout.attributes.len(), 3);
        assert_eq!(layout.stride, 32);
        assert_eq!(layout.attributes[0].format, VertexFormat::Float32x3);
    }

    #[test]
    fn test_pipeline_presets() {
        let default = RenderPipelineDesc::default();
        assert!(default.depth_test);
        assert!(default.depth_write);

        let trans = RenderPipelineDesc::transparent();
        assert!(trans.blend_enabled);
        assert!(!trans.depth_write);

        let wf = RenderPipelineDesc::wireframe();
        assert!(wf.wireframe);
        assert!(!wf.cull_back_face);
    }

    #[test]
    fn test_vertex_format_sizes() {
        assert_eq!(VertexFormat::Float32.byte_size(), 4);
        assert_eq!(VertexFormat::Float32x3.byte_size(), 12);
        assert_eq!(VertexFormat::Float32x4.byte_size(), 16);
    }

    #[test]
    fn test_texture_desc() {
        let tex = TextureDesc {
            label: "depth".into(),
            width: 1024,
            height: 1024,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::DepthStencil,
            mip_levels: 1,
        };
        assert_eq!(tex.format, TextureFormat::Depth32Float);
    }

    #[test]
    fn test_sampler_default() {
        let s = SamplerDesc::default();
        assert_eq!(s.max_anisotropy, 16);
        assert_eq!(s.min_filter, FilterMode::Linear);
    }
}
