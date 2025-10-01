use actix_web::Result;
use sqlx::{Pool, Sqlite};

use crate::{
    discord::{DiscordOAuthCredentials, DiscordUserData},
    dsek::DsekUserData,
    server::from_server,
};

pub async fn fetch_dsek_username(db: &Pool<Sqlite>, user_id: &str) -> Result<String> {
    let res = sqlx::query!(
        "SELECT stil_id
        FROM connected_accounts
        WHERE user_id = ?",
        user_id
    )
    .fetch_one(db)
    .await
    .map_err(from_server)?;

    Ok(res.stil_id)
}

pub async fn get_token(db: &Pool<Sqlite>, user_id: &str) -> Result<DiscordOAuthCredentials> {
    let res = sqlx::query_as!(
        DiscordOAuthCredentials,
        "SELECT access_token, refresh_token, expires_at 
        FROM discord_tokens 
        WHERE user_id = ?",
        user_id
    )
    .fetch_one(db)
    .await
    .map_err(from_server)?;

    Ok(res)
}

pub async fn store_discord_token(
    db: &Pool<Sqlite>,
    user_id: &str,
    oauth: DiscordOAuthCredentials,
) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO discord_tokens (user_id, access_token, refresh_token, expires_at) 
        VALUES (?, ?, ?, ?)",
        user_id,
        oauth.access_token,
        oauth.refresh_token,
        oauth.expires_at
    )
    .execute(db)
    .await
    .map_err(from_server)?;

    Ok(())
}

pub async fn store_discord_user(db: &Pool<Sqlite>, user_data: &DiscordUserData) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO authorized_discord_users (user_id, username)
        VALUES (?, ?)",
        user_data.user_id,
        user_data.username
    )
    .execute(db)
    .await
    .map_err(from_server)?;

    Ok(())
}

pub async fn store_dsek_user(db: &Pool<Sqlite>, user_data: &DsekUserData) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO authorized_dsek_users (stil_id, name)
        VALUES (?, ?)",
        user_data.stil_id,
        user_data.name
    )
    .execute(db)
    .await
    .map_err(from_server)?;

    Ok(())
}

pub async fn connect_users(db: &Pool<Sqlite>, user_id: &str, stil_id: &str) -> Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO connected_accounts (user_id, stil_id)
        VALUES (?, ?)",
        user_id,
        stil_id
    )
    .execute(db)
    .await
    .map_err(from_server)?;

    Ok(())
}

pub async fn fetch_discord_username(db: &Pool<Sqlite>, discord_user_id: &str) -> Result<String> {
    let res = sqlx::query!(
        "SELECT username
        FROM authorized_discord_users
        WHERE user_id = ?",
        discord_user_id
    )
    .fetch_one(db)
    .await
    .map_err(from_server)
    .map(|res| res.username)?;

    Ok(res)
}
