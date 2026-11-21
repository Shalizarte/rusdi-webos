//! nativeLifeCycleInterfaceVersion 2 para webOS.

use crate::error::{RusdiError, Result};
use crate::graphics::renderer::Renderer;
use crate::input::remote::InputHandler;
use crate::core::{player::MediaPlayer, library::Library, skin::Skin, addons::AddonManager, pvr::PvrManager};
use crate::settings::Settings;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Starting, Running, Suspended, Resuming, Terminating, Terminated,
}

pub struct LifeCycleManager {
    state: AppState, settings: Settings,
    renderer: Option<Renderer>,
    player: Option<MediaPlayer>,
    library: Option<Library>,
    skin: Option<Skin>,
    addons: Option<AddonManager>,
    pvr: Option<PvrManager>,
    input_handler: Option<InputHandler>,
    shutdown_tx: watch::Sender<bool>,
}

impl LifeCycleManager {
    pub fn new(settings: Settings) -> Result<Self> {
        let (shutdown_tx, _) = watch::channel(false);
        let renderer = Renderer::new(1920, 1080, "Rusdi webOS")?;
        Ok(Self { state: AppState::Starting, settings, renderer: Some(renderer),
                  player: None, library: None, skin: None, addons: None, pvr: None,
                  input_handler: None, shutdown_tx })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        log_info!("=== Inicializando Rusdi webOS ===");

        if let Some(r) = &mut self.renderer {
            let splash = std::path::Path::new("/media/developer/apps/usr/palm/applications/org.xbmc.kodi/media/splash.jpg");
            if splash.exists() { r.render_splash(splash)?; }
        }

        self.library = Some(Library::new("/media/developer/apps/usr/palm/applications/org.xbmc.kodi/userdata/Database/Rusdi21.db").await?);
        self.addons = Some(AddonManager::new()?);
        self.addons.as_mut().unwrap().add_path(std::path::PathBuf::from("/media/developer/apps/usr/palm/applications/org.xbmc.kodi/addons"));
        let manifests = self.addons.as_mut().unwrap().scan()?;
        for m in manifests { self.addons.as_mut().unwrap().load_addon(&m)?; }

        self.skin = Some(Skin::load(std::path::Path::new("/media/developer/apps/usr/palm/applications/org.xbmc.kodi/system/skins/estuary"), self.renderer.as_mut().unwrap())?);
        self.pvr = Some(PvrManager::new()); self.pvr.as_mut().unwrap().register(Box::new(crate::core::pvr::IptvSimpleBackend::new()));

        let (handler, rx) = InputHandler::new(|action| {
            log_debug!("Input: {:?}", action);
        });
        self.input_handler = Some(handler);

        let input_clone = self.input_handler.as_ref().unwrap().clone();
        std::thread::spawn(move || { let _ = input_clone.run_sdl_loop(); });

        let shutdown_rx = self.shutdown_tx.subscribe();
        tokio::spawn(async move { /* process events */ });

        self.state = AppState::Running;
        log_info!("Rusdi pronto");
        Ok(())
    }

    pub async fn suspend(&mut self) -> Result<()> {
        if self.state == AppState::Running { self.state = AppState::Suspended; }
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        if self.state == AppState::Suspended { self.state = AppState::Resuming; self.state = AppState::Running; }
        Ok(())
    }

    pub async fn terminate(&mut self) -> Result<()> {
        self.state = AppState::Terminating;
        let _ = self.shutdown_tx.send(true);
        self.player.as_mut().map(|p| p.stop());
        self.addons.as_mut().map(|a| { for (_, addon) in &mut a.addons { let _ = addon.on_stop(); } });
        self.state = AppState::Terminated;
        log_info!("Rusdi terminado");
        Ok(())
    }

    pub async fn setup_signal_handlers(&self) -> Result<()> {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
        let shutdown_tx = self.shutdown_tx.clone();
        tokio::spawn(async move {
            tokio::select! { _ = sigterm.recv() => { let _ = shutdown_tx.send(true); } _ = sigint.recv() => { let _ = shutdown_tx.send(true); } }
        });
        Ok(())
    }

    pub fn state(&self) -> AppState { self.state }
}
