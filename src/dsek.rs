use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    str::FromStr,
};

use actix_session::Session;
use actix_web::Result;
use base64::{
    alphabet,
    engine::{
        general_purpose::{self},
        GeneralPurpose,
    },
    prelude::BASE64_URL_SAFE,
    Engine,
};
use serde::{de::Error, Deserialize};
use url_builder::URLBuilder;
use uuid::Uuid;

use crate::{env, server::from_server};

#[derive(Deserialize, Debug)]
pub struct DsekUserData {
    pub name: String,
    #[serde(rename = "preferred_username")]
    pub stil_id: String,
}

impl FromStr for DsekUserData {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let engine = GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);
        fn serde_err<T: Display>(e: T) -> serde_json::Error {
            serde_json::Error::custom(e)
        }

        let parts: Vec<&str> = s.split(".").collect();
        let payload_bytes = engine.decode(parts[1].as_bytes()).map_err(serde_err)?;
        let payload = String::from_utf8(payload_bytes).map_err(serde_err)?;
        serde_json::from_str(&payload)
    }
}

pub fn generate_oauth_url(session: &Session) -> String {
    let client_id = env::var("DSEK_CLIENT_ID");
    let redirect_uri = env::var("DSEK_REDIRECT_URI");

    let state = Uuid::new_v4().to_string();

    session
        .insert("dsek_uuid_state", &state)
        .expect("Could not insert state to session");

    let mut url = URLBuilder::new();
    url.set_protocol("https")
        .set_host("auth.dsek.se")
        .add_route("application/o/authorize/")
        .add_param("client_id", &client_id)
        .add_param("redirect_uri", &redirect_uri)
        .add_param("response_type", "code")
        .add_param("scope", "openid profile")
        .add_param("state", &state)
        .add_param("prompt", "consent");

    url.build()
}

pub async fn fetch_user_data(code: &str) -> Result<DsekUserData> {
    #[derive(Deserialize, Debug)]
    struct TokenResponse {
        id_token: String,
    }

    let token_endpoint = "https://auth.dsek.se/application/o/token/";
    let credentials = BASE64_URL_SAFE.encode(format!(
        "{}:{}",
        env::var("DSEK_CLIENT_ID"),
        env::var("DSEK_CLIENT_SECRET")
    ));

    let mut data = HashMap::new();

    data.insert("grant_type", "authorization_code".to_string());
    data.insert("code", code.to_string());
    data.insert("redirect_uri", env::var("DSEK_REDIRECT_URI"));

    // returns a base64-encoded token containing user data
    let TokenResponse { id_token } = reqwest::Client::new()
        .post(token_endpoint)
        .header("Accept", "application/json")
        .header("Authorization", format!("Basic {}", credentials))
        .form(&data)
        .send()
        .await
        .map_err(from_server)?
        .json()
        .await
        .map_err(from_server)?;

    // parses the encoded token into actual data, see `FromStr for DsekUserData`
    Ok(id_token.parse()?)
}
