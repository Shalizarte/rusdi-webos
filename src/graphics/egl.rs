//! EGL + Wayland/EGLFS context para webOS.

use crate::error::{RusdiError, Result};
use egl::{Context, Display, Surface, ConfigId, EGL_NONE, EGL_NO_CONTEXT, EGL_NO_SURFACE, EGL_NO_DISPLAY, DEFAULT_DISPLAY};
use std::ffi::CString;
use std::os::raw::c_void;

pub struct EglContext {
    display: Display, context: Context, surface: Surface, config_id: ConfigId,
    width: i32, height: i32,
}

impl EglContext {
    pub fn new_wayland(width: i32, height: i32) -> Result<Self> {
        // Tenta Wayland EGL, cai para EGLFS se falhar
        #[cfg(feature = "wayland")]
        {
            if let Ok(ctx) = Self::try_wayland(width, height) {
                return Ok(ctx);
            }
            log_warn!("Wayland EGL falhou, tentando EGLFS");
        }
        Self::new_eglfs(width, height)
    }

    #[cfg(feature = "wayland")]
    fn try_wayland(width: i32, height: i32) -> Result<Self> {
        let wl_display = unsafe { wayland_client::Display::connect(std::ptr::null())? };
        
        // Tenta GetPlatformDisplayEXT (EGL 1.5)
        let display = unsafe { 
            // GetPlatformDisplayEXT pode não estar disponível no crate egl 0.11
            // Usa GetDisplay como fallback
            egl::GetDisplay(DEFAULT_DISPLAY) 
        };
        
        if display == EGL_NO_DISPLAY { 
            return Err(RusdiError::EglInit("GetDisplay falhou".into())); 
        }
        Self::initialize(display, width, height)
    }

    pub fn new_eglfs(width: i32, height: i32) -> Result<Self> {
        let display = unsafe { egl::GetDisplay(DEFAULT_DISPLAY) };
        if display == EGL_NO_DISPLAY { return Err(RusdiError::EglInit("GetDisplay falhou".into())); }
        Self::initialize(display, width, height)
    }

    fn initialize(display: Display, width: i32, height: i32) -> Result<Self> {
        let mut major = 0; let mut minor = 0;
        if unsafe { egl::Initialize(display, &mut major, &mut minor) } == 0 {
            return Err(RusdiError::EglInit(format!("Initialize: {}", Self::egl_error())));
        }
        log_info!("EGL {}:{}", major, minor);

        let attr = [egl::RED_SIZE,8, egl::GREEN_SIZE,8, egl::BLUE_SIZE,8, egl::ALPHA_SIZE,8,
                    egl::DEPTH_SIZE,24, egl::STENCIL_SIZE,8, egl::RENDERABLE_TYPE,egl::OPENGL_ES2_BIT,
                    egl::SURFACE_TYPE,egl::WINDOW_BIT, EGL_NONE];
        let mut config_id = ConfigId(0); let mut num = 0;
        if unsafe { egl::ChooseConfig(display, attr.as_ptr(), &mut config_id, 1, &mut num) } == 0 || num == 0 {
            return Err(RusdiError::EglInit(format!("ChooseConfig: {}", Self::egl_error())));
        }

        let ctx_attr = [egl::CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
        let context = unsafe { egl::CreateContext(display, config_id, EGL_NO_CONTEXT, ctx_attr.as_ptr()) };
        if context == EGL_NO_CONTEXT { return Err(RusdiError::EglInit(format!("CreateContext: {}", Self::egl_error()))); }

        let pbuf_attr = [egl::WIDTH, 1, egl::HEIGHT, 1, EGL_NONE];
        let surface = unsafe { egl::CreatePbufferSurface(display, config_id, pbuf_attr.as_ptr()) };
        if surface == EGL_NO_SURFACE { return Err(RusdiError::EglSurface(format!("PbufferSurface: {}", Self::egl_error()))); }

        if unsafe { egl::MakeCurrent(display, surface, surface, context) } == 0 {
            return Err(RusdiError::EglInit(format!("MakeCurrent: {}", Self::egl_error())));
        }

        Ok(Self { display, context, surface, config_id, width, height })
    }

    pub fn create_window_surface(&mut self, native_window: *mut c_void) -> Result<()> {
        unsafe { egl::DestroySurface(self.display, self.surface) };
        let attr = [EGL_NONE];
        self.surface = unsafe { egl::CreateWindowSurface(self.display, self.config_id, native_window, attr.as_ptr()) };
        if self.surface == EGL_NO_SURFACE { return Err(RusdiError::EglSurface(format!("WindowSurface: {}", Self::egl_error()))); }
        if unsafe { egl::MakeCurrent(self.display, self.surface, self.surface, self.context) } == 0 {
            return Err(RusdiError::EglInit(format!("MakeCurrent(window): {}", Self::egl_error())));
        }
        Ok(())
    }

    pub fn swap_buffers(&self) -> Result<()> {
        if unsafe { egl::SwapBuffers(self.display, self.surface) } == 0 {
            return Err(RusdiError::EglInit(format!("SwapBuffers: {}", Self::egl_error())));
        }
        Ok(())
    }

    pub fn get_proc_address(&self, name: &str) -> *const c_void {
        let cname = CString::new(name).unwrap();
        unsafe { egl::GetProcAddress(cname.as_ptr()) }
    }

    fn egl_error() -> String {
        match unsafe { egl::GetError() } {
            egl::SUCCESS => "SUCCESS", egl::NOT_INITIALIZED => "NOT_INITIALIZED",
            egl::BAD_ACCESS => "BAD_ACCESS", egl::BAD_ALLOC => "BAD_ALLOC",
            egl::BAD_ATTRIBUTE => "BAD_ATTRIBUTE", egl::BAD_CONTEXT => "BAD_CONTEXT",
            egl::BAD_CONFIG => "BAD_CONFIG", egl::BAD_CURRENT_SURFACE => "BAD_CURRENT_SURFACE",
            egl::BAD_DISPLAY => "BAD_DISPLAY", egl::BAD_SURFACE => "BAD_SURFACE",
            egl::BAD_MATCH => "BAD_MATCH", egl::BAD_PARAMETER => "BAD_PARAMETER",
            egl::BAD_NATIVE_PIXMAP => "BAD_NATIVE_PIXMAP", egl::BAD_NATIVE_WINDOW => "BAD_NATIVE_WINDOW",
            egl::CONTEXT_LOST => "CONTEXT_LOST", c => return format!("UNKNOWN(0x{:x})", c),
        }.into()
    }
}

impl Drop for EglContext {
    fn drop(&mut self) {
        unsafe {
            egl::MakeCurrent(self.display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
            egl::DestroySurface(self.display, self.surface);
            egl::DestroyContext(self.display, self.context);
            egl::Terminate(self.display);
        }
    }
}
