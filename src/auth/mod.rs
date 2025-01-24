use serde::{Deserialize, Serialize};

pub mod microsoft;

#[derive(Serialize, Deserialize)]
pub enum AuthMethod {
    Microsoft {
        access_token: String,
        refresh_token: String,
        uuid: String,
        xuid: String,
        username: String
    },
    Offline {
        username: String,
        uuid: Option<String>
    }
}