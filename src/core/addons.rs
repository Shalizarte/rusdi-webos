//! Addon Manager - suporta addons Rust nativos e Python.

use crate::error::{RusdiError, Result};
use crate::python_bridge::interpreter::PythonInterpreter;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonManifest {
    pub id: String, pub name: String, pub version: String,
    pub type_: String, pub extension_point: String,
    pub requires: Vec<String>, pub provides: Vec<String>,
}

pub enum AddonType {
    Script, Plugin, Skin, Codec, Pvr, Language, Other,
}

pub struct AddonManager {
    addons: HashMap<String, Box<dyn Addon>>,
    python: Option<PythonInterpreter>,
    addon_paths: Vec<PathBuf>,
}

impl AddonManager {
    pub fn new() -> Result<Self> {
        let python = PythonInterpreter::initialize()?;
        Ok(Self { addons: HashMap::new(), python: Some(python), addon_paths: vec![] })
    }

    pub fn add_path(&mut self, path: PathBuf) { self.addon_paths.push(path); }

    pub fn scan(&mut self) -> Result<Vec<AddonManifest>> {
        let mut manifests = Vec::new();
        for path in &self.addon_paths.clone() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let addon_dir = entry.path();
                    let addon_xml = addon_dir.join("addon.xml");
                    if addon_xml.exists() {
                        let content = std::fs::read_to_string(&addon_xml)?;
                        if let Ok(manifest) = quick_xml::de::from_str::<AddonManifest>(&content) {
                            manifests.push(manifest);
                        }
                    }
                }
            }
        }
        Ok(manifests)
    }

    pub fn load_addon(&mut self, manifest: &AddonManifest) -> Result<()> {
        match manifest.type_.as_str() {
            "xbmc.python.script" | "xbmc.python.pluginsource" => {
                self.load_python_addon(manifest)?;
            }
            "xbmc.addon.executable" => {
                self.load_native_addon(manifest)?;
            }
            _ => { log_warn!("Tipo de addon não suportado: {}", manifest.type_); }
        }
        Ok(())
    }

    fn load_python_addon(&mut self, manifest: &AddonManifest) -> Result<()> {
        if let Some(py) = &mut self.python {
            let addon_path = self.addon_paths.iter()
                .find(|p| p.join(&manifest.id).exists())
                .ok_or(RusdiError::NotFound(format!("Addon path for {}", manifest.id)))?
                .join(&manifest.id);
            py.exec_file(addon_path.join("default.py"))?;
            self.addons.insert(manifest.id.clone(), Box::new(PythonAddon { id: manifest.id.clone() }));
        }
        Ok(())
    }

    fn load_native_addon(&mut self, _manifest: &AddonManifest) -> Result<()> {
        Ok(())
    }
}

pub trait Addon: Send + Sync {
    fn id(&self) -> &str;
    fn on_start(&mut self) -> Result<()> { Ok(()) }
    fn on_stop(&mut self) -> Result<()> { Ok(()) }
}

struct PythonAddon { id: String }
impl Addon for PythonAddon {
    fn id(&self) -> &str { &self.id }
}
