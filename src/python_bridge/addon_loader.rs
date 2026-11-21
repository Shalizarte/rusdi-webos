//! Loader específico para addons Python do Kodi/Rusdi.

use crate::error::{RusdiError, Result};
use crate::python_bridge::interpreter::PythonInterpreter;
use std::path::Path;

pub struct PythonAddonLoader {
    interpreter: PythonInterpreter,
    xbmc_module_initialized: bool,
}

impl PythonAddonLoader {
    pub fn new(interpreter: PythonInterpreter) -> Self {
        Self { interpreter, xbmc_module_initialized: false }
    }

    pub fn init_xbmc_environment(&mut self, addon_dir: &Path) -> Result<()> {
        self.interpreter.set_sys_path(vec![addon_dir])?;

        let bootstrap = r#"
import sys, os
class XBMCStub:
    def __getattr__(self, name): return lambda *a, **k: None
    def __call__(self, *a, **k): return None

sys.modules['xbmc'] = XBMCStub()
sys.modules['xbmcgui'] = XBMCStub()
sys.modules['xbmcplugin'] = XBMCStub()
sys.modules['xbmcvfs'] = XBMCStub()
sys.modules['xbmcaddon'] = XBMCStub()
sys.modules['xbmcplugin'] = XBMCStub()
"#;
        self.interpreter.exec_code(bootstrap)?;
        self.xbmc_module_initialized = true;
        Ok(())
    }

    pub fn run_script(&self, script_path: &Path) -> Result<()> {
        if !self.xbmc_module_initialized { return Err(RusdiError::Python("XBMC env not initialized".into())); }
        self.interpreter.exec_file(script_path)
    }
}
