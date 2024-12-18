use crate::utils::AppError;
use reqwest::Client;

pub struct SteamAuth {
    client: Client,
    api_key: String,
}

impl SteamAuth {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub fn get_auth_url(&self) -> String {
        // L'URL d'authentification Steam OpenID
        format!(
            "https://steamcommunity.com/openid/login\
            ?openid.ns=http://specs.openid.net/auth/2.0\
            &openid.mode=checkid_setup\
            &openid.return_to=http://localhost:11111/auth/steam/callback\
            &openid.realm=http://localhost:11111/auth/steam/callback\
            &openid.identity=http://specs.openid.net/auth/2.0/identifier_select\
            &openid.claimed_id=http://specs.openid.net/auth/2.0/identifier_select"
        )
    }

    // Change to instance method with &self
    pub fn extract_steam_id(&self, identity: &str) -> Option<String> {
        identity
            .split('/')
            .last()
            .filter(|id| id.chars().all(char::is_numeric))
            .map(String::from)
    }

    pub async fn get_profile(&self, steam_id: &str) -> Result<String, AppError> {
        let url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
            self.api_key, steam_id
        );

        let response = self.client.get(&url).send().await.map_err(|e| AppError {
            message: format!("Failed to fetch Steam profile: {}", e),
        })?;

        response.text().await.map_err(|e| AppError {
            message: format!("Failed to read response: {}", e),
        })
    }
}
