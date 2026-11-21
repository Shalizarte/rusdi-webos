//! Compilação e gerenciamento de shaders GLES 2.0.

use crate::error::{RusdiError, Result};
use gl::types::*;
use std::ffi::CString;

pub struct ShaderProgram {
    pub id: GLuint,
}

impl ShaderProgram {
    pub fn from_source(vert_src: &str, frag_src: &str) -> Result<Self> {
        let vert = Self::compile(GL_VERTEX_SHADER, vert_src)?;
        let frag = Self::compile(GL_FRAGMENT_SHADER, frag_src)?;
        let prog = unsafe { gl::CreateProgram() };
        unsafe { gl::AttachShader(prog, vert); gl::AttachShader(prog, frag); gl::LinkProgram(prog); }
        let mut success = 0;
        unsafe { gl::GetProgramiv(prog, GL_LINK_STATUS, &mut success); }
        if success == 0 {
            let mut len = 0; unsafe { gl::GetProgramiv(prog, GL_INFO_LOG_LENGTH, &mut len); }
            let mut buf = vec![0u8; len as usize];
            unsafe { gl::GetProgramInfoLog(prog, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut i8); }
            return Err(RusdiError::Config(String::from_utf8_lossy(&buf).into()));
        }
        unsafe { gl::DeleteShader(vert); gl::DeleteShader(frag); }
        Ok(Self { id: prog })
    }

    fn compile(shader_type: GLenum, src: &str) -> Result<GLuint> {
        let shader = unsafe { gl::CreateShader(shader_type) };
        let c_src = CString::new(src).unwrap();
        unsafe { gl::ShaderSource(shader, 1, &c_src.as_ptr(), std::ptr::null()); gl::CompileShader(shader); }
        let mut success = 0;
        unsafe { gl::GetShaderiv(shader, GL_COMPILE_STATUS, &mut success); }
        if success == 0 {
            let mut len = 0; unsafe { gl::GetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut len); }
            let mut buf = vec![0u8; len as usize];
            unsafe { gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), buf.as_mut_ptr() as *mut i8); }
            return Err(RusdiError::Config(format!("Shader compile: {}", String::from_utf8_lossy(&buf))));
        }
        Ok(shader)
    }

    pub fn use_program(&self) { unsafe { gl::UseProgram(self.id); } }
    pub fn uniform_1i(&self, name: &str, v: i32) {
        let c = CString::new(name).unwrap();
        unsafe { gl::Uniform1i(gl::GetUniformLocation(self.id, c.as_ptr()), v); }
    }
    pub fn uniform_matrix4fv(&self, name: &str, mat: &[f32; 16]) {
        let c = CString::new(name).unwrap();
        unsafe { gl::UniformMatrix4fv(gl::GetUniformLocation(self.id, c.as_ptr()), 1, gl::FALSE, mat.as_ptr()); }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) { unsafe { gl::DeleteProgram(self.id); } }
}

// Shaders built-in (Estuary-style)
pub const VERT_DEFAULT: &str = r#"
attribute vec4 a_position; attribute vec2 a_texcoord;
uniform mat4 u_mvp; varying vec2 v_texcoord;
void main() { v_texcoord = a_texcoord; gl_Position = u_mvp * a_position; }
"#;

pub const FRAG_TEXTURE: &str = r#"
precision mediump float; varying vec2 v_texcoord; uniform sampler2D u_texture;
void main() { gl_FragColor = texture2D(u_texture, v_texcoord); }
"#;

pub const FRAG_YUV: &str = r#"
precision mediump float; varying vec2 v_texcoord;
uniform sampler2D u_y; uniform sampler2D u_u; uniform sampler2D u_v;
void main() {
    float y = texture2D(u_y, v_texcoord).r;
    float u = texture2D(u_u, v_texcoord).r - 0.5;
    float v = texture2D(u_v, v_texcoord).r - 0.5;
    float r = y + 1.402 * v;
    float g = y - 0.344 * u - 0.714 * v;
    float b = y + 1.772 * u;
    gl_FragColor = vec4(r, g, b, 1.0);
}
"#;
