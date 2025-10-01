use crate::{
    env,
    server::{from_server, AppState},
    storage::{self},
};
use actix_session::Session;
use actix_web::{web, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url_builder::URLBuilder;
use uuid::Uuid;

pub struct DiscordOAuthCredentials {
    pub access_token: String,
    pub expires_at: i64,
    pub refresh_token: String,
}

#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
}

impl From<DiscordTokenResponse> for DiscordOAuthCredentials {
    fn from(oauth_resp: DiscordTokenResponse) -> Self {
        DiscordOAuthCredentials {
            access_token: oauth_resp.access_token,
            refresh_token: oauth_resp.refresh_token,
            expires_at: Utc::now().timestamp() + oauth_resp.expires_in,
        }
    }
}

pub enum AuthMethod {
    Code(String),
    Refresh(DiscordOAuthCredentials),
}

#[derive(Deserialize)]
struct AuthDataResponse {
    user: DiscordUserData,
}

#[derive(Serialize)]
struct MetadataUpdate {
    platform_name: String,
    platform_username: String,
    metadata: Metadata,
}
#[derive(Serialize)]
struct Metadata {
    dsek_member: bool,
}

#[derive(Deserialize, Debug)]
pub struct DiscordUserData {
    #[serde(rename = "id")]
    pub user_id: String,
    pub username: String,
}

pub fn generate_oauth_url(session: &Session) -> String {
    let client_id = env::var("DISCORD_CLIENT_ID");
    let redirect_uri = env::var("DISCORD_REDIRECT_URI");

    let state = Uuid::new_v4().to_string();

    session
        .insert("discord_uuid_state", &state)
        .expect("Could not insert state to session");

    let mut url = URLBuilder::new();
    url.set_protocol("https")
        .set_host("discord.com")
        .add_route("api/oauth2/authorize")
        .add_param("client_id", &client_id)
        .add_param("redirect_uri", &redirect_uri)
        .add_param("response_type", "code")
        .add_param("state", &state)
        .add_param("scope", "identify+role_connections.write")
        .add_param("prompt", "consent");

    url.build()
}

pub async fn fetch_oauth_tokens(method: AuthMethod) -> Result<DiscordOAuthCredentials> {
    let endpoint = "https://discord.com/api/v10/oauth2/token";
    let mut data = HashMap::new();

    match method {
        AuthMethod::Code(code) => {
            data.insert("grant_type", "authorization_code".to_string());
            data.insert("code", code);
            data.insert("redirect_uri", env::var("DISCORD_REDIRECT_URI"));
        }
        AuthMethod::Refresh(oauth) => {
            data.insert("grant_type", "refresh_token".to_string());
            data.insert("refresh_token", oauth.refresh_token);
        }
    }

    data.insert("client_id", env::var("DISCORD_CLIENT_ID"));
    data.insert("client_secret", env::var("DISCORD_CLIENT_SECRET"));

    let res: DiscordOAuthCredentials = reqwest::Client::new()
        .post(endpoint)
        .form(&data)
        .send()
        .await
        .map_err(from_server)?
        .json::<DiscordTokenResponse>()
        .await
        .map_err(from_server)?
        .into();

    Ok(res)
}

pub async fn fetch_user_auth_data(access_token: &str) -> Result<DiscordUserData> {
    let endpoint = "https://discord.com/api/v10/oauth2/@me";

    let AuthDataResponse { user } = reqwest::Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(from_server)?
        .json()
        .await
        .map_err(from_server)?;

    Ok(user)
}

pub async fn update_metadata(data: &web::Data<AppState>, user_id: &str) -> Result<()> {
    let mut oauth = storage::get_token(&data.db, user_id).await?;
    if oauth.expires_at <= Utc::now().timestamp() {
        oauth = fetch_oauth_tokens(AuthMethod::Refresh(oauth)).await?;
    }

    let stil_id = storage::fetch_dsek_username(&data.db, user_id).await?;

    let metadata = Metadata { dsek_member: true };
    let udata = MetadataUpdate {
        platform_name: "D-sektionen inom TLTH".to_string(),
        platform_username: stil_id,
        metadata,
    };

    let endpoint = format!(
        "https://discord.com/api/v10/users/@me/applications/{}/role-connection",
        env::var("DISCORD_CLIENT_ID")
    );

    reqwest::Client::new()
        .put(endpoint)
        .json(&udata)
        .header("Authorization", format!("Bearer {}", oauth.access_token))
        .send()
        .await
        .map_err(from_server)?
        .error_for_status()
        .map_err(from_server)?;

    storage::store_discord_token(&data.db, user_id, oauth).await?;

    Ok(())
}
