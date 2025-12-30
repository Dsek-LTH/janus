use actix_web::Result;
use sqlx::{Pool, Postgres};

use crate::{
    discord::{DiscordOAuthCredentials, DiscordUserData},
    dsek::DsekUserData,
    server::from_server,
};

pub async fn fetch_dsek_username(db: &Pool<Postgres>, user_id: &str) -> Result<String> {
    let res = sqlx::query!(
        "SELECT stil_id
        FROM connected_accounts
        WHERE user_id = $1",
        user_id
    )
    .fetch_one(db)
    .await
    .map_err(from_server)?;

    Ok(res.stil_id)
}

pub async fn get_token(db: &Pool<Postgres>, user_id: &str) -> Result<DiscordOAuthCredentials> {
    let res = sqlx::query_as!(
        DiscordOAuthCredentials,
        "SELECT access_token, refresh_token, expires_at 
        FROM discord_tokens 
        WHERE user_id = $1",
        user_id
    )
    .fetch_one(db)
    .await
    .map_err(from_server)?;

    Ok(res)
}

pub async fn store_discord_token(
    db: &Pool<Postgres>,
    user_id: &str,
    oauth: DiscordOAuthCredentials,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO discord_tokens (user_id, access_token, refresh_token, expires_at) 
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id)
        DO UPDATE SET
        access_token = EXCLUDED.access_token,
        refresh_token = EXCLUDED.refresh_token,
        expires_at = EXCLUDED.expires_at",
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

pub async fn store_discord_user(db: &Pool<Postgres>, user_data: &DiscordUserData) -> Result<()> {
    sqlx::query!(
        "INSERT INTO authorized_discord_users (user_id, username)
        VALUES ($1, $2)
        ON CONFLICT (user_id)
        DO UPDATE SET
        user_id = EXCLUDED.user_id,
        username = EXCLUDED.username",
        user_data.user_id,
        user_data.username
    )
    .execute(db)
    .await
    .map_err(from_server)?;

    Ok(())
}

pub async fn store_dsek_user(db: &Pool<Postgres>, user_data: &DsekUserData) -> Result<()> {
    sqlx::query!(
        "INSERT INTO authorized_dsek_users (stil_id, name)
        VALUES ($1, $2)
        ON CONFLICT (stil_id)
        DO UPDATE SET
        stil_id = EXCLUDED.stil_id,
        name = EXCLUDED.name",
        user_data.stil_id,
        user_data.name
    )
    .execute(db)
    .await
    .map_err(from_server)?;

    Ok(())
}

pub async fn connect_users(db: &Pool<Postgres>, user_id: &str, stil_id: &str) -> Result<()> {
    sqlx::query!(
        "INSERT INTO connected_accounts (user_id, stil_id)
        VALUES ($1, $2)
        ON CONFLICT (stil_id)
        DO UPDATE SET
        user_id = EXCLUDED.user_id,
        stil_id = EXCLUDED.stil_id",
        user_id,
        stil_id
    )
    .execute(db)
    .await
    .map_err(from_server)?;

    Ok(())
}

pub async fn fetch_discord_username(db: &Pool<Postgres>, discord_user_id: &str) -> Result<String> {
    let res = sqlx::query!(
        "SELECT username
        FROM authorized_discord_users
        WHERE user_id = $1",
        discord_user_id
    )
    .fetch_one(db)
    .await
    .map_err(from_server)
    .map(|res| res.username)?;

    Ok(res)
}
