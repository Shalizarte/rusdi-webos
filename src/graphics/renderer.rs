//! Renderer SDL2 + OpenGL ES para UI e splash.

use crate::error::{RusdiError, Result};
use sdl2::video::Window;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::rect::Rect;
use std::path::Path;

pub struct Renderer {
    canvas: Canvas<Window>, texture_creator: TextureCreator<Window>,
    width: u32, height: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self> {
        let sdl = sdl2::init()?; let video = sdl.video()?;
        video.gl_attr().set_context_profile(sdl2::video::GLProfile::GLES);
        video.gl_attr().set_context_version(2, 0);
        video.gl_attr().set_red_size(8); video.gl_attr().set_green_size(8);
        video.gl_attr().set_blue_size(8); video.gl_attr().set_alpha_size(8);
        video.gl_attr().set_depth_size(24); video.gl_attr().set_stencil_size(8);
        video.gl_attr().set_multisample_buffers(1); video.gl_attr().set_multisample_samples(4);

        let window = video.window(title, width, height).opengl().resizable().build()
            .map_err(|e| RusdiError::Sdl2(e))?;
        let canvas = window.into_canvas().accelerated().target_texture().build()
            .map_err(|e| RusdiError::Sdl2(e))?;
        let texture_creator = canvas.texture_creator();

        Ok(Self { canvas, texture_creator, width, height })
    }

    pub fn render_splash(&mut self, image_path: &Path) -> Result<()> {
        use sdl2::image::{LoadTexture, InitFlag};
        sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
        let texture = self.texture_creator.load_texture(image_path)?;
        self.canvas.clear(); self.canvas.copy(&texture, None, None)?; self.canvas.present();
        Ok(())
    }

    pub fn begin_frame(&mut self) { self.canvas.clear(); }
    pub fn end_frame(&mut self) { self.canvas.present(); }
    pub fn draw_texture(&mut self, texture: &sdl2::render::Texture, dst: Rect) -> Result<()> {
        self.canvas.copy(texture, None, dst)?; Ok(())
    }
    pub fn texture_creator(&self) -> &TextureCreator<Window> { &self.texture_creator }
    pub fn set_draw_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.canvas.set_draw_color(sdl2::pixels::Color::RGBA(r, g, b, a));
    }
}
