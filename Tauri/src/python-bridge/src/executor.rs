//! Python code executor

#[cfg(feature = "pyo3")]
mod pyo3_executor {
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList, PyModule, PyTuple};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::path::Path;
    use tracing::{info, warn};

    use crate::{PythonError, Result};

    /// Python executor for running code and scripts
    pub struct PythonExecutor {
        initialized: bool,
    }

    impl PythonExecutor {
        pub fn new() -> Result<Self> {
            // PyO3 auto-initializes Python
            Ok(Self { initialized: true })
        }

        /// Execute Python code and return result as JSON string
        pub fn execute_code(&self, code: &str) -> Result<String> {
            Python::with_gil(|py| {
                let locals = PyDict::new_bound(py);

                // Execute the code
                py.run_bound(code, None, Some(&locals))?;

                // Try to get 'result' variable
                if let Ok(result) = locals.get_item("result") {
                    if let Some(r) = result {
                        // Convert to JSON-serializable form
                        let json_module = py.import_bound("json")?;
                        let json_str: String = json_module
                            .call_method1("dumps", (r,))?
                            .extract()?;
                        return Ok(json_str);
                    }
                }

                Ok("null".to_string())
            })
        }

        /// Execute a Python script file
        pub fn execute_script(&self, script_path: &Path, args: Option<&str>) -> Result<String> {
            let code = std::fs::read_to_string(script_path)?;

            Python::with_gil(|py| {
                let locals = PyDict::new_bound(py);

                // Set script arguments if provided
                if let Some(args_json) = args {
                    let json_module = py.import_bound("json")?;
                    let args_obj = json_module.call_method1("loads", (args_json,))?;
                    locals.set_item("args", args_obj)?;
                }

                // Set __file__ for the script
                locals.set_item("__file__", script_path.to_string_lossy().as_ref())?;

                // Execute the script
                py.run_bound(&code, None, Some(&locals))?;

                // Get result
                if let Ok(result) = locals.get_item("result") {
                    if let Some(r) = result {
                        let json_module = py.import_bound("json")?;
                        let json_str: String = json_module
                            .call_method1("dumps", (r,))?
                            .extract()?;
                        return Ok(json_str);
                    }
                }

                Ok("null".to_string())
            })
        }

        /// Call a Python function from a module
        pub fn call_function(
            &self,
            module_name: &str,
            function_name: &str,
            args: Vec<serde_json::Value>,
        ) -> Result<serde_json::Value> {
            Python::with_gil(|py| {
                let module = py.import_bound(module_name)?;
                let function = module.getattr(function_name)?;

                // Convert args to Python objects
                let py_args: Vec<PyObject> = args
                    .iter()
                    .map(|v| json_value_to_py(py, v))
                    .collect::<Result<_>>()?;

                let result = function.call1(PyTuple::new_bound(py, py_args))?;

                // Convert result back to JSON
                let json_module = py.import_bound("json")?;
                let json_str: String = json_module
                    .call_method1("dumps", (result,))?
                    .extract()?;

                serde_json::from_str(&json_str)
                    .map_err(|e| PythonError::SerializationError(e.to_string()))
            })
        }

        /// Import and initialize a Python module
        pub fn import_module(&self, module_name: &str) -> Result<()> {
            Python::with_gil(|py| {
                py.import_bound(module_name)?;
                info!("Imported Python module: {}", module_name);
                Ok(())
            })
        }

        /// Check if a Python package is available
        pub fn check_package(&self, package_name: &str) -> bool {
            Python::with_gil(|py| py.import_bound(package_name).is_ok())
        }

        /// Install a Python package using pip
        pub fn install_package(&self, package_name: &str) -> Result<()> {
            Python::with_gil(|py| {
                let subprocess = py.import_bound("subprocess")?;
                let sys = py.import_bound("sys")?;
                let executable: String = sys.getattr("executable")?.extract()?;

                subprocess.call_method1(
                    "check_call",
                    ([
                        executable,
                        "-m".to_string(),
                        "pip".to_string(),
                        "install".to_string(),
                        package_name.to_string(),
                    ],),
                )?;

                info!("Installed Python package: {}", package_name);
                Ok(())
            })
        }
    }

    impl Default for PythonExecutor {
        fn default() -> Self {
            Self::new().expect("Failed to initialize Python executor")
        }
    }

    /// Convert JSON value to Python object
    fn json_value_to_py(py: Python, value: &serde_json::Value) -> Result<PyObject> {
        use serde_json::Value;
        use pyo3::ToPyObject;

        Ok(match value {
            Value::Null => py.None(),
            Value::Bool(b) => b.to_object(py),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i.to_object(py)
                } else if let Some(f) = n.as_f64() {
                    f.to_object(py)
                } else {
                    return Err(PythonError::SerializationError("Invalid number".to_string()));
                }
            }
            Value::String(s) => s.to_object(py),
            Value::Array(arr) => {
                let list = PyList::empty_bound(py);
                for item in arr {
                    list.append(json_value_to_py(py, item)?)?;
                }
                list.into()
            }
            Value::Object(obj) => {
                let dict = PyDict::new_bound(py);
                for (k, v) in obj {
                    dict.set_item(k, json_value_to_py(py, v)?)?;
                }
                dict.into()
            }
        })
    }

    pub use PythonExecutor;
}

#[cfg(feature = "pyo3")]
pub use pyo3_executor::PythonExecutor;

#[cfg(not(feature = "pyo3"))]
pub struct PythonExecutor {
    #[allow(dead_code)]
    initialized: bool,
}

#[cfg(not(feature = "pyo3"))]
impl PythonExecutor {
    pub fn new() -> crate::Result<Self> {
        Ok(Self { initialized: false })
    }

    pub fn execute_code(&self, _code: &str) -> crate::Result<String> {
        Err(crate::PythonError::InterpreterNotFound)
    }

    pub fn execute_script(&self, _script_path: &std::path::Path, _args: Option<&str>) -> crate::Result<String> {
        Err(crate::PythonError::InterpreterNotFound)
    }

    pub fn call_function(
        &self,
        _module_name: &str,
        _function_name: &str,
        _args: Vec<serde_json::Value>,
    ) -> crate::Result<serde_json::Value> {
        Err(crate::PythonError::InterpreterNotFound)
    }

    pub fn import_module(&self, _module_name: &str) -> crate::Result<()> {
        Err(crate::PythonError::InterpreterNotFound)
    }

    pub fn check_package(&self, _package_name: &str) -> bool {
        false
    }

    pub fn install_package(&self, _package_name: &str) -> crate::Result<()> {
        Err(crate::PythonError::InterpreterNotFound)
    }
}

#[cfg(not(feature = "pyo3"))]
impl Default for PythonExecutor {
    fn default() -> Self {
        Self { initialized: false }
    }
}
