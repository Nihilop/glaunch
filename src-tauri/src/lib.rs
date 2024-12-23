use std::env;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tokio::time::Duration;
use crate::utils::secrets::SecretsManager;
use crate::utils::AppPaths;
// Modules internes
mod api;
mod auth;
mod commands;
mod db;
mod game_manager;
mod models;
mod monitor;
mod overlay;
mod platforms;
mod services;
mod utils;

// Imports essentiels
use crate::utils::Logger;
use auth::AuthServer;
use db::Database;
use game_manager::GameManager;
use monitor::GameMonitor;
use overlay::GameOverlay;
use utils::settings::SettingsManager;
use utils::AppError;
use windows::Win32::Foundation::HWND;

// États de l'application
pub struct AppState {
    game_manager: Arc<GameManager>,
    game_monitor: Arc<GameMonitor>,
}

impl AppState {
    pub fn new(game_manager: Arc<GameManager>, game_monitor: Arc<GameMonitor>) -> Self {
        Self {
            game_manager,
            game_monitor,
        }
    }
}

pub struct OverlayState {
    pub overlay: Arc<Mutex<Option<GameOverlay>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
    let rt = Arc::new(rt);

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init());

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
        if let Some(url) = argv.iter().find(|arg| arg.starts_with("glaunch://")) {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("deep-link", url.to_string());
            }
        }
    }));

    builder
        .setup(move |app| {
            let app_handle = app.handle();

            // Initialize paths first
            let paths = AppPaths::new(&app_handle)?;

            // Secrets manager setup
            let secrets_manager = SecretsManager::new("glaunch");

            if let Err(e) = Logger::init(&app_handle) {
                eprintln!("Failed to initialize logger: {}", e);
            } else {
                log_info!("Logger initialized successfully");  // Ajouter ce log pour vérifier
            }

            // Validate required secrets
            if let Err(e) = secrets_manager.validate_required_secrets() {
                log_error!("Failed to validate secrets: {}", e);
            }

            // Debug environment variables
            for (key, _) in env::vars() {
                if key.contains("IGDB") || key.contains("STEAM") || key.contains("EPIC") || key.contains("BATTLENET") {
                    log_debug!("After loading env file - Found env var: {}", key);
                }
            }

            // Tray configuration
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let separator = MenuItem::new(app, "", false, None::<&str>)?;

            let menu = Menu::with_items(app, &[&open, &settings, &separator, &quit])?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => std::process::exit(0),
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let settings = SettingsManager::new(&app).unwrap();
                            let _ = window.eval("window.location.hash = '/settings'");
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            log_debug!("Setting up system tray");

            // Initialize database
            log_info!("Initializing database...");
            let database = rt.block_on(async {
                match Database::new(&app_handle).await {
                    Ok(db) => {
                        log_info!("Database initialized successfully");
                        Ok(Arc::new(db))
                    }
                    Err(e) => {
                        log_error!("Failed to initialize database: {}", e);
                        Err(Box::new(e) as Box<dyn std::error::Error>)
                    }
                }
            })?;

            // Auth server setup
            log_info!("Setting up auth server...");
            let auth_server = AuthServer::new(11111);
            std::thread::spawn(move || {
                if let Err(e) = auth_server.start() {
                    log_error!("Error starting auth server: {}", e);
                }
            });

            // Setup API keys
            log_info!("Loading API keys...");
            let igdb_client_id = secrets_manager.get_compiled_secret("IGDB_CLIENT_ID")?;
            let igdb_client_secret = secrets_manager.get_compiled_secret("IGDB_CLIENT_SECRET")?;

            // GameMonitor setup
            log_info!("Setting up game monitor...");
            let game_monitor = Arc::new(GameMonitor::new(database.clone()));
            let monitor_clone = game_monitor.clone();

            rt.spawn(async move {
                log_info!("Starting game monitor loop");
                monitor_clone.start_monitoring();
            });

            // GameManager setup
            log_info!("Setting up game manager...");
            let game_manager = GameManager::new(
                database.clone(),
                app_handle.clone(),
                game_monitor.clone(),
                igdb_client_id,
                igdb_client_secret,
            ).map_err(|e| {
                log_error!("Failed to create game manager: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            }).map(Arc::new)?;
            log_info!("Game manager initialized successfully");

            // State management
            let state = AppState::new(game_manager, game_monitor.clone());
            app.manage(state);

            // Overlay setup
            let overlay = GameOverlay::new(app_handle.clone()).expect("Failed to create overlay");
            let overlay_state = OverlayState {
                overlay: Arc::new(Mutex::new(Some(overlay))),
            };
            app.manage(overlay_state);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app_handle = window.app_handle();

                // Convertir correctement le HWND
                let raw_hwnd = window.hwnd().unwrap().0 as isize;
                let hwnd = HWND(raw_hwnd);

                // Vérifier les paramètres
                match SettingsManager::new(&app_handle) {
                    Ok(settings) => {
                        if settings.should_minimize_to_tray(hwnd) {
                            // Cacher la fenêtre uniquement
                            log_info!("Minimizing to tray instead of closing");
                            let _ = window.hide();
                            api.prevent_close();
                        } else {
                            // Réellement quitter l'application
                            log_info!("Closing application completely");
                            std::process::exit(0); // Force la fermeture complète
                        }
                    }
                    Err(e) => {
                        log_error!("Failed to load settings: {}", e);
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_games,
            commands::launch_game,
            commands::update_game_metadata,
            commands::get_game,
            commands::get_active_game,
            commands::search_igdb_games,
            commands::update_game_with_igdb,
            commands::toggle_overlay,
            commands::add_custom_game,
            commands::delete_game,
            // Auth commands
            commands::auth_steam,
            commands::auth_steam_callback,
            commands::auth_epic,
            commands::auth_epic_callback,
            commands::exchange_epic_code,
            commands::auth_battlenet,
            commands::auth_battlenet_callback,
            // API commands
            commands::get_steam_profile,
            commands::get_steam_friends,
            commands::get_epic_profile,
            commands::get_epic_friends,
            commands::get_battlenet_profile,
            commands::get_battlenet_friends,
            // Settings commands
            commands::get_app_settings,
            commands::save_app_settings,
            commands::export_db,
            commands::import_db,
            commands::check_for_updates,
            commands::install_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                log_info!("Application exit requested");
                api.prevent_exit();
            }
        });
}
