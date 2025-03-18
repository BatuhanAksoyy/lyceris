use serde::{Deserialize, Serialize};

pub mod microsoft;

/// Represents the authentication method used for logging into Minecraft.
#[derive(Serialize, Deserialize, Clone)]
pub enum AuthMethod {
    /// Represents offline authentication with a username.
    Offline { username: String, uuid: Option<String> },
    /// Represents Microsoft account authentication.
    Microsoft {
        username: String,
        xuid: String,
        uuid: String,
        access_token: String,
        refresh_token: String
    },
}