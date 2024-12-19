// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::env;
fn main() {

    let is_dev = std::env::var("TAURI_ENV").unwrap_or_default() == "dev";

    // En dev, utiliser dotenv pour charger le .env
    if is_dev {
        println!("cargo:warning=Loading development environment");
        dotenv::dotenv().ok();
    }

    // Liste des variables à injecter
    let env_vars = vec![
        "IGDB_CLIENT_ID",
        "IGDB_CLIENT_SECRET",
        "STEAM_API_KEY",
        "EPIC_CLIENT_ID",
        "EPIC_CLIENT_ID_SECRET",
        "BATTLENET_CLIENT_ID",
        "BATTLENET_CLIENT_SECRET",
    ];

    // Injecter chaque variable
    for var in env_vars {
        let value = std::env::var(var).unwrap_or_default();
        println!("cargo:rustc-env={}", var);
        println!("cargo:rerun-if-env-changed={}", var);
    }
    glaunch_lib::run()
}
