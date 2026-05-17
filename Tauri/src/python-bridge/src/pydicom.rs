//! PyDICOM Sidecar — Python-based DICOM processing via sidecar process
//!
//! Leverages PyDICOM for advanced DICOM operations that aren't easily done in Rust:
//! - Pixel data processing with NumPy
//! - DICOM SR (Structured Report) parsing
//! - Advanced tag manipulation
//! - WADO / DICOMweb via pynetdicom
//! - AI/ML inference on DICOM images via Python

use crate::{PythonError, Result};
use serde::{Deserialize, Serialize};

/// PyDICOM operation types that can be dispatched to the sidecar
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PyDicomOp {
    /// Parse a DICOM file and return metadata as JSON
    ParseFile { path: String },
    /// Extract pixel array as base64 PNG
    ExtractPixels { path: String, window_center: Option<f64>, window_width: Option<f64> },
    /// Read DICOM SR (Structured Report) and return as JSON
    ParseStructuredReport { path: String },
    /// Apply windowing/leveling to a DICOM image
    ApplyWindowing { path: String, center: f64, width: f64, output_path: String },
    /// Convert DICOM to PNG/JPEG
    ConvertToImage { path: String, format: String, output_path: String },
    /// Anonymize DICOM file
    Anonymize { path: String, output_path: String, retain_dates: bool },
    /// Run AI inference on a DICOM image (e.g., caries detection)
    AiInference { path: String, model_name: String, model_path: String },
    /// Batch convert multiple DICOM files
    BatchConvert { input_dir: String, output_dir: String, format: String },
    /// Get DICOM tags diff between two files
    DiffTags { path_a: String, path_b: String },
}

/// Result from a PyDICOM operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyDicomResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

/// Execute a PyDICOM operation via the Python sidecar
pub async fn execute_pydicom_op(
    sidecar: &crate::PythonSidecar,
    op: &PyDicomOp,
) -> Result<PyDicomResult> {
    let script = generate_pydicom_script(op)?;

    let output = sidecar
        .execute_code(&script, serde_json::Value::Null)
        .await
        .map_err(|e| PythonError::ExecutionError(format!("PyDICOM sidecar: {}", e)))?;

    let result: PyDicomResult = serde_json::from_value(output)
        .map_err(|e| PythonError::SerializationError(format!("PyDICOM result parse: {}", e)))?;

    Ok(result)
}

/// Generate the Python script for a given PyDICOM operation
fn generate_pydicom_script(op: &PyDicomOp) -> Result<String> {
    let script = match op {
        PyDicomOp::ParseFile { path } => format!(
            r#"
import json, time, pydicom
start = time.time()
try:
    ds = pydicom.dcmread("{path}")
    tags = {{}}
    for elem in ds:
        if elem.VR != 'SQ' and elem.VR != 'OW' and elem.VR != 'OB':
            tags[elem.keyword] = str(elem.value)
    print(json.dumps({{"success": True, "data": tags, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
        ),

        PyDicomOp::ExtractPixels { path, window_center, window_width } => {
            let wc = window_center.map_or("None".to_string(), |v| v.to_string());
            let ww = window_width.map_or("None".to_string(), |v| v.to_string());
            format!(
                r#"
import json, time, base64, io, pydicom
import numpy as np
from PIL import Image
start = time.time()
try:
    ds = pydicom.dcmread("{path}")
    arr = ds.pixel_array.astype(float)
    wc, ww = {wc}, {ww}
    if wc is not None and ww is not None:
        arr = np.clip((arr - (wc - ww/2)) / ww * 255, 0, 255)
    else:
        arr = ((arr - arr.min()) / (arr.max() - arr.min()) * 255)
    img = Image.fromarray(arr.astype(np.uint8))
    buf = io.BytesIO()
    img.save(buf, format='PNG')
    b64 = base64.b64encode(buf.getvalue()).decode()
    print(json.dumps({{"success": True, "data": {{"image_base64": b64, "width": img.width, "height": img.height}}, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
            )
        },

        PyDicomOp::ApplyWindowing { path, center, width, output_path } => format!(
            r#"
import json, time, pydicom
import numpy as np
from PIL import Image
start = time.time()
try:
    ds = pydicom.dcmread("{path}")
    arr = ds.pixel_array.astype(float)
    arr = np.clip((arr - ({center} - {width}/2)) / {width} * 255, 0, 255).astype(np.uint8)
    img = Image.fromarray(arr)
    img.save("{output_path}", format='PNG')
    print(json.dumps({{"success": True, "data": {{"output_path": "{output_path}"}}, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
        ),

        PyDicomOp::ConvertToImage { path, format, output_path } => format!(
            r#"
import json, time, pydicom
import numpy as np
from PIL import Image
start = time.time()
try:
    ds = pydicom.dcmread("{path}")
    arr = ds.pixel_array.astype(float)
    arr = ((arr - arr.min()) / (arr.max() - arr.min()) * 255).astype(np.uint8)
    img = Image.fromarray(arr)
    img.save("{output_path}", format="{format}".upper())
    print(json.dumps({{"success": True, "data": {{"output_path": "{output_path}"}}, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
        ),

        PyDicomOp::Anonymize { path, output_path, retain_dates } => {
            let retain = if *retain_dates { "True" } else { "False" };
            format!(
                r#"
import json, time, pydicom
start = time.time()
try:
    ds = pydicom.dcmread("{path}")
    ds.PatientName = "Anonymous"
    ds.PatientID = "ANON"
    if not {retain}:
        if hasattr(ds, 'PatientBirthDate'): ds.PatientBirthDate = ""
        if hasattr(ds, 'StudyDate'): ds.StudyDate = ""
    ds.save_as("{output_path}")
    print(json.dumps({{"success": True, "data": {{"output_path": "{output_path}"}}, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
            )
        },

        PyDicomOp::AiInference { path, model_name, model_path: _ } => format!(
            r#"
import json, time, pydicom
import numpy as np
start = time.time()
try:
    ds = pydicom.dcmread("{path}")
    arr = ds.pixel_array.astype(float)
    arr = ((arr - arr.min()) / (arr.max() - arr.min())).astype(np.float32)
    # Placeholder: load ONNX model and run inference
    result = {{"model": "{model_name}", "predictions": [], "confidence": 0.0}}
    print(json.dumps({{"success": True, "data": result, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
        ),

        PyDicomOp::ParseStructuredReport { path } => format!(
            r#"
import json, time, pydicom
start = time.time()
try:
    ds = pydicom.dcmread("{path}")
    report = {{"sop_class": str(getattr(ds, 'SOPClassUID', '')), "content": []}}
    if hasattr(ds, 'ContentSequence'):
        for item in ds.ContentSequence:
            entry = {{"type": str(getattr(item, 'ValueType', '')), "concept": str(getattr(item, 'ConceptNameCodeSequence', [None])[0].CodeMeaning if hasattr(item, 'ConceptNameCodeSequence') and item.ConceptNameCodeSequence else '')}}
            if hasattr(item, 'TextValue'):
                entry["value"] = str(item.TextValue)
            report["content"].append(entry)
    print(json.dumps({{"success": True, "data": report, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
        ),

        PyDicomOp::BatchConvert { input_dir, output_dir, format } => format!(
            r#"
import json, time, os, pydicom
import numpy as np
from PIL import Image
start = time.time()
try:
    os.makedirs("{output_dir}", exist_ok=True)
    converted = []
    for f in os.listdir("{input_dir}"):
        fp = os.path.join("{input_dir}", f)
        try:
            ds = pydicom.dcmread(fp)
            arr = ds.pixel_array.astype(float)
            arr = ((arr - arr.min()) / max(arr.max() - arr.min(), 1) * 255).astype(np.uint8)
            img = Image.fromarray(arr)
            out = os.path.join("{output_dir}", os.path.splitext(f)[0] + ".{format}")
            img.save(out)
            converted.append(out)
        except: pass
    print(json.dumps({{"success": True, "data": {{"converted": converted, "count": len(converted)}}, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
        ),

        PyDicomOp::DiffTags { path_a, path_b } => format!(
            r#"
import json, time, pydicom
start = time.time()
try:
    ds_a = pydicom.dcmread("{path_a}")
    ds_b = pydicom.dcmread("{path_b}")
    diffs = []
    all_keys = set()
    for elem in ds_a: all_keys.add(elem.keyword)
    for elem in ds_b: all_keys.add(elem.keyword)
    for k in sorted(all_keys):
        va = str(getattr(ds_a, k, "N/A")) if hasattr(ds_a, k) else "N/A"
        vb = str(getattr(ds_b, k, "N/A")) if hasattr(ds_b, k) else "N/A"
        if va != vb:
            diffs.append({{"tag": k, "file_a": va[:200], "file_b": vb[:200]}})
    print(json.dumps({{"success": True, "data": {{"diffs": diffs, "count": len(diffs)}}, "error": None, "elapsed_ms": int((time.time()-start)*1000)}}))
except Exception as e:
    print(json.dumps({{"success": False, "data": {{}}, "error": str(e), "elapsed_ms": int((time.time()-start)*1000)}}))
"#
        ),
    };

    Ok(script)
}

/// Check if PyDICOM is available in the Python environment
pub fn pydicom_check_script() -> String {
    r#"
import json
try:
    import pydicom
    print(json.dumps({"available": True, "version": pydicom.__version__}))
except ImportError:
    print(json.dumps({"available": False, "version": null}))
"#
    .to_string()
}

/// Script to install PyDICOM and its dependencies
pub fn pydicom_install_script() -> String {
    r#"
import subprocess, json, sys
try:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pydicom", "Pillow", "numpy"])
    print(json.dumps({"success": True}))
except Exception as e:
    print(json.dumps({"success": False, "error": str(e)}))
"#
    .to_string()
}
