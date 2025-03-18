use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, Scope, TokenUrl};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{error::Error, http::fetch::fetch_with_options, util::base64::decode_base64};

/// The client ID for Microsoft authentication.
pub static CLIENT_ID: &str = "00000000402b5328";
/// The redirect URI for Microsoft authentication.
pub static REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
/// The authorization URL for Microsoft authentication.
pub static AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf";
/// The token URL for Microsoft authentication.
pub static TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";

/// Represents the token received from Microsoft after authentication.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MSToken {
    access_token: String,
    refresh_token: String,
}

/// Represents the token received from Xbox Live after authentication.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct XboxToken {
    issue_instant: String,
    not_after: String,
    token: String,
}

/// Represents the response from the Minecraft API after successful authentication.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftResponse {
    pub username: String,
    pub access_token: String,
    pub expires_in: u32,
}

/// Represents the token received from Xbox Live's XSTS service.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct XstsToken {
    token: String,
    display_claims: DisplayClaims,
}

/// Represents the display claims returned by the XSTS token.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

/// Represents a user's Xbox Live identity.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Xui {
    uhs: String,
}

/// Represents a user's skin in Minecraft.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Skin {
    id: String,
    state: String,
    url: String,
    variant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
}

/// Represents a user's cape in Minecraft.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Cape {
    id: String,
    state: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
}

/// Represents a user's profile in Minecraft, including skins and capes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserProfile {
    pub id: Option<String>,
    pub name: Option<String>,
    skins: Option<Vec<Skin>>,
    capes: Option<Vec<Cape>>,
    path: Option<String>,
    error: Option<String>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

/// Represents the decoded JWT from Minecraft authentication.
#[derive(Debug, Deserialize, Clone)]
pub struct MCJWTDecoded {
    xuid: String,
    exp: u64,
}

/// Represents a Minecraft account with authentication details.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct MinecraftAccount {
    pub xuid: String,
    pub exp: u64,
    pub uuid: String,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub client_id: String,
}

/// Creates the authorization link for Microsoft authentication.
///
/// # Returns
/// A result containing the authorization URL as a string.
pub fn create_link() -> crate::Result<String> {
    let auth_url = AuthUrl::new(AUTH_URL.to_string())?;
    let token_url = TokenUrl::new(TOKEN_URL.to_string())?;

    let client = oauth2::basic::BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string())?);

    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "service::user.auth.xboxlive.com::MBI_SSL".to_string(),
        ))
        .url();

    Ok(authorize_url.to_string())
}

/// Authenticates the user using the provided authorization code.
///
/// # Parameters
/// - `code`: The authorization code received from the Microsoft authentication process.
/// - `client`: The HTTP client used for making requests.
///
/// # Returns
/// A result containing the authenticated `MinecraftAccount`.
pub async fn authenticate(
    code: String,
    client: &Client,
) -> crate::Result<MinecraftAccount> {
    let ms_token = get_ms_token(&code, client).await?;
    let xbox_token = get_xbox_token(&ms_token.access_token, client).await?;
    let xsts_token = get_xsts_token(&xbox_token.token, client).await?;
    let userhash = xsts_token
        .display_claims
        .xui
        .first()
        .ok_or(Error::Authentication("No XUI claims found.".to_string()))?
        .uhs
        .clone();

    obtain_minecraft_account(&xsts_token.token, &userhash, ms_token.refresh_token, client).await
}

/// Refreshes the access token using the provided refresh token.
///
/// # Parameters
/// - `refresh_token`: The refresh token used to obtain a new access token.
/// - `client`: The HTTP client used for making requests.
///
/// # Returns
/// A result containing the refreshed `MinecraftAccount`.
pub async fn refresh(
    refresh_token: String,
    client: &Client,
) -> crate::Result<MinecraftAccount> {
    let token_response = client
        .post(TOKEN_URL)
        .form(&[ 
            ("client_id", CLIENT_ID),
            ("scope", "service::user.auth.xboxlive.com::MBI_SSL"),
            ("grant_type", "refresh_token"),
            ("redirect_uri", REDIRECT_URI),
            ("refresh_token", &refresh_token),
        ])
        .send()
        .await?;

    let ms_token: MSToken = token_response.json().await?;
    let xbox_token = get_xbox_token(&ms_token.access_token, client).await?;
    let xsts_token = get_xsts_token(&xbox_token.token, client).await?;
    let userhash = xsts_token
        .display_claims
        .xui
        .first()
        .ok_or(Error::Authentication("No XUI claims found.".to_string()))?
        .uhs
        .clone();

    obtain_minecraft_account(&xsts_token.token, &userhash, ms_token.refresh_token, client).await
}

/// Obtains the Minecraft account details using the provided tokens.
///
/// # Parameters
/// - `xsts_token`: The XSTS token for authentication.
/// - `userhash`: The user hash obtained from the XSTS token.
/// - `refresh_token`: The refresh token for obtaining new access tokens.
/// - `client`: The HTTP client used for making requests.
///
/// # Returns
/// A result containing the authenticated `MinecraftAccount`.
async fn obtain_minecraft_account(
    xsts_token: &str,
    userhash: &str,
    refresh_token: String,
    client: &Client,
) -> crate::Result<MinecraftAccount> {
    let token = get_minecraft_token(xsts_token, userhash, client).await?;
    let profile = get_profile(token.access_token.clone()).await?;
    let jwt = parse_login_token(&token.access_token)?;

    Ok(MinecraftAccount {
        xuid: jwt.xuid,
        exp: jwt.exp,
        uuid: profile.id.unwrap_or_default(),
        username: profile.name.unwrap_or_default(),
        access_token: token.access_token,
        refresh_token,
        client_id: CLIENT_ID.to_string(),
    })
}

/// Retrieves the Microsoft token using the provided authorization code.
///
/// # Parameters
/// - `code`: The authorization code received from the Microsoft authentication process.
/// - `client`: The HTTP client used for making requests.
///
/// # Returns
/// A result containing the `MSToken`.
async fn get_ms_token(code: &str, client: &Client) -> crate::Result<MSToken> {
    let token_response = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("scope", "service::user.auth.xboxlive.com::MBI_SSL"),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await?;

    let ms_token: MSToken = token_response.json().await?;
    Ok(ms_token)
}

/// Retrieves the Xbox token using the provided Microsoft token.
///
/// # Parameters
/// - `ms_token`: The Microsoft access token.
/// - `client`: The HTTP client used for making requests.
///
/// # Returns
/// A result containing the `XboxToken`.
async fn get_xbox_token(ms_token: &str, client: &Client) -> crate::Result<XboxToken> {
    let body = serde_json::json!( {
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": ms_token
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    fetch_token(
        "https://user.auth.xboxlive.com/user/authenticate",
        body,
        client,
    )
    .await
}

/// Retrieves the XSTS token using the provided Xbox token.
///
/// # Parameters
/// - `xbox_token`: The Xbox token for authentication.
/// - `client`: The HTTP client used for making requests.
///
/// # Returns
/// A result containing the `XstsToken`.
async fn get_xsts_token(xbox_token: &str, client: &Client) -> crate::Result<XstsToken> {
    let body = serde_json::json!( {
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbox_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    fetch_token(
        "https://xsts.auth.xboxlive.com/xsts/authorize",
        body,
        client,
    )
    .await
}

/// Fetches a token from the specified URL using the provided body.
///
/// # Parameters
/// - `url`: The URL to fetch the token from.
/// - `body`: The body of the request containing necessary parameters.
/// - `client`: The HTTP client used for making requests.
///
/// # Returns
/// A result containing the deserialized response of type `T`.
async fn fetch_token<T: for<'de> Deserialize<'de>>(
    url: &str,
    body: serde_json::Value,
    client: &Client,
) -> crate::Result<T> {
    let token_response: T = fetch_with_options(
        url,
        Some(crate::http::fetch::FetchOptions {
            method: Method::POST,
            headers: HashMap::default(),
            query_params: HashMap::default(),
            body: Some(body),
        }),
        client,
    )
    .await?;

    Ok(token_response)
}

/// Returns player's Minecraft data.
///
/// # Parameters
/// - `xsts_token`: Xbox token.
/// - `userhash`: Hash value.
/// - `client`: Reqwest client.
/// 
/// # Returns
/// A result containing the `MinecraftResponse`.
async fn get_minecraft_token(
    xsts_token: &str,
    userhash: &str,
    client: &Client,
) -> crate::Result<MinecraftResponse> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", userhash, xsts_token)
    });

    fetch_with_options(
        "https://api.minecraftservices.com/authentication/login_with_xbox",
        Some(crate::http::fetch::FetchOptions {
            method: Method::POST,
            headers: HashMap::default(),
            query_params: HashMap::default(),
            body: Some(body),
        }),
        client,
    )
    .await
}

/// Parses login token.
///
/// # Parameters
/// - `mc_token`: Token to be parsed.
///
/// # Returns
/// A result containing the `MCJWTDecoded`.
fn parse_login_token(mc_token: &str) -> crate::Result<MCJWTDecoded> {
    let base64_url = mc_token
        .split('.')
        .nth(1)
        .ok_or(Error::MalformedToken(mc_token.to_string()))?;

    let decoded_bytes = decode_base64(base64_url)?;
    let json_payload = String::from_utf8(decoded_bytes)?;

    let decoded: MCJWTDecoded = serde_json::from_str(&json_payload)?;

    Ok(decoded)
}

/// Retrieves the Minecraft profile using the provided access token.
///
/// # Parameters
/// - `access_token`: The access token for authentication.
///
/// # Returns
/// A result containing the `UserProfile`.
async fn get_profile(access_token: String) -> crate::Result<UserProfile> {
    let api_url = "https://api.minecraftservices.com/minecraft/profile";
    let client = Client::new();

    let response = client
        .get(api_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    let profile = response.json::<UserProfile>().await?;

    if let Some(error) = profile.error {
        match error.as_str() {
            "NOT_FOUND" => Err(Error::Authentication(
                "Account does not own Minecraft.".to_string(),
            )),
            _ => Err(Error::Authentication(error)),
        }
    } else {
        Ok(profile)
    }
}

/// Validates the expiration time of the token.
///
/// # Parameters
/// - `exp`: The expiration time of the token.
///
/// # Returns
/// A boolean indicating whether the token is still valid.
pub fn validate(exp: u64) -> bool {
    exp > SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System time error")
        .unwrap()
        .as_secs()
}