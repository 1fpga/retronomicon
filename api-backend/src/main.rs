use crate::routes::{GitHubUserInfo, GoogleUserInfo};
use rocket::figment::value::{Map, Value};
use rocket::figment::{map, Provider};
use rocket::{get, http::Status, routes, serde::json::Json};
use rocket_db_pools::Database;
use rocket_oauth2::OAuth2;
use std::collections::BTreeMap;
use std::env;

mod db;
mod error;
mod fairings;
mod models;
mod schema;
mod user;

mod routes;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Frontend {
    pub base_url: String,
}

#[get("/healthcheck")]
async fn health_check_handler() -> Result<Json<String>, Status> {
    Ok(Json("ok".into()))
}

fn database_config() -> impl Provider {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool_size: u32 = str::parse(&env::var("DATABASE_POOL_SIZE").unwrap_or("10".to_string()))
        .expect("Invalid DATABASE_POOL_SIZE");

    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => pool_size.into(),
    };
    ("databases", map!["retronomicon_db" => db])
}

fn oauth_config() -> impl Provider {
    let mut oauth = BTreeMap::new();
    for (k, v) in env::vars() {
        if k.starts_with("ROCKET_OAUTH_") {
            let mut parts = k.splitn(4, '_');
            parts.next();
            parts.next();

            let provider = parts.next().unwrap();
            let key = parts.next().unwrap();
            let value = oauth
                .entry(provider.to_lowercase())
                .or_insert_with(BTreeMap::<String, String>::new);
            value.insert(key.to_lowercase(), v);
        }
    }

    ("oauth", oauth)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // The order is first, local environment variables, then global ones, then
    // only use development if in debug mode.
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();

    #[cfg(debug_assertions)]
    dotenvy::from_filename(".env.development").expect("Failed to load .env.development file");

    let figment = rocket::Config::figment()
        .merge(database_config())
        .merge(oauth_config());

    let v = figment.find_value("secret_key").unwrap();
    env::set_var("JWT_SECRET", v.into_string().unwrap());

    let rocket = rocket::custom(figment);
    let rocket = rocket
        .mount("/api", routes![health_check_handler])
        .mount("/api", routes::routes())
        .attach(db::RetronomiconDb::init())
        .attach(OAuth2::<GitHubUserInfo>::fairing("github"))
        .attach(OAuth2::<GoogleUserInfo>::fairing("google"))
        .attach(fairings::cors::CORS)
        .attach(rocket::fairing::AdHoc::config::<Frontend>());

    rocket.launch().await?;

    Ok(())
}
