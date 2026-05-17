//! S386-S390: Workflow Templates
//!
//! Predefined and custom workflow templates for common dental lab case types.

use serde::{Deserialize, Serialize};

/// Step in a workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStep {
    pub name: String,
    pub description: String,
    pub estimated_minutes: u32,
    pub required_role: Option<String>,
    pub auto_advance: bool,
}

/// Workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub steps: Vec<TemplateStep>,
    pub is_builtin: bool,
}

impl WorkflowTemplate {
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            category: category.into(),
            steps: Vec::new(),
            is_builtin: false,
        }
    }

    pub fn add_step(&mut self, name: impl Into<String>, desc: impl Into<String>, est_min: u32) {
        self.steps.push(TemplateStep {
            name: name.into(),
            description: desc.into(),
            estimated_minutes: est_min,
            required_role: None,
            auto_advance: false,
        });
    }

    pub fn total_estimated_minutes(&self) -> u32 {
        self.steps.iter().map(|s| s.estimated_minutes).sum()
    }

    pub fn step_count(&self) -> usize { self.steps.len() }
}

/// Built-in templates for common dental cases
pub fn builtin_templates() -> Vec<WorkflowTemplate> {
    let mut templates = Vec::new();

    // Single crown
    let mut crown = WorkflowTemplate::new("Single Crown", "Fixed Prosthetics");
    crown.description = "Standard single-crown workflow from scan to ship".into();
    crown.is_builtin = true;
    crown.add_step("Import Scan", "Import intraoral or model scan", 5);
    crown.add_step("Margin Detection", "Auto-detect or manually trace margin line", 10);
    crown.add_step("Die Trimming", "Trim virtual die", 5);
    crown.add_step("Wax-Up / Design", "Design crown anatomy", 25);
    crown.add_step("Occlusal Adjustment", "Verify & adjust occlusal contacts", 10);
    crown.add_step("CAM Nesting", "Nest into milling blank", 5);
    crown.add_step("Milling", "Mill on CNC", 45);
    crown.add_step("Sintering", "Sinter if zirconia", 480);
    crown.add_step("Staining & Glazing", "Characterize with stains/glaze", 20);
    crown.add_step("QC Inspection", "Verify fit, contacts, shade", 10);
    crown.add_step("Ship", "Package and ship", 5);
    templates.push(crown);

    // 3-unit bridge
    let mut bridge = WorkflowTemplate::new("3-Unit Bridge", "Fixed Prosthetics");
    bridge.description = "Three-unit bridge workflow".into();
    bridge.is_builtin = true;
    bridge.add_step("Import Scan", "Import opposing and preps", 5);
    bridge.add_step("Margin Detection", "Trace margins on both abutments", 15);
    bridge.add_step("Framework Design", "Design bridge framework", 30);
    bridge.add_step("Pontic Design", "Shape pontic anatomy", 15);
    bridge.add_step("Connector Verification", "Verify connector cross-sections", 5);
    bridge.add_step("CAM", "Nest and generate toolpath", 10);
    bridge.add_step("Milling", "Mill framework", 60);
    bridge.add_step("Post-Process", "Sinter / heat-treat", 480);
    bridge.add_step("Layering / Stain", "Apply ceramic or stain", 40);
    bridge.add_step("QC & Ship", "Final inspection and ship", 15);
    templates.push(bridge);

    // Surgical guide
    let mut guide = WorkflowTemplate::new("Surgical Guide", "Implantology");
    guide.description = "Implant surgical guide workflow".into();
    guide.is_builtin = true;
    guide.add_step("DICOM/CBCT Import", "Import CBCT data", 10);
    guide.add_step("Segmentation", "Segment bone and teeth", 20);
    guide.add_step("Implant Planning", "Place virtual implants", 30);
    guide.add_step("Guide Design", "Design guide body and sleeves", 25);
    guide.add_step("3D Print", "Print guide in biocompatible resin", 120);
    guide.add_step("Post-Cure", "UV post-cure cycle", 30);
    guide.add_step("QC", "Verify sleeve positions", 15);
    templates.push(guide);

    templates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates() {
        let t = builtin_templates();
        assert!(t.len() >= 3);
        assert!(t.iter().all(|t| t.is_builtin));
    }

    #[test]
    fn test_single_crown_template() {
        let t = builtin_templates();
        let crown = t.iter().find(|t| t.name == "Single Crown").unwrap();
        assert_eq!(crown.step_count(), 11);
        assert!(crown.total_estimated_minutes() > 0);
    }

    #[test]
    fn test_custom_template() {
        let mut tpl = WorkflowTemplate::new("Custom Denture", "Removable");
        tpl.add_step("Scan", "Import scan", 5);
        tpl.add_step("Design", "Design denture", 30);
        assert_eq!(tpl.step_count(), 2);
        assert_eq!(tpl.total_estimated_minutes(), 35);
    }
}
