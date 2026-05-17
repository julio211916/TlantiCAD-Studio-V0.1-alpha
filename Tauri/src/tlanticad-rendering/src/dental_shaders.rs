//! S211-S215: Dental-specific shaders and advanced lighting
//!
//! Shader uniform definitions, SSAO parameters, shadow map config,
//! and environment-lighting (HDRI) descriptors.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────
//  Dental-specific shader presets
// ────────────────────────────────────────────────────────────────────

/// Pre-defined dental material shader configs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DentalShaderPreset {
    Zirconia,
    Ceramic,
    Titanium,
    PMMA,
    Emax,
    Wax,
    Gingiva,
    Bone,
    Resin,
}

/// PBR uniform block sent to the GPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrUniforms {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ambient_occlusion: f32,
    pub emissive: [f32; 3],
    pub subsurface_scattering: f32,
    pub translucency: f32,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
}

impl PbrUniforms {
    pub fn from_preset(preset: DentalShaderPreset) -> Self {
        match preset {
            DentalShaderPreset::Zirconia => Self {
                base_color: [0.96, 0.95, 0.92, 1.0],
                metallic: 0.0, roughness: 0.15,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.3, translucency: 0.25,
                clearcoat: 0.7, clearcoat_roughness: 0.05,
            },
            DentalShaderPreset::Ceramic => Self {
                base_color: [0.95, 0.93, 0.88, 1.0],
                metallic: 0.0, roughness: 0.2,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.4, translucency: 0.3,
                clearcoat: 0.5, clearcoat_roughness: 0.1,
            },
            DentalShaderPreset::Titanium => Self {
                base_color: [0.85, 0.85, 0.87, 1.0],
                metallic: 1.0, roughness: 0.15,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.0, translucency: 0.0,
                clearcoat: 0.0, clearcoat_roughness: 0.0,
            },
            DentalShaderPreset::PMMA => Self {
                base_color: [0.90, 0.85, 0.78, 1.0],
                metallic: 0.0, roughness: 0.3,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.5, translucency: 0.4,
                clearcoat: 0.3, clearcoat_roughness: 0.15,
            },
            DentalShaderPreset::Emax => Self {
                base_color: [0.94, 0.92, 0.87, 1.0],
                metallic: 0.0, roughness: 0.12,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.35, translucency: 0.35,
                clearcoat: 0.8, clearcoat_roughness: 0.04,
            },
            DentalShaderPreset::Wax => Self {
                base_color: [0.6, 0.85, 0.4, 0.85],
                metallic: 0.0, roughness: 0.4,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.8, translucency: 0.7,
                clearcoat: 0.1, clearcoat_roughness: 0.3,
            },
            DentalShaderPreset::Gingiva => Self {
                base_color: [0.85, 0.45, 0.42, 1.0],
                metallic: 0.0, roughness: 0.55,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.6, translucency: 0.15,
                clearcoat: 0.0, clearcoat_roughness: 0.0,
            },
            DentalShaderPreset::Bone => Self {
                base_color: [0.93, 0.90, 0.82, 1.0],
                metallic: 0.0, roughness: 0.65,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.2, translucency: 0.05,
                clearcoat: 0.0, clearcoat_roughness: 0.0,
            },
            DentalShaderPreset::Resin => Self {
                base_color: [0.88, 0.82, 0.72, 1.0],
                metallic: 0.0, roughness: 0.25,
                ambient_occlusion: 1.0, emissive: [0.0; 3],
                subsurface_scattering: 0.45, translucency: 0.35,
                clearcoat: 0.6, clearcoat_roughness: 0.08,
            },
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Shadow mapping
// ────────────────────────────────────────────────────────────────────

/// Shadow map configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowMapConfig {
    pub resolution: u32,
    pub cascade_count: u32,
    pub cascade_splits: Vec<f32>,
    pub bias: f32,
    pub normal_bias: f32,
    pub soft_shadows: bool,
    pub pcf_radius: u32,
}

impl Default for ShadowMapConfig {
    fn default() -> Self {
        Self {
            resolution: 2048,
            cascade_count: 3,
            cascade_splits: vec![0.1, 0.3, 1.0],
            bias: 0.005,
            normal_bias: 0.04,
            soft_shadows: true,
            pcf_radius: 2,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  SSAO
// ────────────────────────────────────────────────────────────────────

/// Screen-space ambient occlusion parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsaoConfig {
    pub enabled: bool,
    pub radius: f32,
    pub bias: f32,
    pub intensity: f32,
    pub sample_count: u32,
    pub blur_passes: u32,
}

impl Default for SsaoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            radius: 0.5,
            bias: 0.025,
            intensity: 1.5,
            sample_count: 32,
            blur_passes: 2,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Environment lighting (HDRI)
// ────────────────────────────────────────────────────────────────────

/// Environment map type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvMapType {
    Equirectangular,
    CubeMap,
    SphericalHarmonics,
}

/// HDRI environment map descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentMap {
    pub name: String,
    pub map_type: EnvMapType,
    pub intensity: f32,
    pub rotation_y: f32,
    pub blur_level: f32,
}

impl EnvironmentMap {
    /// Dental studio environment preset
    pub fn dental_studio() -> Self {
        Self {
            name: "dental_studio".into(),
            map_type: EnvMapType::SphericalHarmonics,
            intensity: 1.0,
            rotation_y: 0.0,
            blur_level: 0.3,
        }
    }

    /// Clean white environment for shade matching
    pub fn shade_matching() -> Self {
        Self {
            name: "shade_matching".into(),
            map_type: EnvMapType::SphericalHarmonics,
            intensity: 1.2,
            rotation_y: 0.0,
            blur_level: 0.8,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
//  Render settings aggregate
// ────────────────────────────────────────────────────────────────────

/// Global render quality settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    pub ssao: SsaoConfig,
    pub shadow: ShadowMapConfig,
    pub environment: EnvironmentMap,
    pub msaa_samples: u32,
    pub tone_mapping: ToneMapping,
    pub gamma: f32,
    pub exposure: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToneMapping {
    Linear,
    Reinhard,
    ACES,
    Filmic,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            ssao: SsaoConfig::default(),
            shadow: ShadowMapConfig::default(),
            environment: EnvironmentMap::dental_studio(),
            msaa_samples: 4,
            tone_mapping: ToneMapping::ACES,
            gamma: 2.2,
            exposure: 1.0,
        }
    }
}

impl RenderSettings {
    /// High-quality preset for screenshots/presentation
    pub fn high_quality() -> Self {
        Self {
            ssao: SsaoConfig { sample_count: 64, blur_passes: 3, ..Default::default() },
            shadow: ShadowMapConfig { resolution: 4096, cascade_count: 4,
                cascade_splits: vec![0.05, 0.15, 0.4, 1.0], ..Default::default() },
            msaa_samples: 8,
            ..Default::default()
        }
    }

    /// Performance preset for real-time editing
    pub fn performance() -> Self {
        Self {
            ssao: SsaoConfig { sample_count: 16, blur_passes: 1, ..Default::default() },
            shadow: ShadowMapConfig { resolution: 1024, cascade_count: 2,
                cascade_splits: vec![0.3, 1.0], ..Default::default() },
            msaa_samples: 2,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dental_shader_presets() {
        let zi = PbrUniforms::from_preset(DentalShaderPreset::Zirconia);
        assert!(zi.translucency > 0.0);
        assert_eq!(zi.metallic, 0.0);

        let ti = PbrUniforms::from_preset(DentalShaderPreset::Titanium);
        assert_eq!(ti.metallic, 1.0);
        assert_eq!(ti.translucency, 0.0);
    }

    #[test]
    fn test_all_presets_valid() {
        let presets = [
            DentalShaderPreset::Zirconia, DentalShaderPreset::Ceramic,
            DentalShaderPreset::Titanium, DentalShaderPreset::PMMA,
            DentalShaderPreset::Emax, DentalShaderPreset::Wax,
            DentalShaderPreset::Gingiva, DentalShaderPreset::Bone,
            DentalShaderPreset::Resin,
        ];
        for p in presets {
            let u = PbrUniforms::from_preset(p);
            assert!(u.roughness >= 0.0 && u.roughness <= 1.0);
            assert!(u.metallic >= 0.0 && u.metallic <= 1.0);
        }
    }

    #[test]
    fn test_shadow_config() {
        let cfg = ShadowMapConfig::default();
        assert_eq!(cfg.resolution, 2048);
        assert_eq!(cfg.cascade_count, 3);
        assert_eq!(cfg.cascade_splits.len(), 3);
    }

    #[test]
    fn test_ssao_config() {
        let cfg = SsaoConfig::default();
        assert_eq!(cfg.sample_count, 32);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_render_settings_presets() {
        let hq = RenderSettings::high_quality();
        assert_eq!(hq.msaa_samples, 8);
        assert_eq!(hq.shadow.resolution, 4096);

        let perf = RenderSettings::performance();
        assert_eq!(perf.msaa_samples, 2);
        assert_eq!(perf.shadow.resolution, 1024);
    }

    #[test]
    fn test_environment_map() {
        let ds = EnvironmentMap::dental_studio();
        assert_eq!(ds.map_type, EnvMapType::SphericalHarmonics);

        let sm = EnvironmentMap::shade_matching();
        assert!(sm.intensity > 1.0);
    }
}
