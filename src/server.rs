use std::fmt::{Debug, Display};

use crate::{
    discord::{self, AuthMethod},
    dsek, env, storage,
};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    error::{ErrorForbidden, InternalError},
    get,
    web::{self, Redirect},
    App, Error, HttpResponse, HttpServer, ResponseError, Result,
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite, SqlitePool};

/// Converts any error into a 500 Internal Server Error response.
pub fn from_server<T: Debug + Display>(err: T) -> impl ResponseError {
    InternalError::new(
        format!("{:?}", err),
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
    )
}

fn verify_state(session: &Session, oauth_return: &web::Query<OAuthReturn>, state_name: &str) -> Result<()> {
    if let Some(state) = session.get::<String>(state_name)? {
        if state != oauth_return.state {
            return Err(ErrorForbidden(
                "OAuth state not same as cached state.\nWho are you...?".to_string(),
            ));
        }
    } else {
        return Err(ErrorForbidden(
            "OAuth state not found.\nWho are you...?".to_string(),
        ));
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct OAuthReturn {
    code: String,
    state: String,
}

pub struct AppState {
    pub db: Pool<Sqlite>,
}

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("hej"))
}

#[get("/linked-role")]
async fn linked_role(session: Session) -> Result<Redirect> {
    let url = discord::generate_oauth_url(&session);
    Ok(Redirect::to(url).temporary())
}

#[get("/discord-oauth-callback")]
async fn discord_oauth_callback(
    session: Session,
    oauth_return: web::Query<OAuthReturn>,
    data: web::Data<AppState>,
) -> Result<Redirect> {
    verify_state(&session, &oauth_return, "discord_uuid_state")?;

    let method = AuthMethod::Code(oauth_return.code.clone());
    let oauth_token = discord::fetch_oauth_tokens(method).await?;

    let user_data = discord::fetch_user_auth_data(&oauth_token.access_token).await?;
    storage::store_discord_user(&data.db, &user_data).await?;
    storage::store_discord_token(&data.db, &user_data.user_id, oauth_token).await?;
    session.insert("discord_user_id", &user_data.user_id)?;

    let dsek_redirect_url = dsek::generate_oauth_url(&session);
    Ok(Redirect::to(dsek_redirect_url).temporary())
}

#[get("/dsek-oauth-callback")]
async fn dsek_oauth_callback(
    session: Session,
    oauth_return: web::Query<OAuthReturn>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    verify_state(&session, &oauth_return, "dsek_uuid_state")?;

    let dsek_user = dsek::fetch_user_data(&oauth_return.code).await?;

    let discord_user_id = session
        .get::<String>("discord_user_id")?
        .ok_or("Could not found Discord UserID in session")
        .map_err(from_server)?;

    let discord_username = storage::fetch_discord_username(&data.db, &discord_user_id).await?;

    storage::store_dsek_user(&data.db, &dsek_user).await?;
    storage::connect_users(&data.db, &discord_user_id, &dsek_user.stil_id).await?;
    discord::update_metadata(&data, &discord_user_id).await?;

    Ok(HttpResponse::Ok().body(format!(
        "Successfully linked Discord account ({}) with Dsek account ({}). You may return to Discord and close this tab.", 
        discord_username, 
        dsek_user.stil_id
    )))
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let db_url = env::var("DATABASE_URL");
    let storage = SqlitePool::connect(&db_url)
        .await
        .map_err(|_| std::io::ErrorKind::ConnectionRefused)?;

    HttpServer::new(move || {
        let cookie_store = {
            let key = [0; 64];
            // should it just be random? leaving it commented out for now
            // rand::rng().fill(&mut key);
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&key)).build()
        };

        let app_state = web::Data::new(AppState {
            db: storage.clone(),
        });

        App::new()
            .wrap(cookie_store)
            .app_data(app_state)
            .service(index)
            .service(linked_role)
            .service(discord_oauth_callback)
            .service(dsek_oauth_callback)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
