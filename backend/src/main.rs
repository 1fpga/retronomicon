use crate::fairings::config::{DbPepper, JwtKeys, RetronomiconConfig};
use crate::routes::v1;
use clap::Parser;
use retronomicon_db::{run_migrations, RetronomiconDbPool};
use rocket::fairing::AdHoc;
use rocket::fs::relative;
use rocket::response::status::NoContent;
use rocket::{get, http::Status, routes};
use rocket_oauth2::OAuth2;
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use std::env;
use std::path::PathBuf;

mod config;
mod fairings;
mod guards;

mod routes;
mod utils;

#[derive(Debug, Parser)]
struct Opts {
    /// Additional configuration files for Rocket (in toml).
    #[clap(long)]
    rocket: Vec<PathBuf>,
}

#[get("/healthz")]
async fn health_handler() -> Result<NoContent, Status> {
    Ok(NoContent)
}

#[rocket::launch]
async fn rocket() -> _ {
    let opts = Opts::parse();
    rocket::info!("Opts: {:?}", opts);

    let figment = config::create_figment(&opts.rocket, "debug").unwrap();

    run_migrations(
        figment
            .find_value("databases.retronomicon_db.url")
            .unwrap()
            .as_str()
            .unwrap(),
    );

    let secret_key = figment
        .find_value("secret_key")
        .expect("No secret key.")
        .into_string()
        .unwrap();

    let jwt_secret_b64 = figment
        .extract_inner::<String>("jwt_secret")
        .or_else(|_| env::var("JWT_SECRET"))
        .unwrap_or_else(|_| secret_key.clone());
    let db_pepper = figment
        .extract_inner::<String>("db_pepper")
        .or_else(|_| env::var("DATABASE_PEPPER"))
        .unwrap_or_else(|_| secret_key.clone());

    let static_root: Option<String> = figment
        .extract_inner("static_root")
        .ok()
        .or_else(|| env::var("STATIC_ROOT").ok());

    #[cfg(debug_assertions)]
    let static_root = static_root.or_else(|| Some(relative!("../frontend/build").to_string()));

    if static_root.is_none() {
        rocket::warn!("No static root set, serving no static files.");
    }

    let prometheus = rocket_prometheus::PrometheusMetrics::new();

    rocket::custom(figment)
        // The health endpoint.
        .mount("/", routes![health_handler])
        .mount("/api", routes::routes())
        // The v1 actual API endpoints.
        .mount("/api/v1", v1::routes())
        .mount(
            "/api/swagger",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .mount("/metrics", prometheus.clone())
        .mount(
            "/",
            rocket::fs::FileServer::new(
                static_root.unwrap_or_else(|| "/dev/null".to_string()),
                rocket::fs::Options::Index | rocket::fs::Options::Missing,
            ),
        )
        .attach(RetronomiconDbPool::init())
        .attach(prometheus)
        .attach(OAuth2::<routes::auth::GitHubUserInfo>::fairing("github"))
        .attach(OAuth2::<routes::auth::GoogleUserInfo>::fairing("google"))
        .attach(OAuth2::<routes::auth::PatreonUserInfo>::fairing("patreon"))
        .attach(fairings::cors::Cors)
        .manage(JwtKeys::from_base64(&jwt_secret_b64))
        .manage(DbPepper::from_base64(&db_pepper))
        .attach(AdHoc::config::<RetronomiconConfig>())
}
