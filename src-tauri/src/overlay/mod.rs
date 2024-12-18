// src-tauri/src/overlay/mod.rs
use tauri::AppHandle;
use tauri::Manager;
use windows::core::*;
mod process;
mod window;
use crate::monitor::GameMonitor;
use process::GameProcess;
use window::OverlayWindow;

pub struct GameOverlay {
    window: OverlayWindow,
    pub is_visible: bool,
    pub app: AppHandle,
    game_process: Option<GameProcess>,
}

impl GameOverlay {
    pub fn new(app: AppHandle) -> Result<Self> {
        Ok(Self {
            window: OverlayWindow::new()?,
            is_visible: false,
            app,
            game_process: None,
        })
    }

    pub fn set_target_game(&mut self, game_monitor: &GameMonitor, game_id: &str) {
        if let Some(session) = game_monitor.get_active_game() {
            if session.game_id == game_id {
                self.game_process = Some(GameProcess {
                    process_id: session.process_id,
                    window_handle: session.window_handle,
                });
            }
        }
    }

    pub fn show(&mut self) -> Result<()> {
        if !self.is_visible {
            // Si on a un jeu actif, on vérifie son état
            if let Some(game) = &self.game_process {
                // Configurer la fenêtre pour s'afficher par-dessus le jeu
                self.window.attach_to_window(game.window_handle)?;
            }

            self.window.make_transparent()?;
            self.window.set_topmost()?;
            self.window.show()?;

            if let Some(window) = self.app.get_webview_window("overlay") {
                let _ = window.eval("window.location.hash = '/overlay'");
                let _ = window.set_decorations(false);
                let _ = window.set_always_on_top(true);
                let _ = window.show();
                let _ = window.set_focus();
            }

            self.is_visible = true;
        }
        Ok(())
    }

    pub fn hide(&mut self) -> Result<()> {
        if self.is_visible {
            // Cacher la fenêtre native Windows
            self.window.hide()?;

            // Cacher la fenêtre Tauri
            if let Some(window) = self.app.get_webview_window("overlay") {
                let _ = window.hide();
            }

            self.is_visible = false;
        }
        Ok(())
    }

    pub fn toggle(&mut self) -> Result<()> {
        if self.is_visible {
            self.hide()
        } else {
            self.show()
        }
    }
}
