pub mod battlenet;
pub mod epic;
pub mod server;
pub mod steam;

pub use battlenet::BattleNetAuth;
pub use epic::EpicAuth;
pub use epic::EpicTokenResponse;
pub use server::AuthServer;
pub use steam::SteamAuth;

// Types communs pour l'authentification
#[derive(Debug)]
pub enum AuthPlatform {
    Steam,
    Epic,
    BattleNet,
}

#[derive(Debug)]
pub struct AuthCallback {
    pub platform: AuthPlatform,
    pub code: String,
}
