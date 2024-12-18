use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use std::env;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent, Emitter,
};
use tauri_plugin_deep_link::DeepLinkExt;

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
use auth::AuthServer;
use db::Database;
use dotenv::dotenv;
use game_manager::GameManager;
use monitor::GameMonitor;
use overlay::GameOverlay;
use utils::settings::SettingsManager;
use utils::AppError;

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
            dotenv().ok();

            // Configuration du tray
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

            // Initialisation de la base de données et des composants
            let database = rt.block_on(async { Database::new().await.map(Arc::new) })?;

            // Auth server setup
            let auth_server = AuthServer::new(11111);
            std::thread::spawn(move || {
                if let Err(e) = auth_server.start() {
                    eprintln!("Error starting auth server: {}", e);
                }
            });

            // Setup API keys
            let igdb_client_id = env::var("IGDB_CLIENT_ID").unwrap_or_else(|_| String::new());
            let igdb_client_secret = env::var("IGDB_CLIENT_SECRET").unwrap_or_else(|_| String::new());

            // GameMonitor setup
            let game_monitor = Arc::new(GameMonitor::new(database.clone()));
            let monitor_clone = game_monitor.clone();

            rt.spawn(async move {
                loop {
                    monitor_clone.start_monitoring();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            // GameManager setup
            let game_manager = Arc::new(GameManager::new(
                database.clone(),
                app.handle().clone(),
                game_monitor.clone(),
                igdb_client_id,
                igdb_client_secret,
            ).map_err(|e| e.to_string())?);

            // State management
            let state = AppState::new(game_manager, game_monitor.clone());
            app.manage(state);

            // Overlay setup
            let overlay = GameOverlay::new(app.handle().clone())
                .expect("Failed to create overlay");
            let overlay_state = OverlayState {
                overlay: Arc::new(Mutex::new(Some(overlay))),
            };
            app.manage(overlay_state);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {  // Enlevez .event()
                let app_handle = window.app_handle();

                // Vérifier les paramètres sans async/await
                let settings = SettingsManager::new().unwrap();
                if settings.get_settings().minimize_to_tray {
                    window.hide().unwrap();
                    api.prevent_close();
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
                api.prevent_exit();
            }
        });
}