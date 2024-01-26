use crate::guards::emailer::SmtpConfig;
use crate::routes::v1;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use jsonwebtoken::{DecodingKey, EncodingKey};
use retronomicon_db::{run_migrations, RetronomiconDbPool};
use rocket::figment::providers::Env;
use rocket::figment::value::{Map, Value};
use rocket::figment::{map, Provider};
use rocket::response::status::NoContent;
use rocket::{get, http::Status, routes};
use rocket_oauth2::OAuth2;
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

mod fairings;
mod guards;

mod routes;
mod utils;

#[derive(Debug, serde::Deserialize)]
pub struct RetronomiconConfig {
    pub base_url: String,
    pub root_team: Vec<String>,
    pub root_team_id: i32,

    pub smtp: SmtpConfig,
}

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    pub fn from_base64(secret: &str) -> Self {
        let secret = STANDARD.decode(secret).expect("Invalid base64 JWT secret");
        let encoding = EncodingKey::from_secret(&secret);
        let decoding = DecodingKey::from_secret(&secret);
        Self { encoding, decoding }
    }
}

pub struct DbPepper(pub Vec<u8>);

impl DbPepper {
    pub fn from_base64(secret: &str) -> Self {
        let secret = STANDARD.decode(secret).expect("Invalid base64 pepper");
        Self(secret)
    }
}

#[get("/healthz")]
async fn health_handler() -> Result<NoContent, Status> {
    Ok(NoContent)
}

fn database_config() -> impl Provider {
    let db_url = Some(Value::from(
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    ));
    let pool_size = env::var("DATABASE_POOL_SIZE")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .map(Value::from);
    let certs_files: Option<Value> = env::var("DATABASE_CERTS")
        .ok()
        .map(|e| Value::from(e.split(';').map(|c| Value::from(c)).collect::<Vec<_>>()));

    let db: Map<_, Option<Value>> = map! {
        "url" => db_url.into(),
        "pool_size" => pool_size,
        "certs" => certs_files.into(),
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

#[rocket::launch]
async fn rocket() -> _ {
    // The order is first, local environment variables, then global ones, then
    // only use development if in debug mode.
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();

    #[cfg(debug_assertions)]
    dotenvy::from_filename(".env.development").ok();

    run_migrations();

    let figment = rocket::Config::figment()
        .merge(database_config())
        .merge(oauth_config())
        .merge(Env::prefixed("APP_").split("_"));

    let secret_key = figment
        .find_value("secret_key")
        .expect("No secret key.")
        .into_string()
        .unwrap();
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| secret_key.clone());
    let db_pepper = env::var("DATABASE_PEPPER").unwrap_or_else(|_| secret_key.clone());

    let static_root = figment
        .find_value("static_root")
        .ok()
        .and_then(|v| v.into_string())
        .unwrap_or_else(|| {
            env::var("STATIC_ROOT").ok().unwrap_or_else(|| {
                env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("static")
                    .to_string_lossy()
                    .to_string()
            })
        });

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
            rocket::fs::FileServer::from(PathBuf::from(static_root)),
        )
        .attach(RetronomiconDbPool::init())
        .attach(prometheus)
        .attach(OAuth2::<routes::auth::GitHubUserInfo>::fairing("github"))
        .attach(OAuth2::<routes::auth::GoogleUserInfo>::fairing("google"))
        .attach(OAuth2::<routes::auth::PatreonUserInfo>::fairing("patreon"))
        .attach(fairings::cors::Cors)
        .manage(JwtKeys::from_base64(&jwt_secret))
        .manage(DbPepper::from_base64(&db_pepper))
        .manage(guards::storage::StorageConfig {
            region: env::var("AWS_REGION").expect("AWS_REGION environment variable must be set"),
            cores_bucket: env::var("AWS_CORES_BUCKET").unwrap_or("retronomicon-cores".to_string()),
            cores_url_base: env::var("AWS_CORES_URL_BASE")
                .unwrap_or("https://cores.retronomicon.land".to_string()),
        })
        .attach(rocket::fairing::AdHoc::config::<RetronomiconConfig>())
}
