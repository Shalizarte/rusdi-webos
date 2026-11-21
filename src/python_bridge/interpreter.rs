//! Embedded Python 3.11 interpreter para compatibilidade addons.

use crate::error::{RusdiError, Result};
use pyo3::prelude::*;
use std::path::Path;

pub struct PythonInterpreter {}

impl PythonInterpreter {
    pub fn initialize() -> Result<Self> {
        pyo3::prepare_freethreaded_python();
        Ok(Self {})
    }

    pub fn exec_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Python::with_gil(|py| {
            let code = std::fs::read_to_string(path)?;
            py.run(&code, None, None)?;
            Ok(())
        })
    }

    pub fn exec_code(&self, code: &str) -> Result<()> {
        Python::with_gil(|py| { py.run(code, None, None)?; Ok(()) })
    }

    pub fn call_function(&self, module: &str, func: &str, args: Vec<&str>) -> Result<PyObject> {
        Python::with_gil(|py| {
            let mod_ = py.import(module)?;
            let func = mod_.getattr(func)?;
            let result = func.call1(args)?;
            Ok(result.into())
        })
    }

    pub fn set_sys_path(&self, paths: Vec<&Path>) -> Result<()> {
        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let path_list = sys.getattr("path")?;
            for p in paths {
                path_list.call_method1("append", (p.to_string_lossy().to_string(),))?;
            }
            Ok(())
        })
    }
}
