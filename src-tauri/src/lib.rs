use std::env;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tokio::time::Duration;

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

    if let Err(e) = Logger::init() {
        eprintln!("Failed to initialize logger: {}", e);
    }

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
            // Charger les variables d'environnement à partir d'un fichier .env uniquement en mode développement
            dotenv().ok();
            log_info!("Environment variables loaded from .env file");


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

            log_debug!("Setting up system tray");

            // Initialisation de la base de données et des composants
            log_info!("Initializing database...");
            let database = match rt.block_on(async { Database::new().await }) {
                Ok(db) => {
                    log_info!("Database initialized successfully");
                    Arc::new(db)
                }
                Err(e) => {
                    log_error!("Failed to initialize database: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            };

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
            let igdb_client_id = env::var("IGDB_CLIENT_ID").unwrap_or_else(|_| {
                log_warn!("IGDB_CLIENT_ID not found in environment variables");
                String::new()
            });
            let igdb_client_secret = env::var("IGDB_CLIENT_SECRET").unwrap_or_else(|_| {
                log_warn!("IGDB_CLIENT_SECRET not found in environment variables");
                String::new()
            });

            // GameMonitor setup
            log_info!("Setting up game monitor...");
            let game_monitor = Arc::new(GameMonitor::new(database.clone()));
            let monitor_clone = game_monitor.clone();

            rt.spawn(async move {
                log_info!("Starting game monitor loop");
                // On lance simplement le monitoring et on laisse la fonction gérer sa propre boucle
                monitor_clone.start_monitoring();
                // Plus besoin de gérer la boucle ici car elle est dans start_monitoring
            });

            // GameManager setup
            log_info!("Setting up game manager...");
            let game_manager = GameManager::new(
                database.clone(),
                app.handle().clone(),
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
            let overlay = GameOverlay::new(app.handle().clone()).expect("Failed to create overlay");
            let overlay_state = OverlayState {
                overlay: Arc::new(Mutex::new(Some(overlay))),
            };
            app.manage(overlay_state);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Enlevez .event()
                let app_handle = window.app_handle();

                // Vérifier les paramètres sans async/await
                let settings = SettingsManager::new().unwrap();
                if settings.get_settings().minimize_to_tray {
                    log_info!("Minimizing to tray instead of closing");
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
                log_info!("Application exit requested");
                api.prevent_exit();
            }
        });
}
