//! Pre-built dental analysis Python scripts

use crate::interpreter::PythonScript;

/// Generate a Python script that analyzes bone density from HU values
pub fn bone_density_script(hu_values: &[f32]) -> PythonScript {
    let data = serde_json::json!({ "hu_values": hu_values });
    PythonScript::new(
        "bone_density_analysis",
        r#"
import json, math
values = _input['hu_values']
mean_hu = sum(values) / len(values)
variance = sum((v - mean_hu)**2 for v in values) / len(values)
std_hu = math.sqrt(variance)

if mean_hu > 1250:
    density_class = "D1"
elif mean_hu > 850:
    density_class = "D2"
elif mean_hu > 350:
    density_class = "D3"
else:
    density_class = "D4"

print(json.dumps({"mean_hu": mean_hu, "std_hu": std_hu, "density_class": density_class}))
"#,
    )
    .with_data(data)
}

/// Generate a script for caries risk scoring
pub fn caries_risk_script(features: &serde_json::Value) -> PythonScript {
    PythonScript::new(
        "caries_risk",
        r#"
import json
f = _input
score = 0.0
# Simple heuristic scoring
score += f.get('probe_depth_avg', 0) * 0.3
score += f.get('bleeding_index', 0) * 0.2
score += (1 - f.get('oral_hygiene', 1)) * 0.3
score += f.get('sugar_frequency', 0) * 0.2
risk = "low" if score < 0.3 else "medium" if score < 0.6 else "high"
print(json.dumps({"score": round(score, 3), "risk": risk}))
"#,
    )
    .with_data(features.clone())
}

/// Script to convert mesh vertex data and compute statistics
pub fn mesh_stats_script(vertex_count: usize, face_count: usize) -> PythonScript {
    let data = serde_json::json!({ "vertices": vertex_count, "faces": face_count });
    PythonScript::new(
        "mesh_stats",
        r#"
import json, math
v = _input['vertices']
f = _input['faces']
euler = v - f // 2  # simplified
print(json.dumps({"vertex_count": v, "face_count": f, "euler_estimate": euler}))
"#,
    )
    .with_data(data)
}
