pub mod battlenet;
pub mod epic;
pub mod steam;
pub mod traits;
use crate::Database;
use std::sync::Arc;
use tauri::AppHandle;

pub use battlenet::BattleNetPlatform;
pub use epic::EpicPlatform;
pub use steam::SteamPlatform;
pub use traits::*;

// Factory pour créer les instances de plateformes
pub fn create_platform(
    platform_type: &str,
    database: Arc<Database>,
    app_handle: AppHandle,
) -> Box<dyn GamePlatform> {
    match platform_type {
        "steam" => Box::new(SteamPlatform::new(database.clone())),
        "battlenet" => Box::new(BattleNetPlatform::new(database.clone(), app_handle)),
        "epic" => Box::new(EpicPlatform::new(database.clone())),
        _ => panic!("Unknown platform type: {}", platform_type),
    }
}
