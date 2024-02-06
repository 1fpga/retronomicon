use crate::ghenkins::UserParam;
use crate::user::User as CucumberUser;
use anyhow::{anyhow, Error};
use backend::fairings::config::{DbPepper, JwtKeys, RetronomiconConfig};
use backend::routes::v1;
use backend::{config, routes};
use cucumber::{writer, World as _};
use retronomicon_db as db;
use retronomicon_dto as dto;
use rocket::fairing::AdHoc;
use rocket::fs::relative;
use rocket::futures::lock::Mutex;
use rocket::local::asynchronous::Client;
use rocket_oauth2::OAuth2;
use std::collections::BTreeMap;
use std::sync::Arc;
use url::Url;

pub mod ghenkins;
pub mod user;

#[derive(cucumber::World, Debug)]
#[world(init = World::new)]
struct World {
    pub client: Arc<Client>,

    admins: BTreeMap<String, Arc<Mutex<CucumberUser>>>,
    users: BTreeMap<String, Arc<Mutex<CucumberUser>>>,
    teams: BTreeMap<String, dto::teams::TeamCreateResponse>,

    pub games: BTreeMap<String, i32>,
    pub systems: BTreeMap<String, i32>,

    last_result: Option<Result<String, Error>>,
}

impl World {
    pub async fn user(&mut self, name: &UserParam) -> Result<Arc<Mutex<CucumberUser>>, Error> {
        match name {
            UserParam::Admin(name) => {
                if !self.admins.contains_key(name) {
                    let user = CucumberUser::admin(self.client.clone(), name).await?;

                    self.admins
                        .insert(name.to_string(), Arc::new(Mutex::new(user)));
                }

                Ok(self.admins.get(name).expect("Just created admin").clone())
            }
            UserParam::User(name) => {
                if !self.users.contains_key(name) {
                    let user = CucumberUser::user(self.client.clone(), name).await?;

                    self.users
                        .insert(name.to_string(), Arc::new(Mutex::new(user)));
                }

                Ok(self.users.get(name).expect("Just created user").clone())
            }
            UserParam::Anonymous => Ok(Arc::new(Mutex::new(
                CucumberUser::anonymous(self.client.clone()).await?,
            ))),
        }
    }

    pub async fn auth_user(&mut self, name: &UserParam) -> Result<Arc<Mutex<CucumberUser>>, Error> {
        let user = self.user(name).await?;
        user.lock().await.authenticate().await?;
        Ok(user)
    }

    pub async fn team(
        &mut self,
        owner: &UserParam,
        name: &str,
    ) -> Result<dto::teams::TeamCreateResponse, Error> {
        if !self.teams.contains_key(name) {
            let user = self.auth_user(owner).await?;
            let team = user.lock().await.create_team(name).await?;
            self.teams.insert(name.to_string(), team);
        }

        Ok(self.teams.get(name).expect("Just created team").clone())
    }

    async fn new() -> Self {
        // Relative to the root of the crate.
        let figment =
            config::create_figment(&[relative!("tests/Rocket.test.toml").into()], "debug").unwrap();

        let secret_key = figment
            .find_value("secret_key")
            .expect("No secret key.")
            .into_string()
            .unwrap();
        let jwt_secret_b64 = secret_key.clone();
        let db_pepper = secret_key.clone();

        // Double check that we're running against a localhost database.
        let db_url = Url::parse(
            &figment
                .find_value("databases.retronomicon_db.url")
                .expect("No database URL.")
                .into_string()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            db_url.host_str(),
            Some("localhost"),
            "Cucumber tests must run with a local database."
        );

        let rocket = rocket::custom(figment)
            .mount("/", v1::routes())
            .attach(db::RetronomiconDbPool::init())
            .attach(OAuth2::<routes::auth::GitHubUserInfo>::fairing("github"))
            .attach(OAuth2::<routes::auth::GoogleUserInfo>::fairing("google"))
            .attach(OAuth2::<routes::auth::PatreonUserInfo>::fairing("patreon"))
            .attach(AdHoc::config::<RetronomiconConfig>())
            .manage(JwtKeys::from_base64(&jwt_secret_b64))
            .manage(DbPepper::from_base64(&db_pepper));
        let client = Arc::new(
            Client::untracked(rocket)
                .await
                .expect("Rocket client failed"),
        );

        Self {
            client,
            admins: BTreeMap::new(),
            users: BTreeMap::new(),
            teams: BTreeMap::new(),
            games: BTreeMap::new(),
            systems: BTreeMap::new(),
            last_result: None,
        }
    }

    pub fn record_result<T: serde::Serialize>(&mut self, result: Result<T, Error>) {
        self.last_result = Some(
            result
                .map(|v| serde_json::to_string_pretty(&v).expect("Serialization of error failed."))
                .map_err(|e| anyhow!(e.to_string())),
        );
    }

    pub fn assert_result_ok(&mut self) {
        if let Some(Err(ref e)) = self.last_result {
            panic!("Expected Ok, got Err: {}", e);
        }
    }

    pub fn assert_result_err(&mut self) {
        if let Some(Ok(ref e)) = self.last_result {
            panic!("Expected Err, got Ok: {}", e);
        }
    }

    pub fn reset_result(&mut self) {
        self.last_result = None;
    }
}

#[tokio::main]
async fn main() {
    World::cucumber()
        .max_concurrent_scenarios(1)
        .with_writer(writer::Libtest::or_basic())
        .run_and_exit("tests/features/")
        .await;
}
