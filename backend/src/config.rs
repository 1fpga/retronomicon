use rocket::figment::value::{Map, Value};
use rocket::figment::{map, Provider};
use std::collections::BTreeMap;
use std::env;

pub fn database_config() -> impl Provider {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool_size: u32 = str::parse(&env::var("DATABASE_POOL_SIZE").unwrap_or("10".to_string()))
        .expect("Invalid DATABASE_POOL_SIZE");
    let certs_files: Vec<String> = env::var("DATABASE_CERTS")
        .ok()
        .map(|e| e.split(';').map(|c| c.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => pool_size.into(),
        "certs" => certs_files.into(),
    };
    ("databases", map!["retronomicon_db" => db])
}

pub fn oauth_config() -> impl Provider {
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
