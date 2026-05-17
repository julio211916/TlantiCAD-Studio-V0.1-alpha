//! NumPy array ↔ ndarray conversion utilities

use serde::{Deserialize, Serialize};

/// NumPy dtype identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumpyDtype {
    Float32,
    Float64,
    Int32,
    Int64,
    Uint8,
    Uint16,
}

impl NumpyDtype {
    pub fn numpy_name(&self) -> &'static str {
        match self {
            NumpyDtype::Float32 => "float32",
            NumpyDtype::Float64 => "float64",
            NumpyDtype::Int32   => "int32",
            NumpyDtype::Int64   => "int64",
            NumpyDtype::Uint8   => "uint8",
            NumpyDtype::Uint16  => "uint16",
        }
    }
}

/// Serializable representation of a numpy array for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumpyArraySpec {
    pub shape: Vec<usize>,
    pub dtype: NumpyDtype,
    pub data_base64: String,
}

impl NumpyArraySpec {
    /// Create spec from f32 slice
    pub fn from_f32(data: &[f32], shape: Vec<usize>) -> Self {
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
        Self {
            shape,
            dtype: NumpyDtype::Float32,
            data_base64: base64_encode(&bytes),
        }
    }

    /// Create spec from u16 (DICOM pixel data)
    pub fn from_u16(data: &[u16], shape: Vec<usize>) -> Self {
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
        Self {
            shape,
            dtype: NumpyDtype::Uint16,
            data_base64: base64_encode(&bytes),
        }
    }

    /// Generate Python code to reconstruct this array
    pub fn to_python_code(&self, var_name: &str) -> String {
        format!(
            "import numpy as np, base64\n{} = np.frombuffer(base64.b64decode('{}'), dtype=np.{}).reshape({:?})",
            var_name,
            self.data_base64,
            self.dtype.numpy_name(),
            self.shape
        )
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(CHARS[(b0 >> 2) & 0x3f] as char);
        out.push(CHARS[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 { out.push(CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(CHARS[b2 & 0x3f] as char); } else { out.push('='); }
    }
    out
}
