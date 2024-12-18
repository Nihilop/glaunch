pub mod battlenet;
pub mod epic;
pub mod steam;
pub mod server;

pub use battlenet::BattleNetAuth;
pub use epic::EpicAuth;
pub use epic::EpicTokenResponse;
pub use steam::SteamAuth;
pub use server::AuthServer;

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
