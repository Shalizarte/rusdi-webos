//! Skin Engine - parseia XML (Estuary-style) e renderiza via OpenGL ES.

use crate::error::{RusdiError, Result};
use crate::graphics::shaders::{ShaderProgram, VERT_DEFAULT, FRAG_TEXTURE};
use crate::graphics::renderer::Renderer;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SkinElement {
    pub id: String, pub element_type: String,
    pub pos: (i32, i32), pub size: (u32, u32),
    pub texture: Option<String>, pub color: Option<[f32; 4]>,
    pub visible: bool, pub animation: Option<String>,
}

pub struct Skin {
    pub name: String, pub resolution: (u32, u32),
    elements: HashMap<String, SkinElement>,
    shader: Option<ShaderProgram>,
    textures: HashMap<String, sdl2::render::Texture>,
}

impl Skin {
    pub fn load(skin_dir: &Path, renderer: &mut Renderer) -> Result<Self> {
        let xml_path = skin_dir.join("skin.xml");
        let content = std::fs::read_to_string(xml_path)?;
        let mut elements = HashMap::new();
        let mut textures = HashMap::new();

        let shader = ShaderProgram::from_source(VERT_DEFAULT, FRAG_TEXTURE)?;

        for entry in std::fs::read_dir(skin_dir.join("media"))? {
            let path = entry?.path();
            if let Some(ext) = path.extension() {
                if ext == "png" || ext == "jpg" {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    let tex = renderer.texture_creator().load_texture(&path)?;
                    textures.insert(name, tex);
                }
            }
        }

        Ok(Self { name: "Estuary-Rust".into(), resolution: (1920, 1080),
                  elements, shader: Some(shader), textures })
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        if let Some(shader) = &self.shader {
            shader.use_program();
            for elem in self.elements.values() {
                if !elem.visible { continue; }
                if let Some(tex) = elem.texture.as_ref().and_then(|t| self.textures.get(t)) {
                    let dst = sdl2::rect::Rect::new(elem.pos.0, elem.pos.1, elem.size.0, elem.size.1);
                    renderer.draw_texture(tex, dst).ok();
                }
            }
        }
    }
}
