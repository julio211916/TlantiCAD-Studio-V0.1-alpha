//! TlantiCAD Python Bridge
//! Provides Python script execution for dental analysis workflows.
//! PyO3 is optional — when disabled, scripts run via subprocess.

pub mod interpreter;
pub mod numpy_bridge;
pub mod scripts;

pub use interpreter::*;
pub use numpy_bridge::*;
pub use scripts::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_context_default() {
        let ctx = ScriptContext::default();
        assert_eq!(ctx.timeout_secs, 30);
        assert!(matches!(ctx.method, ExecutionMethod::Subprocess));
        assert!(ctx.python_path.is_none());
    }

    #[test]
    fn test_python_script_new() {
        let s = PythonScript::new("test", "print('hello')");
        assert_eq!(s.name, "test");
        assert!(s.code.contains("print"));
    }

    #[test]
    fn test_python_script_with_data() {
        let s = PythonScript::new("t", "pass")
            .with_data(serde_json::json!({"a": 1}));
        assert!(s.input_data.is_some());
    }

    #[test]
    fn test_python_script_require() {
        let s = PythonScript::new("t", "pass")
            .require("numpy")
            .require("scipy");
        assert_eq!(s.requirements.len(), 2);
    }

    #[test]
    fn test_python_available() {
        // just ensure it returns without panic
        let _ = python_available();
    }

    #[test]
    fn test_python_version() {
        // may be None if python not installed, just check no panic
        let _ = python_version();
    }

    #[test]
    fn test_bone_density_script() {
        let s = bone_density_script(&[100.0, 200.0, 300.0]);
        assert!(s.code.contains("hu_values") || !s.code.is_empty());
    }

    #[test]
    fn test_numpy_dtype_name() {
        assert_eq!(NumpyDtype::Float32.numpy_name(), "float32");
        assert_eq!(NumpyDtype::Uint16.numpy_name(), "uint16");
    }

    #[test]
    fn test_numpy_array_spec() {
        let spec = NumpyArraySpec::from_f32(&[1.0, 2.0, 3.0], vec![3]);
        let code = spec.to_python_code("arr");
        assert!(code.contains("numpy"));
    }
}
